package cdn

import (
	"bytes"
	"fmt"
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
		if exists, err := linkQuery.Clone().Exist(ctx); !exists || err != nil {
			return ErrFileNotFound
		}

		l, err := linkQuery.Only(ctx)
		if err != nil {
			return ErrFileNotFound
		}

		// Check if the link is active based on active_clicks or active_before
		currentTime := time.Now()
		hasDownloadsRemaining := l.DownloadsRemaining > 0
		hasTimeRemaining := currentTime.Before(l.ExpiresAt)

		// If neither condition is met, return 404
		if !hasDownloadsRemaining && !hasTimeRemaining {
			return ErrFileNotFound
		}

		// If active by clicks, decrement the counter
		if hasDownloadsRemaining {
			_, err := graph.Link.UpdateOne(l).
				SetDownloadsRemaining(l.DownloadsRemaining - 1).
				Save(ctx)
			if err != nil {
				return fmt.Errorf("failed to decrement active clicks: %w", err)
			}
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
