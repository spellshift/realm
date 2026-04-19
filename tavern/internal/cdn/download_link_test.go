package cdn_test

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/crypto/sha3"
	"realm.pub/tavern/internal/cdn"
	"realm.pub/tavern/internal/ent/enttest"
)

// TestDownloadLink asserts that the link download handler exhibits expected behavior.
func TestDownloadLink(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	ctx := context.Background()

	expectedContent := []byte("link_file_content")
	existingAsset := newAsset(graph, "LinkTestAsset", expectedContent)
	downloadLimit := 10

	existingLink := graph.Link.Create().
		SetAsset(existingAsset).
		SetExpiresAt(time.Now().Add(1 * time.Hour)).
		SetDownloadLimit(downloadLimit).
		SaveX(ctx)

	expiredLink := graph.Link.Create().
		SetAsset(existingAsset).
		SetExpiresAt(time.Now().Add(-1 * time.Hour)).
		SaveX(ctx)

	handler := cdn.NewLinkDownloadHandler(graph, "/download/")

	tests := []struct {
		name       string
		reqURL     string
		wantStatus int
		wantBody   []byte
	}{
		{
			name:       "Valid",
			reqURL:     fmt.Sprintf("/download/%s", existingLink.Path),
			wantStatus: http.StatusOK,
			wantBody:   expectedContent,
		},
		{
			name:       "NotFound",
			reqURL:     "/download/nonexistent-path",
			wantStatus: cdn.ErrFileNotFound.StatusCode,
			wantBody:   []byte(fmt.Sprintf("%s\n", cdn.ErrFileNotFound.Error())),
		},
		{
			name:       "Expired",
			reqURL:     fmt.Sprintf("/download/%s", expiredLink.Path),
			wantStatus: cdn.ErrFileNotFound.StatusCode,
			wantBody:   []byte(fmt.Sprintf("%s\n", cdn.ErrFileNotFound.Error())),
		},
		{
			name:       "EmptyPath",
			reqURL:     "/download/",
			wantStatus: cdn.ErrFileNotFound.StatusCode,
			wantBody:   []byte(fmt.Sprintf("%s\n", cdn.ErrFileNotFound.Error())),
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, tc.reqURL, nil)
			w := httptest.NewRecorder()
			handler.ServeHTTP(w, req)

			result := w.Result()
			assert.Equal(t, tc.wantStatus, result.StatusCode)

			body, err := io.ReadAll(result.Body)
			require.NoError(t, err)
			defer result.Body.Close()

			if tc.wantStatus == http.StatusOK {
				assert.Equal(t, tc.wantBody, body)
				hash := fmt.Sprintf("%x", sha3.Sum256(body))
				assert.Equal(t, hash, result.Header.Get(cdn.HeaderEtag))
			} else {
				assert.Equal(t, tc.wantBody, body)
			}
		})
	}

	t.Run("ChunkedDownload", func(t *testing.T) {
		// Set a small chunk size for testing
		origMaxChunkSize := cdn.MaxChunkSize
		cdn.MaxChunkSize = 4
		defer func() { cdn.MaxChunkSize = origMaxChunkSize }()

		largeContent := []byte("abcdefghijklmnop") // 16 bytes, 4 chunks of 4 bytes
		largeAsset := newAsset(graph, "LargeLinkAsset", largeContent)
		chunkedLink := graph.Link.Create().
			SetAsset(largeAsset).
			SetExpiresAt(time.Now().Add(1 * time.Hour)).
			SaveX(ctx)

		req := httptest.NewRequest(http.MethodGet, fmt.Sprintf("/download/%s", chunkedLink.Path), nil)
		w := httptest.NewRecorder()
		handler.ServeHTTP(w, req)

		result := w.Result()
		assert.Equal(t, http.StatusOK, result.StatusCode)

		body, err := io.ReadAll(result.Body)
		require.NoError(t, err)
		defer result.Body.Close()

		assert.Equal(t, largeContent, body)
	})
}
