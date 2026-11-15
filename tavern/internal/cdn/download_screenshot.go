package cdn

import (
	"net/http"
	"strconv"
	"strings"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/screenshot"
	"realm.pub/tavern/internal/errors"
	"realm.pub/tavern/internal/logging"
)

// NewScreenshotDownloadHandler returns an HTTP handler responsible for downloading a Screenshot from the CDN.
func NewScreenshotDownloadHandler(graph *ent.Client, prefix string) http.Handler {
	return errors.WrapHandler(func(w http.ResponseWriter, req *http.Request) error {
		ctx := cdnAuditCtx(req)

		// Get the Screenshot ID from the request URI
		fileIDStr := strings.TrimPrefix(req.URL.Path, prefix)
		if fileIDStr == "" || fileIDStr == "." || fileIDStr == "/" || len(fileIDStr) > maxFileIDLen {
			return ErrInvalidFileID
		}
		fileID, err := strconv.Atoi(fileIDStr)
		if err != nil {
			return ErrInvalidFileID
		}

		fileQuery := graph.Screenshot.Query().Where(screenshot.ID(fileID))

		// If hash was provided, check to see if the file has been updated.
		if hash := req.Header.Get(HeaderIfNoneMatch); hash != "" {
			if exists := fileQuery.Clone().Where(screenshot.Hash(hash)).ExistX(ctx); exists {
				downloadLog.DebugContext(ctx, "screenshot not modified", "screenshot_id", fileID)
				return ErrFileNotModified
			}
		}

		// Ensure the file exists
		if exists := fileQuery.Clone().ExistX(ctx); !exists {
			downloadLog.DebugContext(ctx, "screenshot not found", "screenshot_id", fileID)
			return ErrFileNotFound
		}

		f := fileQuery.WithHost().OnlyX(ctx)

		// Set Etag to hash of file
		w.Header().Set(HeaderEtag, f.Hash)

		// Set Content-Type and serve content in chunks
		w.Header().Set("Content-Type", "image/png")
		serveChunkedContent(w, f.Content, f.CreatedAt)

		attrs := append(logging.TargetIntAttrs("screenshot", f.ID, f.Name),
			"screenshot_id", f.ID,
			"mode", "single",
		)
		if f.Edges.Host != nil {
			attrs = append(attrs, "host_id", f.Edges.Host.ID)
		}
		logging.Audit(ctx, "cdn", "cdn.screenshot_downloaded", "screenshot downloaded from CDN", attrs...)

		return nil
	})
}
