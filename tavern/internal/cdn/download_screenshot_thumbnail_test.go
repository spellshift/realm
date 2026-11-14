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

// TestDownloadScreenshotThumbnail asserts that the screenshot thumbnail download handler
// exhibits expected behavior, including falling back to full resolution content when no
// thumbnail has been generated.
func TestDownloadScreenshotThumbnail(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	ctx := context.Background()

	existingHost := graph.Host.Create().
		SetIdentifier("screenshot-thumbnail-test-host").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)

	expectedContent := []byte("fake_png_content")
	expectedThumbnail := []byte("fake_thumbnail_content")
	existingScreenshot := graph.Screenshot.Create().
		SetName("test-screenshot.png").
		SetContent(expectedContent).
		SetThumbnail(expectedThumbnail).
		SetHost(existingHost).
		SaveX(ctx)

	screenshotWithoutThumbnail := graph.Screenshot.Create().
		SetName("no-thumbnail.png").
		SetContent(expectedContent).
		SetHost(existingHost).
		SaveX(ctx)

	handler := cdn.NewScreenshotThumbnailDownloadHandler(graph, "/download/")

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
			wantBody:   expectedThumbnail,
		},
		{
			name:       "FallsBackToContentWhenNoThumbnail",
			reqURL:     fmt.Sprintf("/download/%d", screenshotWithoutThumbnail.ID),
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
			name:       "Cached",
			reqURL:     fmt.Sprintf("/download/%d", existingScreenshot.ID),
			wantStatus: cdn.ErrFileNotModified.StatusCode,
			wantBody:   []byte(fmt.Sprintf("%s\n", cdn.ErrFileNotModified.Error())),
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, tc.reqURL, nil)

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
}
