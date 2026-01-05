package main

import (
	"io"
	"net/http"
	"net/http/httptest"
	"path/filepath"
	"testing"
	"realm.pub/tavern/internal/keyservice"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestStatusHandler(t *testing.T) {
	// Setup Dependencies
	secretsDir := t.TempDir()
	secretsPath := filepath.Join(secretsDir, "secrets.yaml")
	t.Setenv("SECRETS_FILE_PATH", secretsPath)

	keySvc, err := keyservice.NewKeyService()
	require.NoError(t, err)
	handler := newStatusHandler(keySvc)

	// Test Cases
	tests := []struct {
		name string
		w    *httptest.ResponseRecorder
		r    *http.Request

		wantCode int
		wantBody string
	}{
		{
			name: "Successful",
			w:    httptest.NewRecorder(),
			r:    httptest.NewRequest(http.MethodGet, "/status", nil),

			wantCode: http.StatusOK,
			wantBody: OKStatusText,
		},
	}

	// Run Tests
	for _, tc := range tests {
		handler(tc.w, tc.r)

		body, err := io.ReadAll(tc.w.Body)
		require.NoError(t, err)

		assert.Equal(t, tc.wantCode, tc.w.Code)
		assert.Contains(t, string(body), tc.wantBody)
	}
}
