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
		limit := 5
		raceLink := graph.Link.Create().
			SetAsset(raceAsset).
			SetExpiresAt(time.Now().Add(1 * time.Hour)).
			SetDownloadLimit(limit).
			SaveX(ctx)

		const concurrency = 50
		var successCount atomic.Int64
		var rejectedCount atomic.Int64
		var unexpectedCount atomic.Int64
		var wg sync.WaitGroup
		// Release all goroutines at once (rather than staggering their
		// startup) to maximize contention on the read-then-write window the
		// atomic UPDATE is protecting.
		start := make(chan struct{})
		wg.Add(concurrency)
		for i := 0; i < concurrency; i++ {
			go func() {
				defer wg.Done()
				<-start
				req := httptest.NewRequest(http.MethodGet, fmt.Sprintf("/download/%s", raceLink.Path), nil)
				w := httptest.NewRecorder()
				handler.ServeHTTP(w, req)
				switch w.Result().StatusCode {
				case http.StatusOK:
					successCount.Add(1)
				case cdn.ErrFileNotFound.StatusCode:
					rejectedCount.Add(1)
				default:
					// Anything other than a clean 200 or the expected 404
					// (e.g. a 500 masked as a "rejection") must be surfaced
					// rather than silently counted as a correct rejection.
					unexpectedCount.Add(1)
				}
			}()
		}
		close(start)
		wg.Wait()

		assert.Equal(t, int64(0), unexpectedCount.Load(), "no request should return a status other than 200 or the expected 404")
		assert.Equal(t, int64(limit), successCount.Load(), "exactly DownloadLimit requests should succeed, regardless of concurrency")
		assert.Equal(t, int64(concurrency-limit), rejectedCount.Load(), "every request beyond DownloadLimit should be rejected with 404")

		reloaded := graph.Link.GetX(ctx, raceLink.ID)
		assert.Equal(t, limit, reloaded.Downloads, "Downloads counter must not exceed DownloadLimit")
	})

	t.Run("DownloadLimitSequentialAboveOne", func(t *testing.T) {
		// Sanity check that limits above 1 allow exactly that many downloads
		// before being rejected, i.e. the fix didn't collapse every limit to 1.
		seqAsset := newAsset(graph, "SequentialLimitAsset", []byte("sequential_content"))
		limit := 3
		seqLink := graph.Link.Create().
			SetAsset(seqAsset).
			SetExpiresAt(time.Now().Add(1 * time.Hour)).
			SetDownloadLimit(limit).
			SaveX(ctx)

		for i := 0; i < limit; i++ {
			req := httptest.NewRequest(http.MethodGet, fmt.Sprintf("/download/%s", seqLink.Path), nil)
			w := httptest.NewRecorder()
			handler.ServeHTTP(w, req)
			require.Equal(t, http.StatusOK, w.Result().StatusCode, "download %d of %d should succeed", i+1, limit)
		}

		// The next download beyond the limit must be rejected.
		req := httptest.NewRequest(http.MethodGet, fmt.Sprintf("/download/%s", seqLink.Path), nil)
		w := httptest.NewRecorder()
		handler.ServeHTTP(w, req)
		assert.Equal(t, cdn.ErrFileNotFound.StatusCode, w.Result().StatusCode)

		reloaded := graph.Link.GetX(ctx, seqLink.ID)
		assert.Equal(t, limit, reloaded.Downloads)
	})

	t.Run("UnlimitedDownloadsIncrementCounter", func(t *testing.T) {
		// A link with no DownloadLimit set is never rejected, but the
		// Downloads counter should still be incremented atomically on every
		// successful download (AddDownloads, not the old stale SetDownloads).
		unlimitedAsset := newAsset(graph, "UnlimitedAsset", []byte("unlimited_content"))
		unlimitedLink := graph.Link.Create().
			SetAsset(unlimitedAsset).
			SetExpiresAt(time.Now().Add(1 * time.Hour)).
			SaveX(ctx)

		const requests = 5
		for i := 0; i < requests; i++ {
			req := httptest.NewRequest(http.MethodGet, fmt.Sprintf("/download/%s", unlimitedLink.Path), nil)
			w := httptest.NewRecorder()
			handler.ServeHTTP(w, req)
			require.Equal(t, http.StatusOK, w.Result().StatusCode)
		}

		reloaded := graph.Link.GetX(ctx, unlimitedLink.ID)
		assert.Equal(t, requests, reloaded.Downloads)
	})
}
