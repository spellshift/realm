package cdn

import (
	"bytes"
	"net/http"
	"strings"
	"time"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/link"
	"realm.pub/tavern/internal/errors"
)

// NewLinkDownloadHandler returns an HTTP handler responsible for downloading a file via a Link from the CDN.
// Files are accessed using the link path. The link's active_clicks and active_before fields determine
// if the file can be downloaded:
// - If active_clicks > 0, the file is returned and the counter is decremented
// - If current time < active_before, the file is returned
// - Otherwise, 404 is returned
func NewLinkDownloadHandler(graph *ent.Client, prefix string) http.Handler {
	return errors.WrapHandler(func(w http.ResponseWriter, req *http.Request) error {
		ctx := req.Context()

		// Get the link path from the request URI
		linkPath := strings.TrimPrefix(req.URL.Path, prefix)
		if linkPath == "" || linkPath == "." || linkPath == "/" {
			return ErrFileNotFound
		}

		// Query for the link by path, including the associated file
		linkQuery := graph.Link.Query().
			Where(link.Path(linkPath)).
			WithFile()

		// Ensure the link exists
		if exists := linkQuery.Clone().ExistX(ctx); !exists {
			return ErrFileNotFound
		}

		l := linkQuery.OnlyX(ctx)

		// Check if the link is active based on active_clicks or active_before
		currentTime := time.Now()
		isActiveByClicks := l.ActiveClicks > 0
		isActiveByTime := currentTime.Before(l.ActiveBefore)

		// If neither condition is met, return 404
		if !isActiveByClicks && !isActiveByTime {
			return ErrFileNotFound
		}

		// If active by clicks, decrement the counter
		if isActiveByClicks {
			graph.Link.UpdateOne(l).
				SetActiveClicks(l.ActiveClicks - 1).
				SaveX(ctx)
		}

		// Get the associated file
		f := l.Edges.File
		if f == nil {
			return ErrFileNotFound
		}

		// Set Etag to hash of file
		w.Header().Set(HeaderEtag, f.Hash)

		// Set Content-Type and serve content
		w.Header().Set("Content-Type", "application/octet-stream")
		http.ServeContent(w, req, f.Name, f.LastModifiedAt, bytes.NewReader(f.Content))

		return nil
	})
}
