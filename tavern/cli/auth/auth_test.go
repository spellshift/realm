package auth_test

import (
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"net/url"
	"os"
	"path/filepath"
	"strconv"
	"sync/atomic"
	"testing"
	"time"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/cli/auth"
	tavernauth "realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent/enttest"
	tavernhttp "realm.pub/tavern/internal/http"
)

func TestTokenAuthenticate(t *testing.T) {
	// Build Request
	req, err := http.NewRequest(http.MethodGet, "http://127.0.0.1/test", nil)
	require.NoError(t, err)

	// Authenticate Request
	token := auth.Token("test")
	token.Authenticate(req)

	// Ensure Header Set
	assert.Equal(t, string(token), req.Header.Get(tavernauth.HeaderAPIAccessToken))
}

func TestAuthenticate(t *testing.T) {
	// Setup Dependencies
	ctx := context.Background()

	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	// Create Test User
	existingAdmin := graph.User.Create().
		SetName("test_admin").
		SetOauthID("test_admin").
		SetPhotoURL("http://google.com/").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)
	existingUser := graph.User.Create().
		SetName("test_user").
		SetOauthID("test_user").
		SetPhotoURL("http://google.com/").
		SetIsActivated(true).
		SetIsAdmin(false).
		SaveX(ctx)

	// Test Cases
	tests := []struct {
		name            string
		sessionToken    string
		tavernURL       string
		wantAccessToken string
		wantError       error
	}{
		{
			name:            "Admin",
			sessionToken:    existingAdmin.SessionToken,
			tavernURL:       "http://127.0.0.1",
			wantAccessToken: existingAdmin.AccessToken,
		},
		{
			name:            "User",
			sessionToken:    existingUser.SessionToken,
			tavernURL:       "http://127.0.0.1",
			wantAccessToken: existingUser.AccessToken,
		},
		{
			name:         "InvalidURL",
			sessionToken: existingUser.SessionToken,
			tavernURL:    "💩://invalid",
			wantError:    auth.ErrInvalidURL,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			t.Setenv("TAVERN_USE_BROWSER_OAUTH", "1")

			// Setup Timeout
			deadline, ok := t.Deadline()
			if !ok {
				deadline = time.Now().Add(10 * time.Second)
			}
			ctx, cancel := context.WithDeadline(context.Background(), deadline)
			defer cancel()

			// Setup Browser
			browser := auth.BrowserFunc(func(tavernURL string) error {
				// Assert Redirect Port Set
				u, err := url.Parse(tavernURL)
				require.NoError(t, err)
				redirPort, err := strconv.Atoi(u.Query().Get(tavernauth.ParamTokenRedirPort))
				require.NoError(t, err)
				assert.LessOrEqual(t, redirPort, 65535)
				assert.Greater(t, redirPort, 0)

				// Setup Access Token Redirect Handler
				srv := tavernhttp.NewServer(
					tavernhttp.RouteMap{
						"/access_token/redirect": tavernhttp.Endpoint{
							Handler:          tavernauth.NewTokenRedirectHandler(),
							LoginRedirectURI: "/oauth/login",
						},
					},
					tavernhttp.WithAuthentication(graph),
				)

				// Set Session Token (if configured)
				r, err := http.NewRequest(http.MethodGet, tavernURL, nil)
				if err != nil {
					return err
				}
				if tc.sessionToken != "" {
					r.AddCookie(&http.Cookie{
						Name:     tavernauth.SessionCookieName,
						Value:    tc.sessionToken,
						Path:     "/",
						HttpOnly: true,
						Expires:  time.Now().Add(1 * time.Hour),
					})
				}

				// Handle HTTP Request
				w := httptest.NewRecorder()
				srv.ServeHTTP(w, r)
				assert.Equal(t, http.StatusFound, w.Result().StatusCode)

				// Parse Redirect URL
				redirURL := w.Result().Header.Get("Location")
				assert.NotEmpty(t, redirURL)
				if redirURL == "" {
					return fmt.Errorf("no redirect url was found")
				}

				// Follow Redirect
				redirReq, err := http.NewRequest(http.MethodGet, redirURL, nil)
				if err != nil {
					return err
				}
				_, err = http.DefaultClient.Do(redirReq)
				return err
			})

			// Authenticate
			token, err := auth.Authenticate(ctx, browser, tc.tavernURL)

			// Assertions
			assert.Equal(t, tc.wantAccessToken, string(token))
			assert.ErrorIs(t, err, tc.wantError)
		})
	}
}

