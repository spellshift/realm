package cdn

import (
	"net/http"
	"strings"
	"time"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
	"realm.pub/tavern/internal/errors"
)

// HeaderIfNoneMatch is the name of the header the client should set if they wish to only download the asset if it has been modified since the provided hash.
// HeaderEtag is set by the server to the hash of the downloaded asset.
const (
	HeaderIfNoneMatch = "If-None-Match"
	HeaderEtag        = "Etag"
)

// MaxChunkSize is the maximum number of bytes written in a single chunk when serving downloads.
// This is set to 31MB to accommodate GCP's 32MB response size limit.
var MaxChunkSize = 31 * 1024 * 1024

// serveChunkedContent writes the content to w in chunks of at most MaxChunkSize bytes,
// flushing after each chunk to ensure data is sent to the client incrementally.
// By not setting Content-Length, Go's net/http automatically uses Transfer-Encoding: chunked,
// which bypasses GCP Cloud Run's 32 MiB response size limit.
func serveChunkedContent(w http.ResponseWriter, content []byte, modTime time.Time) {
	if !modTime.IsZero() {
		w.Header().Set("Last-Modified", modTime.UTC().Format(http.TimeFormat))
	}

	for offset := 0; offset < len(content); offset += MaxChunkSize {
		end := offset + MaxChunkSize
		if end > len(content) {
			end = len(content)
		}
		if _, err := w.Write(content[offset:end]); err != nil {
			return
		}
		if f, ok := w.(http.Flusher); ok {
			f.Flush()
		}
	}
}

// NewDownloadHandler returns an HTTP handler responsible for downloading a asset from the CDN.
func NewDownloadHandler(graph *ent.Client, prefix string) http.Handler {
	return errors.WrapHandler(func(w http.ResponseWriter, req *http.Request) error {
		ctx := req.Context()

		// Get the Asset name from the request URI
		assetName := strings.TrimPrefix(req.URL.Path, prefix)
		if assetName == "" || assetName == "." || assetName == "/" {
			return ErrInvalidFileName
		}

		assetQuery := graph.Asset.Query().Where(asset.Name(assetName))

		// If hash was provided, check to see if the asset has been updated. Note that
		// http.ServeContent should handle this, but we want to avoid the expensive DB
		// query where possible.
		if hash := req.Header.Get(HeaderIfNoneMatch); hash != "" {
			if exists := assetQuery.Clone().Where(asset.Hash(hash)).ExistX(ctx); exists {
				return ErrFileNotModified
			}
		}

		// Ensure the asset exists
		if exists := assetQuery.Clone().ExistX(ctx); !exists {
			return ErrFileNotFound
		}

		a := assetQuery.OnlyX(ctx)

		// Set Etag to hash of asset
		w.Header().Set(HeaderEtag, a.Hash)

		// Set Content-Type and serve content in chunks
		w.Header().Set("Content-Type", "application/octet-stream")
		serveChunkedContent(w, a.Content, a.LastModifiedAt)

		return nil
	})
}
