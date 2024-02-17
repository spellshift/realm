package auth_test

import (
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strconv"
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
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())
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
