package auth_test

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
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

	_, err := auth.Authenticate(ctx, nil, srv.URL, auth.WithCacheFile(cachePath))
	// The remote device auth flow cannot complete in this test, so we
	// expect Authenticate to surface a deadline-related error. We only
	// care that the cache file was cleaned up before that happened.
	assert.Error(t, err, "expected Authenticate to return an error once RDA times out")

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

// TestAuthenticate_DefaultRDA verifies the default Remote Device Authentication
// flow including trailing-slash URL normalization, plaintext HTTP warning,
// polling until approved, and credential caching without opening a browser.
func TestAuthenticate_DefaultRDA(t *testing.T) {
	t.Setenv(auth.EnvUseBrowserOAuth, "")

	const (
		expectedDeviceCode = "dev-code-123"
		expectedToken      = "rda-access-token"
	)

	var pollCount atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch r.URL.Path {
		case "/auth/rda/code":
			assert.Equal(t, http.MethodPost, r.Method)
			w.Header().Set("Content-Type", "application/json")
			_ = json.NewEncoder(w).Encode(map[string]any{
				"device_code":               expectedDeviceCode,
				"user_code":                 "ABCD-EFGH",
				"verification_uri":          "https://tavern.example/device",
				"verification_uri_complete": "https://tavern.example/device?code=ABCD-EFGH",
				"expires_in":                10,
				"interval":                  1,
			})
		case "/auth/rda/token":
			assert.Equal(t, http.MethodGet, r.Method)
			assert.Equal(t, expectedDeviceCode, r.URL.Query().Get("device_code"))
			if pollCount.Add(1) == 1 {
				http.Error(w, `{"error":"authorization_pending"}`, http.StatusBadRequest)
				return
			}
			w.Header().Set("Content-Type", "application/json")
			_ = json.NewEncoder(w).Encode(map[string]string{
				"access_token": expectedToken,
			})
		default:
			http.Error(w, "unexpected path: "+r.URL.Path, http.StatusNotFound)
		}
	}))
	defer srv.Close()

	// Capture slog warnings to verify plaintext http:// warning fires in RDA flow.
	var logBuf bytes.Buffer
	prevLogger := slog.Default()
	slog.SetDefault(slog.New(slog.NewTextHandler(&logBuf, &slog.HandlerOptions{Level: slog.LevelWarn})))
	defer slog.SetDefault(prevLogger)

	browser := auth.BrowserFunc(func(string) error {
		t.Fatalf("browser should not be opened in default RDA flow")
		return nil
	})

	cachePath := filepath.Join(t.TempDir(), "cache")
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	// Pass URL with trailing slash to verify normalization.
	token, err := auth.Authenticate(ctx, browser, srv.URL+"/", auth.WithCacheFile(cachePath))
	require.NoError(t, err)
	assert.Equal(t, expectedToken, string(token))
	assert.GreaterOrEqual(t, pollCount.Load(), int32(2))
	assert.Contains(t, logBuf.String(), "using insecure access token url (http)")

	cached, err := os.ReadFile(cachePath)
	require.NoError(t, err)
	assert.Equal(t, expectedToken, string(cached))
}

// TestAuthenticate_DefaultRDA_InvalidURL verifies that invalid URLs fail fast
// with ErrInvalidURL in the default RDA flow (without TAVERN_USE_BROWSER_OAUTH).
func TestAuthenticate_DefaultRDA_InvalidURL(t *testing.T) {
	t.Setenv(auth.EnvUseBrowserOAuth, "")

	for _, rawURL := range []string{"💩://invalid", "", "not-a-url"} {
		t.Run(rawURL, func(t *testing.T) {
			ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
			defer cancel()

			token, err := auth.Authenticate(ctx, nil, rawURL)
			assert.Empty(t, string(token))
			assert.ErrorIs(t, err, auth.ErrInvalidURL)
		})
	}
}

// TestAuthenticate_BrowserOAuthOptIn verifies that browser OAuth can be enabled
// both programmatically via WithBrowserOAuth() and via boolean env var values.
func TestAuthenticate_BrowserOAuthOptIn(t *testing.T) {
	const wantToken = "browser-opt-in-token"

	tests := []struct {
		name   string
		envVal string
		opts   []auth.AuthOption
	}{
		{
			name:   "WithBrowserOAuthOption",
			envVal: "",
			opts:   []auth.AuthOption{auth.WithBrowserOAuth()},
		},
		{
			name:   "EnvTrue",
			envVal: "true",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			t.Setenv(auth.EnvUseBrowserOAuth, tc.envVal)

			ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			defer cancel()

			browserCalled := false
			browser := auth.BrowserFunc(func(loginURL string) error {
				browserCalled = true
				u, err := url.Parse(loginURL)
				require.NoError(t, err)
				assert.Equal(t, "/access_token/redirect", u.Path)
				redirPort := u.Query().Get(tavernauth.ParamTokenRedirPort)
				require.NotEmpty(t, redirPort)

				callbackURL := fmt.Sprintf("http://127.0.0.1:%s/?%s=%s", redirPort, tavernauth.ParamTokenRedirToken, wantToken)
				resp, err := http.Get(callbackURL)
				if err != nil {
					return err
				}
				resp.Body.Close()
				return nil
			})

			token, err := auth.Authenticate(ctx, browser, "https://tavern.example/", tc.opts...)
			require.NoError(t, err)
			assert.True(t, browserCalled)
			assert.Equal(t, wantToken, string(token))
		})
	}
}
