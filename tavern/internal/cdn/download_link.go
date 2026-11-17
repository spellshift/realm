package cdn

import (
	"net/http"
	"strings"
	"time"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
	"realm.pub/tavern/internal/ent/link"
	"realm.pub/tavern/internal/errors"
	"realm.pub/tavern/internal/logging"
)

// NewLinkDownloadHandler returns an HTTP handler responsible for downloading an Asset via a Link from the CDN.
// Assets are accessed using the link path. The Link's ExpiresAt and DownloadLimit determine if the Link can still be used for downloads.
// - If the Link cannot be found at the path provided, 404 is returned
// - If ExpiresAt <= now, 404 is returned
// - If DownloadLimit is set and Downloads >= DownloadLimit, 404 is returned
// - Otherwise, the Asset is returned
func NewLinkDownloadHandler(graph *ent.Client, prefix string) http.Handler {
	return errors.WrapHandler(func(w http.ResponseWriter, req *http.Request) error {
		ctx := cdnAuditCtx(req)

		// Get the link path from the request URI
		linkPath := strings.TrimPrefix(req.URL.Path, prefix)
		if linkPath == "" || linkPath == "." || linkPath == "/" {
			logging.AuditDenied(ctx, "cdn", "cdn.link_denied", "CDN link download denied: empty link path",
				"link_path", linkPath,
				"reason", "invalid_path",
			)
			return ErrFileNotFound
		}

		// Query for the link by path, including the associated asset
		l, err := graph.Link.Query().
			Where(link.Path(linkPath)).
			WithAsset().
			Only(ctx)
		if err != nil {
			reason := "not_found"
			if ent.IsNotFound(err) {
				linkLog.DebugContext(ctx, "link not found", "link_path", linkPath)
			} else {
				reason = "query_error"
				linkLog.ErrorContext(ctx, "failed to query link", "link_path", linkPath, "error", err)
			}
			deniedAttrs := append(logging.TargetAttrs("link", linkPath, linkPath),
				"link_path", linkPath,
				"reason", reason,
			)
			logging.AuditDenied(ctx, "cdn", "cdn.link_denied", "CDN link download denied: link not found", deniedAttrs...)
			return ErrFileNotFound
		}

		// Ensure an asset is associated with the Link
		a := l.Edges.Asset
		if a == nil {
			linkLog.WarnContext(ctx, "link has no associated asset", "link_id", l.ID, "link_path", linkPath)
			logging.AuditDenied(ctx, "cdn", "cdn.link_denied", "CDN link download denied: missing asset",
				"link_id", l.ID,
				"link_path", linkPath,
				"reason", "missing_asset",
			)
			return ErrFileNotFound
		}
		if a.Status != asset.StatusFINISHED {
			return ErrFileNotFound
		}

		// Check Expiry
		if time.Now().After(l.ExpiresAt) {
			linkLog.DebugContext(ctx, "rejected download for expired link", "link_id", l.ID, "link_path", linkPath)
			logging.AuditDenied(ctx, "cdn", "cdn.link_denied", "CDN link download denied: link expired",
				"link_id", l.ID,
				"link_path", linkPath,
				"asset_id", a.ID,
				"asset_name", a.Name,
				"reason", "expired",
			)
			return ErrFileNotFound
		}

		// Check DownloadLimit
		downloadLimit := -1
		if l.DownloadLimit != nil {
			downloadLimit = *l.DownloadLimit
		}
		if downloadLimit > 0 && l.Downloads >= downloadLimit {
			linkLog.DebugContext(ctx, "rejected download for exhausted link", "link_id", l.ID, "link_path", linkPath, "downloads", l.Downloads, "download_limit", downloadLimit)
			logging.AuditDenied(ctx, "cdn", "cdn.link_denied", "CDN link download denied: download limit reached",
				"link_id", l.ID,
				"link_path", linkPath,
				"asset_id", a.ID,
				"asset_name", a.Name,
				"downloads", l.Downloads,
				"download_limit", downloadLimit,
				"reason", "limit_reached",
			)
			return ErrFileNotFound
		}

		// Increment Link Downloads atomically, enforcing the limit in the UPDATE itself
		update := graph.Link.Update().Where(link.ID(l.ID)).AddDownloads(1)
		if downloadLimit > 0 {
			update = update.Where(link.DownloadsLT(downloadLimit))
		}
		affected, err := update.Save(ctx)
		if err != nil {
			linkLog.ErrorContext(ctx, "failed to increment downloads for link", "link_id", l.ID, "link_path", linkPath, "downloads", l.Downloads, "error", err)

			// Only error if a download limit is enforced
			if downloadLimit > 0 {
				logging.AuditDenied(ctx, "cdn", "cdn.link_denied", "CDN link download denied: failed to enforce download limit",
					"link_id", l.ID,
					"link_path", linkPath,
					"asset_id", a.ID,
					"asset_name", a.Name,
					"download_limit", downloadLimit,
					"reason", "limit_update_error",
				)
				return ErrFileNotFound
			}
		}
		if downloadLimit > 0 && affected == 0 {
			linkLog.DebugContext(ctx, "rejected download for exhausted link", "link_id", l.ID, "link_path", linkPath, "download_limit", downloadLimit)
			logging.AuditDenied(ctx, "cdn", "cdn.link_denied", "CDN link download denied: download limit reached",
				"link_id", l.ID,
				"link_path", linkPath,
				"asset_id", a.ID,
				"asset_name", a.Name,
				"download_limit", downloadLimit,
				"reason", "limit_reached",
			)
			return ErrFileNotFound
		}

		var downloadsRemaining any
		if downloadLimit > 0 {
			rem := downloadLimit - (l.Downloads + 1)
			if updated, gErr := graph.Link.Get(ctx, l.ID); gErr == nil {
				rem = downloadLimit - updated.Downloads
			}
			if rem < 0 {
				rem = 0
			}
			downloadsRemaining = rem
		}

		// Set Etag to hash of asset
		w.Header().Set(HeaderEtag, a.Hash)

		// Set Content-Type and serve content in chunks
		w.Header().Set("Content-Type", "application/octet-stream")
		serveChunkedContent(w, a.Content, a.LastModifiedAt)

		logging.Audit(ctx, "cdn", "cdn.link_downloaded", "payload downloaded via CDN link",
			"link_id", l.ID,
			"link_path", l.Path,
			"asset_id", a.ID,
			"asset_name", a.Name,
			"downloads_remaining", downloadsRemaining,
		)

		return nil
	})
}
