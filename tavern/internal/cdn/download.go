package cdn

import (
	"bytes"
	"net/http"
	"strings"

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

		// Set Content-Type and serve content
		w.Header().Set("Content-Type", "application/octet-stream")
		http.ServeContent(w, req, a.Name, a.LastModifiedAt, bytes.NewReader(a.Content))

		return nil
	})
}
