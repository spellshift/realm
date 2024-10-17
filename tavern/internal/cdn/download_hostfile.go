package cdn

import (
	"bytes"
	"net/http"
	"path/filepath"
	"strconv"
	"strings"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/hostfile"
	"realm.pub/tavern/internal/errors"
)

// maxFileIDLen is the maximum length the string for a file id may be.
const (
	maxFileIDLen = 256
)

// NewHostFileDownloadHandler returns an HTTP handler responsible for downloading a HostFile from the CDN.
func NewHostFileDownloadHandler(graph *ent.Client, prefix string) http.Handler {
	return errors.WrapHandler(func(w http.ResponseWriter, req *http.Request) error {
		ctx := req.Context()

		// Get the HostFile ID from the request URI
		fileIDStr := strings.TrimPrefix(req.URL.Path, prefix)
		if fileIDStr == "" || fileIDStr == "." || fileIDStr == "/" || len(fileIDStr) > maxFileIDLen {
			return ErrInvalidFileID
		}
		fileID, err := strconv.Atoi(fileIDStr)
		if err != nil {
			return ErrInvalidFileID
		}

		fileQuery := graph.HostFile.Query().Where(hostfile.ID(fileID))

		// If hash was provided, check to see if the file has been updated. Note that
		// http.ServeContent should handle this, but we want to avoid the expensive DB
		// query where possible.
		if hash := req.Header.Get(HeaderIfNoneMatch); hash != "" {
			if exists := fileQuery.Clone().Where(hostfile.Hash(hash)).ExistX(ctx); exists {
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
		http.ServeContent(w, req, filepath.Base(f.Path), f.LastModifiedAt, bytes.NewReader(f.Content))

		return nil
	})
}
