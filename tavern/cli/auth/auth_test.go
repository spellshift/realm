package auth_test

import (
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"realm.pub/tavern/cli/auth"
	tavernauth "realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent/enttest"
	tavernhttp "realm.pub/tavern/internal/http"
)

func TestAuthenticate(t *testing.T) {
	// Setup Dependencies
	ctx := context.Background()
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())

	// Create Test User
	existingAdmin := graph.User.Create().
		SetName("test_user").
		SetOauthID("test_user").
		SetPhotoURL("http://google.com/").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	// Test Cases
	tests := []struct {
		name string

		sessionToken string

		wantAccessToken string
		wantError       error
	}{
		{
			name:            "AuthenticatedAdmin",
			sessionToken:    existingAdmin.SessionToken,
			wantAccessToken: existingAdmin.AccessToken,
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
				// Setup CLI Login Handler
				srv := tavernhttp.NewServer(tavernhttp.RouteMap{
					"/access_token/redirect": tavernhttp.Endpoint{
						Handler:          tavernauth.NewTokenRedirectHandler(),
						LoginRedirectURI: "/oauth/login",
					},
				}, tavernhttp.WithAuthentication(graph))

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
				redirURL := w.Result().Header.Get("Location")
				t.Logf("Redir URL: %s\n", redirURL)
				assert.NotEmpty(t, redirURL)
				if redirURL == "" {
					return fmt.Errorf("no redirect url was found")
				}

				redirReq, err := http.NewRequest(http.MethodGet, redirURL, nil)
				if err != nil {
					return err
				}
				_, err = http.DefaultClient.Do(redirReq)
				return err
			})

			// Authenticate
			token, err := auth.Authenticate(ctx, browser, "http://127.0.0.1")

			// Assertions
			assert.Equal(t, tc.wantAccessToken, string(token))
			assert.ErrorIs(t, err, tc.wantError)
		})
	}
}
