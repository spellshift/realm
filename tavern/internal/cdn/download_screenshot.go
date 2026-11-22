package cdn

import (
	"bytes"
	"net/http"
	"strconv"
	"strings"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/screenshot"
	"realm.pub/tavern/internal/errors"
	"realm.pub/tavern/internal/logging"
)

var pngMagic = []byte{0x89, 'P', 'N', 'G', '\r', '\n', 0x1a, '\n'}

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
		if err != nil || fileID <= 0 {
			return ErrInvalidFileID
		}

		fileQuery := graph.Screenshot.Query().Where(screenshot.ID(fileID))

		// If hash was provided, check to see if the file has been updated.
		if hash := req.Header.Get(HeaderIfNoneMatch); hash != "" {
			exists, err := fileQuery.Clone().Where(screenshot.Hash(hash)).Exist(ctx)
			if err != nil {
				return err
			}
			if exists {
				downloadLog.DebugContext(ctx, "screenshot not modified", "screenshot_id", fileID)
				return ErrFileNotModified
			}
		}

		f, err := fileQuery.WithHost().Only(ctx)
		if ent.IsNotFound(err) {
			downloadLog.DebugContext(ctx, "screenshot not found", "screenshot_id", fileID)
			return ErrFileNotFound
		}
		if err != nil {
			return err
		}

		// Set Etag to hash of file
		w.Header().Set(HeaderEtag, f.Hash)
		w.Header().Set("X-Content-Type-Options", "nosniff")

		// Only serve inline as image/png if the payload actually starts with the PNG magic header;
		// fall back to an octet-stream attachment for untrusted/non-PNG bytes.
		if bytes.HasPrefix(f.Content, pngMagic) {
			w.Header().Set("Content-Type", "image/png")
		} else {
			w.Header().Set("Content-Type", "application/octet-stream")
			w.Header().Set("Content-Disposition", "attachment")
		}
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
