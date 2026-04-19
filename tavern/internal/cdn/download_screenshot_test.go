package cdn_test

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/cdn"
	"realm.pub/tavern/internal/ent/enttest"
)

// TestDownloadScreenshot asserts that the screenshot download handler exhibits expected behavior.
func TestDownloadScreenshot(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	ctx := context.Background()

	existingHost := graph.Host.Create().
		SetIdentifier("screenshot-test-host").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)

	expectedContent := []byte("fake_png_content")
	existingScreenshot := graph.Screenshot.Create().
		SetName("test-screenshot.png").
		SetContent(expectedContent).
		SetHost(existingHost).
		SaveX(ctx)

	handler := cdn.NewScreenshotDownloadHandler(graph, "/download/")

	tests := []struct {
		name       string
		reqURL     string
		wantStatus int
		wantBody   []byte
	}{
		{
			name:       "Valid",
			reqURL:     fmt.Sprintf("/download/%d", existingScreenshot.ID),
			wantStatus: http.StatusOK,
			wantBody:   expectedContent,
		},
		{
			name:       "NotFound",
			reqURL:     "/download/99999",
			wantStatus: cdn.ErrFileNotFound.StatusCode,
			wantBody:   []byte(fmt.Sprintf("%s\n", cdn.ErrFileNotFound.Error())),
		},
		{
			name:       "InvalidID/Alphabet",
			reqURL:     "/download/abcd",
			wantStatus: cdn.ErrInvalidFileID.StatusCode,
			wantBody:   []byte(fmt.Sprintf("%s\n", cdn.ErrInvalidFileID.Error())),
		},
		{
			name:       "InvalidID/Empty",
			reqURL:     "/download/",
			wantStatus: cdn.ErrInvalidFileID.StatusCode,
			wantBody:   []byte(fmt.Sprintf("%s\n", cdn.ErrInvalidFileID.Error())),
		},
		{
			name:   "Cached",
			reqURL: fmt.Sprintf("/download/%d", existingScreenshot.ID),
			wantStatus: cdn.ErrFileNotModified.StatusCode,
			wantBody:   []byte(fmt.Sprintf("%s\n", cdn.ErrFileNotModified.Error())),
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, tc.reqURL, nil)

			// For the Cached test, set the If-None-Match header
			if tc.name == "Cached" {
				req.Header.Set(cdn.HeaderIfNoneMatch, existingScreenshot.Hash)
			}

			w := httptest.NewRecorder()
			handler.ServeHTTP(w, req)

			result := w.Result()
			assert.Equal(t, tc.wantStatus, result.StatusCode)

			body, err := io.ReadAll(result.Body)
			require.NoError(t, err)
			defer result.Body.Close()

			assert.Equal(t, tc.wantBody, body)
		})
	}

	t.Run("ChunkedDownload", func(t *testing.T) {
		// Set a small chunk size for testing
		origMaxChunkSize := cdn.MaxChunkSize
		cdn.MaxChunkSize = 4
		defer func() { cdn.MaxChunkSize = origMaxChunkSize }()

		largeContent := []byte("abcdefghijklmnop") // 16 bytes, 4 chunks of 4 bytes
		largeScreenshot := graph.Screenshot.Create().
			SetName("large-screenshot.png").
			SetContent(largeContent).
			SetHost(existingHost).
			SaveX(ctx)

		req := httptest.NewRequest(http.MethodGet, fmt.Sprintf("/download/%d", largeScreenshot.ID), nil)
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
