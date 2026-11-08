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

		ctx := req.Context()
		if !graph.Host.Query().Where(host.ID(hostID)).ExistX(ctx) {
			return ErrFileNotFound
		}

		ids, err := graph.Screenshot.Query().
			Where(screenshot.HasHostWith(host.ID(hostID))).
			Order(ent.Desc(screenshot.FieldCreatedAt), ent.Desc(screenshot.FieldID)).
			IDs(ctx)
		if err != nil {
			return err
		}

		w.Header().Set("Content-Type", "application/zip")
		w.Header().Set("Content-Disposition", fmt.Sprintf("attachment; filename=host-%d-screenshots.zip", hostID))
		archive := zip.NewWriter(w)
		for _, id := range ids {
			item, err := graph.Screenshot.Get(ctx, id)
			if err != nil {
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
		return archive.Close()
	})
}