// TestAuthenticate_CacheValid verifies that when the cache file contains a valid
// token, Authenticate returns it directly without triggering re-authentication.
func TestAuthenticate_CacheValid(t *testing.T) {
	const validToken = "valid-cached-token"

	// Tavern stand-in that accepts the cached token via the /graphql probe.
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/graphql" {
			http.Error(w, "unexpected path: "+r.URL.Path, http.StatusNotFound)
			return
		}
		if r.Header.Get(tavernauth.HeaderAPIAccessToken) == validToken {
			w.WriteHeader(http.StatusOK)
			_, _ = w.Write([]byte(`{"data":{"me":{"id":"1"}}}`))
			return
		}
		http.Error(w, "unauthorized", http.StatusUnauthorized)
	}))
	defer srv.Close()

	// Seed cache file with a valid token.
	cachePath := filepath.Join(t.TempDir(), "cache")
	require.NoError(t, os.WriteFile(cachePath, []byte(validToken), 0600))

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	// Browser must not be invoked when a valid cache exists.
	browser := auth.BrowserFunc(func(string) error {
		t.Fatalf("browser should not be opened when cache is valid")
		return nil
	})

	token, err := auth.Authenticate(ctx, browser, srv.URL, auth.WithCacheFile(cachePath))
	require.NoError(t, err)
	assert.Equal(t, validToken, string(token))

	// Cache file should still exist after a successful validation.
	_, statErr := os.Stat(cachePath)
	assert.NoError(t, statErr, "cache file should be preserved when cached token is valid")
}

// TestAuthenticate_CacheInvalidIsCleanedUp verifies that when the cache file
// contains a token the server rejects, the cache file is removed so that
// subsequent authentications do not reuse the invalid credentials.
func TestAuthenticate_CacheInvalidIsCleanedUp(t *testing.T) {
	const staleToken = "stale-cached-token"

	// Tavern stand-in that rejects all tokens with 401.
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "unauthorized", http.StatusUnauthorized)
	}))
	defer srv.Close()

	// Seed cache file with a stale token.
	cachePath := filepath.Join(t.TempDir(), "cache")
	require.NoError(t, os.WriteFile(cachePath, []byte(staleToken), 0600))

	// Use a short deadline since we expect Authenticate to fall through to the
	// remote device authentication flow (which will never complete in this
	// test). We only care that the cache file was removed beforehand.
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	_, _ = auth.Authenticate(ctx, nil, srv.URL, auth.WithCacheFile(cachePath))

	// Cache file must be removed after the server rejects the cached token.
	_, statErr := os.Stat(cachePath)
	assert.True(t, os.IsNotExist(statErr), "expected stale cache file to be removed, got err=%v", statErr)
}

// TestAuthenticate_CacheRetainedOnTransientFailure verifies that when token
// validation fails for a reason other than an unauthorized response (e.g. the
// tavern server is unreachable), the cache file is preserved and the cached
// token is returned optimistically.
func TestAuthenticate_CacheRetainedOnTransientFailure(t *testing.T) {
	const cachedToken = "cached-token"

	// Start and immediately close a server so the address is unreachable.
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {}))
	unreachableURL := srv.URL
	srv.Close()

	cachePath := filepath.Join(t.TempDir(), "cache")
	require.NoError(t, os.WriteFile(cachePath, []byte(cachedToken), 0600))

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	token, err := auth.Authenticate(ctx, nil, unreachableURL, auth.WithCacheFile(cachePath))
	require.NoError(t, err)
	assert.Equal(t, cachedToken, string(token))

	// Cache file must be preserved on transient errors.
	_, statErr := os.Stat(cachePath)
	assert.NoError(t, statErr, "cache file should be preserved when validation fails transiently")
}

// TestAuthenticate_EnvVarSkipsCacheValidation verifies that an explicit API key
// provided via environment variable short-circuits cache validation entirely,
// leaving the cache file untouched even if present.
func TestAuthenticate_EnvVarSkipsCacheValidation(t *testing.T) {
	var reqCount atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		reqCount.Add(1)
		http.Error(w, "unauthorized", http.StatusUnauthorized)
	}))
	defer srv.Close()

	cachePath := filepath.Join(t.TempDir(), "cache")
	require.NoError(t, os.WriteFile(cachePath, []byte("should-not-be-read"), 0600))

	t.Setenv("TAVERN_API_KEY", "env-token")

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	token, err := auth.Authenticate(
		ctx,
		nil,
		srv.URL,
		auth.WithCacheFile(cachePath),
		auth.WithAPIKeyFromEnv("TAVERN_API_KEY"),
	)
	require.NoError(t, err)
	assert.Equal(t, "env-token", string(token))

	// No requests should have been sent to tavern since the env var
	// short-circuits the cache validation.
	assert.Equal(t, int32(0), reqCount.Load(), "should not validate cache when env token is present")

	// Cache file should remain intact.
	_, statErr := os.Stat(cachePath)
	assert.NoError(t, statErr, "cache file should be preserved when env var provides token")
}
