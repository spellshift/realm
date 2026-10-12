package cdn

import (
	"log/slog"
	"net/http"
	"strings"
	"time"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/link"
	"realm.pub/tavern/internal/errors"
)

// NewLinkDownloadHandler returns an HTTP handler responsible for downloading an Asset via a Link from the CDN.
// Assets are accessed using the link path. The Link's ExpiresAt and DownloadLimit determine if the Link can still be used for downloads.
// - If the Link cannot be found at the path provided, 404 is returned
// - If ExpiresAt <= now, 404 is returned
// - If DownloadLimit is set and Downloads >= DownloadLimit, 404 is returned
// - Otherwise, the Asset is returned
func NewLinkDownloadHandler(graph *ent.Client, prefix string) http.Handler {
	return errors.WrapHandler(func(w http.ResponseWriter, req *http.Request) error {
		ctx := req.Context()

		// Get the link path from the request URI
		linkPath := strings.TrimPrefix(req.URL.Path, prefix)
		if linkPath == "" || linkPath == "." || linkPath == "/" {
			return ErrFileNotFound
		}

		// Query for the link by path, including the associated asset
		l, err := graph.Link.Query().
			Where(link.Path(linkPath)).
			WithAsset().
			Only(ctx)
		if err != nil {
			slog.Error("failed to query link", "err", err, "path", linkPath)
			return ErrFileNotFound
		}

		// Ensure an asset is associated with the Link
		a := l.Edges.Asset
		if a == nil {
			return ErrFileNotFound
		}

		// Check Expiry
		if time.Now().After(l.ExpiresAt) {
			slog.Info("Failed attempt to download expired link", "path", linkPath)
			return ErrFileNotFound
		}

		// Reserve one download atomically. The conditional UPDATE
		// (SET downloads = downloads + 1 WHERE id = ? AND downloads < limit)
		// executes as a single statement, so concurrent requests cannot all
		// observe the same stale count and exceed the limit. Zero affected
		// rows means the limit was reached (possibly by a concurrent request).
		downloadLimit := -1
		if l.DownloadLimit != nil {
			downloadLimit = *l.DownloadLimit
		}
		if downloadLimit > 0 {
			n, err := graph.Link.Update().
				Where(link.ID(l.ID), link.DownloadsLT(downloadLimit)).
				AddDownloads(1).
				Save(ctx)
			if err != nil {
				slog.Error("failed to increment downloads for link", "path", linkPath, "err", err)
				return ErrFileNotFound
			}
			if n == 0 {
				slog.Info("Failed attempt to download link, maximum downloads reached", "path", linkPath, "download_limit", downloadLimit)
				return ErrFileNotFound
			}
		} else {
			// No limit enforced: best-effort atomic increment, still served on error.
			if _, err := graph.Link.UpdateOne(l).
				AddDownloads(1).
				Save(ctx); err != nil {
				slog.Error("failed to increment downloads for link", "path", linkPath, "err", err)
			}
		}

		// Set Etag to hash of asset
		w.Header().Set(HeaderEtag, a.Hash)

		// Set Content-Type and serve content in chunks
		w.Header().Set("Content-Type", "application/octet-stream")
		serveChunkedContent(w, a.Content, a.LastModifiedAt)

		return nil
	})
}
