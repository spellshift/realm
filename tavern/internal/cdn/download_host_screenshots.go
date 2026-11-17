package cdn

import (
	"archive/zip"
	"fmt"
	"net/http"
	"path"
	"strconv"
	"strings"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/host"
	"realm.pub/tavern/internal/ent/screenshot"
	"realm.pub/tavern/internal/errors"
	"realm.pub/tavern/internal/logging"
)

// NewHostScreenshotsDownloadHandler streams all screenshots for a host as a ZIP archive.
func NewHostScreenshotsDownloadHandler(graph *ent.Client, prefix string) http.Handler {
	return errors.WrapHandler(func(w http.ResponseWriter, req *http.Request) error {
		hostIDStr := strings.TrimPrefix(req.URL.Path, prefix)
		if hostIDStr == "" || hostIDStr == "." || hostIDStr == "/" || len(hostIDStr) > maxFileIDLen {
			return ErrInvalidFileID
		}
		hostID, err := strconv.Atoi(hostIDStr)
		if err != nil || hostID <= 0 {
			return ErrInvalidFileID
		}

		ctx := cdnAuditCtx(req)
		if !graph.Host.Query().Where(host.ID(hostID)).ExistX(ctx) {
			downloadLog.DebugContext(ctx, "host not found for screenshots archive", "host_id", hostID)
			return ErrFileNotFound
		}

		ids, err := graph.Screenshot.Query().
			Where(screenshot.HasHostWith(host.ID(hostID))).
			Order(ent.Desc(screenshot.FieldCreatedAt), ent.Desc(screenshot.FieldID)).
			IDs(ctx)
		if err != nil {
			downloadLog.ErrorContext(ctx, "failed to query host screenshots", "host_id", hostID, "error", err)
			return err
		}

		w.Header().Set("Content-Type", "application/zip")
		w.Header().Set("Content-Disposition", fmt.Sprintf("attachment; filename=host-%d-screenshots.zip", hostID))
		archive := zip.NewWriter(w)
		for _, id := range ids {
			item, err := graph.Screenshot.Get(ctx, id)
			if err != nil {
				downloadLog.ErrorContext(ctx, "failed to load screenshot for archive", "host_id", hostID, "screenshot_id", id, "error", err)
				return err
			}
			name := path.Base(strings.ReplaceAll(item.Name, "\\", "/"))
			if name == "." || name == "/" || name == "" {
				name = "screenshot.png"
			}
			entry, err := archive.Create(fmt.Sprintf("%d-%s", id, name))
			if err != nil {
				return err
			}
			if _, err := entry.Write(item.Content); err != nil {
				return err
			}
		}
		if err := archive.Close(); err != nil {
			return err
		}

		logging.Audit(ctx, "cdn", "cdn.screenshot_downloaded", "host screenshots archive downloaded from CDN",
			"host_id", hostID,
			"screenshot_count", len(ids),
			"mode", "all",
		)
		return nil
	})
}
