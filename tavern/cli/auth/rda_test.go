package auth_test

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/cli/auth"
)

func TestAuthenticateRemoteDevice_PostsBodyAndStopsOnTerminalStatuses(t *testing.T) {
	tests := []struct {
		name           string
		tokenStatus    int
		tokenBody      string
		wantErrContain string
	}{
		{
			name:           "denied 403 stops immediately",
			tokenStatus:    http.StatusForbidden,
			tokenBody:      `{"status":"DENIED","error":"access_denied"}`,
			wantErrContain: "access_denied",
		},
		{
			name:           "expired 410 stops immediately",
			tokenStatus:    http.StatusGone,
			tokenBody:      `{"status":"EXPIRED","error":"expired_token"}`,
			wantErrContain: "expired_token",
		},
		{
			name:           "legacy 200 DENIED stops immediately",
			tokenStatus:    http.StatusOK,
			tokenBody:      `{"status":"DENIED"}`,
			wantErrContain: "access_denied",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			var pollCount atomic.Int32
			var sawQueryDeviceCode atomic.Bool
			var sawPostBodyDeviceCode atomic.Bool

			srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				switch r.URL.Path {
				case "/auth/rda/code":
					w.Header().Set("Content-Type", "application/json")
					_, _ = w.Write([]byte(`{"device_code":"secret-dev-code","user_code":"abcd1234","verification_uri":"http://localhost/profile","verification_uri_complete":"http://localhost/profile?device-code=abcd1234","expires_in":60,"interval":1}`))
				case "/auth/rda/token":
					pollCount.Add(1)
					if r.URL.Query().Get("device_code") != "" {
						sawQueryDeviceCode.Store(true)
					}
					if r.Method == http.MethodPost {
						var req struct {
							DeviceCode string `json:"device_code"`
						}
						if json.NewDecoder(r.Body).Decode(&req) == nil && req.DeviceCode == "secret-dev-code" {
							sawPostBodyDeviceCode.Store(true)
						}
					}
					w.Header().Set("Content-Type", "application/json")
					w.WriteHeader(tc.tokenStatus)
					_, _ = w.Write([]byte(tc.tokenBody))
				default:
					http.NotFound(w, r)
				}
			}))
			defer srv.Close()

			ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			defer cancel()

			tokenCh := make(chan auth.Token, 1)
			errCh := make(chan error, 1)
			go auth.AuthenticateRemoteDevice(ctx, srv.URL, tokenCh, errCh)

			select {
			case tok := <-tokenCh:
				t.Fatalf("unexpected token %q", string(tok))
			case err := <-errCh:
				require.Error(t, err)
				assert.Contains(t, err.Error(), tc.wantErrContain)
			case <-ctx.Done():
				t.Fatalf("timed out waiting for terminal poll error")
			}

			assert.Equal(t, int32(1), pollCount.Load(), "should stop after first terminal response")
			assert.False(t, sawQueryDeviceCode.Load(), "device_code must not be sent in URL query string")
			assert.True(t, sawPostBodyDeviceCode.Load(), "device_code must be sent in POST JSON body")
		})
	}
}

func TestAuthenticate_CacheWriteHardening(t *testing.T) {
	const issuedToken = "fresh-rda-token"

	t.Run("replaces pre-existing loose perms file with 0600", func(t *testing.T) {
		dir := t.TempDir()
		cachePath := filepath.Join(dir, ".tavern-auth")

		recreateSrv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			switch r.URL.Path {
			case "/auth/rda/code":
				w.Header().Set("Content-Type", "application/json")
				_, _ = w.Write([]byte(`{"device_code":"dev-cache-code","user_code":"11223344","verification_uri":"http://localhost/profile","verification_uri_complete":"http://localhost/profile?device-code=11223344","expires_in":60,"interval":1}`))
			case "/auth/rda/token":
				// Plant a pre-existing 0644 file right before Authenticate writes the new token
				_ = os.WriteFile(cachePath, []byte("old"), 0644)
				_ = os.Chmod(cachePath, 0644)
				w.Header().Set("Content-Type", "application/json")
				_, _ = w.Write([]byte(`{"status":"APPROVED","access_token":"` + issuedToken + `"}`))
			default:
				http.Error(w, "unauthorized", http.StatusUnauthorized)
			}
		}))
		defer recreateSrv.Close()

		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()

		tok, err := auth.Authenticate(ctx, nil, recreateSrv.URL, auth.WithCacheFile(cachePath))
		require.NoError(t, err)
		assert.Equal(t, issuedToken, string(tok))

		info, err := os.Lstat(cachePath)
		require.NoError(t, err)
		assert.Equal(t, os.FileMode(0600), info.Mode().Perm())

		data, err := os.ReadFile(cachePath)
		require.NoError(t, err)
		assert.Equal(t, issuedToken, string(data))
	})

	t.Run("refuses to write credentials through a symlink", func(t *testing.T) {
		dir := t.TempDir()
		targetPath := filepath.Join(dir, "victim-file")
		require.NoError(t, os.WriteFile(targetPath, []byte("original-content"), 0644))
		cachePath := filepath.Join(dir, ".tavern-auth")

		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			switch r.URL.Path {
			case "/auth/rda/code":
				w.Header().Set("Content-Type", "application/json")
				_, _ = w.Write([]byte(`{"device_code":"dev-symlink","user_code":"55667788","verification_uri":"http://localhost/profile","verification_uri_complete":"http://localhost/profile?device-code=55667788","expires_in":60,"interval":1}`))
			case "/auth/rda/token":
				// Plant a symlink at cachePath right before token write
				_ = os.Symlink(targetPath, cachePath)
				w.Header().Set("Content-Type", "application/json")
				_, _ = w.Write([]byte(`{"status":"APPROVED","access_token":"` + issuedToken + `"}`))
			default:
				http.Error(w, "unauthorized", http.StatusUnauthorized)
			}
		}))
		defer srv.Close()

		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()

		tok, err := auth.Authenticate(ctx, nil, srv.URL, auth.WithCacheFile(cachePath))
		require.NoError(t, err)
		assert.Equal(t, issuedToken, string(tok))

		// Victim target must remain completely untouched
		victimData, err := os.ReadFile(targetPath)
		require.NoError(t, err)
		assert.Equal(t, "original-content", string(victimData))
	})
}
