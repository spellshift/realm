package cdn

import (
	"bytes"
	"net/http"
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
		// Basic validation
		if fileIDStr == "" || fileIDStr == "." || fileIDStr == "/" || len(fileIDStr) > 256 {
			return ErrInvalidFileID
		}

		fileID, err := strconv.Atoi(fileIDStr)
		if err != nil {
			return ErrInvalidFileID
		}

		// Prepare query
		q := graph.Screenshot.Query().Where(screenshot.ID(fileID))

		// If hash was provided, check to see if the file has been updated.
		if hash := req.Header.Get(HeaderIfNoneMatch); hash != "" {
			exists, err := q.Clone().Where(screenshot.Hash(hash)).Exist(ctx)
			if err != nil {
				return err
			}
			if exists {
				return ErrFileNotModified
			}
		}

		// Get the screenshot
		s, err := q.Only(ctx)
		if ent.IsNotFound(err) {
			return ErrFileNotFound
		}
		if err != nil {
			return err
		}

		// Set Etag
		if s.Hash != "" {
			w.Header().Set(HeaderEtag, s.Hash)
		}

		w.Header().Set("Content-Type", "image/bmp")
		http.ServeContent(w, req, "screenshot.bmp", s.LastModifiedAt, bytes.NewReader(s.Content))

		return nil
	})
}
