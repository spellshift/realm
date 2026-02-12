package cdn

import (
	"bytes"
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

		// Check DownloadLimit
		downloadLimit := -1
		if l.DownloadLimit != nil {
			downloadLimit = *l.DownloadLimit
		}
		if downloadLimit > 0 && l.Downloads >= downloadLimit {
			slog.Info("Failed attempt to download link, maximum downloads reached", "path", linkPath, "downloads", l.Downloads, "download_limit", l.DownloadLimit)
			return ErrFileNotFound
		}

		// Increment Link Downloads
		if _, err := graph.Link.UpdateOne(l).
			SetDownloads(l.Downloads + 1).
			Save(ctx); err != nil {
			slog.Error("failed to increment downloads for link", "path", linkPath, "downloads", l.Downloads, "err", err)

			// Only error if a download limit is enforced
			if downloadLimit > 0 {
				return ErrFileNotFound
			}
		}

		// Set Etag to hash of asset
		w.Header().Set(HeaderEtag, a.Hash)

		// Set Content-Type and serve content
		w.Header().Set("Content-Type", "application/octet-stream")
		http.ServeContent(w, req, a.Name, a.LastModifiedAt, bytes.NewReader(a.Content))

		return nil
	})
}
