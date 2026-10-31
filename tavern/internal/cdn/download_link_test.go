package cdn_test

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"sync"
	"sync/atomic"
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

	t.Run("DownloadLimitReached", func(t *testing.T) {
		limitedAsset := newAsset(graph, "LimitReachedAsset", []byte("limited_content"))
		limit := 1
		limitedLink := graph.Link.Create().
			SetAsset(limitedAsset).
			SetExpiresAt(time.Now().Add(1 * time.Hour)).
			SetDownloadLimit(limit).
			SaveX(ctx)

		// First download succeeds and consumes the only allowed download.
		req := httptest.NewRequest(http.MethodGet, fmt.Sprintf("/download/%s", limitedLink.Path), nil)
		w := httptest.NewRecorder()
		handler.ServeHTTP(w, req)
		assert.Equal(t, http.StatusOK, w.Result().StatusCode)

		// Second download must be rejected since the limit has been reached.
		req = httptest.NewRequest(http.MethodGet, fmt.Sprintf("/download/%s", limitedLink.Path), nil)
		w = httptest.NewRecorder()
		handler.ServeHTTP(w, req)
		result := w.Result()
		assert.Equal(t, cdn.ErrFileNotFound.StatusCode, result.StatusCode)

		body, err := io.ReadAll(result.Body)
		require.NoError(t, err)
		defer result.Body.Close()
		assert.Equal(t, []byte(fmt.Sprintf("%s\n", cdn.ErrFileNotFound.Error())), body)

		reloaded := graph.Link.GetX(ctx, limitedLink.ID)
		assert.Equal(t, limit, reloaded.Downloads)
	})

	t.Run("DownloadLimitZero", func(t *testing.T) {
		// A link created with DownloadLimit:0 must never serve the asset.
		zeroLimitAsset := newAsset(graph, "ZeroLimitAsset", []byte("zero_limit_content"))
		zeroLimit := 0
		zeroLimitLink := graph.Link.Create().
			SetAsset(zeroLimitAsset).
			SetExpiresAt(time.Now().Add(1 * time.Hour)).
			SetDownloadLimit(zeroLimit).
			SaveX(ctx)

		req := httptest.NewRequest(http.MethodGet, fmt.Sprintf("/download/%s", zeroLimitLink.Path), nil)
		w := httptest.NewRecorder()
		handler.ServeHTTP(w, req)
		result := w.Result()
		assert.Equal(t, cdn.ErrFileNotFound.StatusCode, result.StatusCode)

		reloaded := graph.Link.GetX(ctx, zeroLimitLink.ID)
		assert.Equal(t, 0, reloaded.Downloads)
	})

	t.Run("DownloadLimitConcurrentRace", func(t *testing.T) {
		// Regression test for #2410: concurrent requests must not be able to
		// exceed DownloadLimit via a check-then-increment race.
		raceAsset := newAsset(graph, "RaceAsset", []byte("race_content"))
		limit := 1
		raceLink := graph.Link.Create().
			SetAsset(raceAsset).
			SetExpiresAt(time.Now().Add(1 * time.Hour)).
			SetDownloadLimit(limit).
			SaveX(ctx)

		const concurrency = 20
		var successCount atomic.Int64
		var wg sync.WaitGroup
		wg.Add(concurrency)
		for i := 0; i < concurrency; i++ {
			go func() {
				defer wg.Done()
				req := httptest.NewRequest(http.MethodGet, fmt.Sprintf("/download/%s", raceLink.Path), nil)
				w := httptest.NewRecorder()
				handler.ServeHTTP(w, req)
				if w.Result().StatusCode == http.StatusOK {
					successCount.Add(1)
				}
			}()
		}
		wg.Wait()

		assert.Equal(t, int64(limit), successCount.Load(), "exactly DownloadLimit requests should succeed, regardless of concurrency")

		reloaded := graph.Link.GetX(ctx, raceLink.ID)
		assert.Equal(t, limit, reloaded.Downloads, "Downloads counter must not exceed DownloadLimit")
	})
}
