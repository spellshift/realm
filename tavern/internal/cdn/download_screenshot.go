package cdn

import (
	"bytes"
	"net/http"
	"path/filepath"
	"strconv"
	"strings"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/screenshot"
	"realm.pub/tavern/internal/errors"
)

// NewScreenshotDownloadHandler returns an HTTP handler responsible for downloading a Screenshot from the CDN.
func NewScreenshotDownloadHandler(graph *ent.Client, prefix string) http.Handler {
	return errors.WrapHandler(func(w http.ResponseWriter, req *http.Request) error {
		ctx := req.Context()

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

		// If hash was provided, check to see if the file has been updated. Note that
		// http.ServeContent should handle this, but we want to avoid the expensive DB
		// query where possible.
		if hash := req.Header.Get(HeaderIfNoneMatch); hash != "" {
			if exists := fileQuery.Clone().Where(screenshot.Hash(hash)).ExistX(ctx); exists {
				return ErrFileNotModified
			}
		}

		// Ensure the file exists
		if exists := fileQuery.Clone().ExistX(ctx); !exists {
			return ErrFileNotFound
		}

		f := fileQuery.OnlyX(ctx)

		// Set Etag to hash of file
		w.Header().Set(HeaderEtag, f.Hash)

		// Set Content-Type and serve content
		// Screenshots are PNGs, so we can set Content-Type if we know it, or just octet-stream.
		// Since we used xcap and converted to PNG, we can say image/png or just let browser detect.
		// But existing code uses application/octet-stream. I'll stick to that or mirror HostFile behavior.
		w.Header().Set("Content-Type", "application/octet-stream")
		http.ServeContent(w, req, filepath.Base(f.Path), f.LastModifiedAt, bytes.NewReader(f.Content))

		return nil
	})
}
