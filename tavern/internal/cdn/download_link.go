package cdn

import (
	"log/slog"
	"net/http"
	"strings"
	"time"

	entsql "entgo.io/ent/dialect/sql"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/link"
	"realm.pub/tavern/internal/ent/predicate"
	"realm.pub/tavern/internal/errors"
)

// NewLinkDownloadHandler returns an HTTP handler responsible for downloading an Asset via a Link from the CDN.
// Assets are accessed using the link path. The Link's ExpiresAt and DownloadLimit determine if the Link can still be used for downloads.
// - If the Link cannot be found at the path provided, 404 is returned
// - If ExpiresAt <= now, 404 is returned
// - If DownloadLimit is set and Downloads >= DownloadLimit, 404 is returned (a DownloadLimit of 0 means the link can never be downloaded)
// - If DownloadLimit is unset (nil), downloads are unlimited
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

		// Reserve a download slot atomically. When a DownloadLimit is set, the
		// conditional UPDATE (SET downloads = downloads + 1 WHERE id = ? AND
		// downloads < download_limit) executes as a single statement, so
		// concurrent requests cannot all observe the same stale count and
		// exceed the limit. The limit is compared against the live
		// download_limit column (not the value read above) so a concurrent
		// change to the limit itself can't be raced either. Zero affected
		// rows means the limit has been reached (possibly by a concurrent
		// request racing this one, or because the limit was lowered).
		if l.DownloadLimit != nil {
			downloadLimit := *l.DownloadLimit

			// Fast-path: avoid an unnecessary write against already-exhausted
			// links (including links created with DownloadLimit: 0).
			if l.Downloads >= downloadLimit {
				slog.Info("Failed attempt to download link, maximum downloads reached", "path", linkPath, "downloads", l.Downloads, "download_limit", downloadLimit)
				return ErrFileNotFound
			}

			affected, err := graph.Link.Update().
				Where(
					link.ID(l.ID),
					predicate.Link(func(s *entsql.Selector) {
						s.Where(entsql.ColumnsLT(link.FieldDownloads, link.FieldDownloadLimit))
					}),
				).
				AddDownloads(1).
				Save(ctx)
			if err != nil {
				slog.Error("failed to increment downloads for link", "path", linkPath, "downloads", l.Downloads, "download_limit", downloadLimit, "err", err)
				return ErrFileNotFound
			}
			if affected == 0 {
				slog.Info("Failed attempt to download link, maximum downloads reached", "path", linkPath, "downloads", l.Downloads, "download_limit", downloadLimit)
				return ErrFileNotFound
			}
		} else {
			// No limit enforced: best-effort atomic increment, still served on error.
			if _, err := graph.Link.UpdateOne(l).
				AddDownloads(1).
				Save(ctx); err != nil {
				slog.Error("failed to increment downloads for link", "path", linkPath, "downloads", l.Downloads, "err", err)
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
