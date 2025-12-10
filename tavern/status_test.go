package main

import (
	"io"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestStatusHandler(t *testing.T) {
	// Setup Dependencies
	handler := newStatusHandler()

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
