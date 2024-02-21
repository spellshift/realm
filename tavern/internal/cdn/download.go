package cdn

import (
	"bytes"
	"net/http"
	"strings"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/file"
	"realm.pub/tavern/internal/errors"
)

// HeaderIfNoneMatch is the name of the header the client should set if they wish to only download the file if it has been modified since the provided hash.
// HeaderEtag is set by the server to the hash of the downloaded file.
const (
	HeaderIfNoneMatch = "If-None-Match"
	HeaderEtag        = "Etag"
)

// NewDownloadHandler returns an HTTP handler responsible for downloading a file from the CDN.
func NewDownloadHandler(graph *ent.Client, prefix string) http.Handler {
	return errors.WrapHandler(func(w http.ResponseWriter, req *http.Request) error {
		ctx := req.Context()

		// Get the File name from the request URI
		fileName := strings.TrimPrefix(req.URL.Path, prefix)
		if fileName == "" || fileName == "." || fileName == "/" {
			return ErrInvalidFileName
		}

		fileQuery := graph.File.Query().Where(file.Name(fileName))

		// If hash was provided, check to see if the file has been updated. Note that
		// http.ServeContent should handle this, but we want to avoid the expensive DB
		// query where possible.
		if hash := req.Header.Get(HeaderIfNoneMatch); hash != "" {
			if exists := fileQuery.Clone().Where(file.Hash(hash)).ExistX(ctx); exists {
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
		w.Header().Set("Content-Type", "application/octet-stream")
		http.ServeContent(w, req, f.Name, f.LastModifiedAt, bytes.NewReader(f.Content))

		return nil
	})
}
