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

		// If hash was provided, check to see if the file has been updated.
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
		w.Header().Set("Content-Type", "image/png")
		http.ServeContent(w, req, filepath.Base(f.Name), f.CreatedAt, bytes.NewReader(f.Content))

		return nil
	})
}
