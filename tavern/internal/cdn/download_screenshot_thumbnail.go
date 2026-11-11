package cdn

import (
	"net/http"
	"strconv"
	"strings"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/screenshot"
	"realm.pub/tavern/internal/errors"
)

// NewScreenshotThumbnailDownloadHandler returns an HTTP handler responsible for downloading the
// low resolution thumbnail of a Screenshot from the CDN. If the screenshot does not have a
// generated thumbnail (e.g. it was created before thumbnails were introduced, or the content
// could not be decoded as an image), the full resolution content is served instead.
func NewScreenshotThumbnailDownloadHandler(graph *ent.Client, prefix string) http.Handler {
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

		// Only load the fields needed to serve the thumbnail. The (potentially much larger)
		// full resolution content is loaded separately, and only as a fallback for screenshots
		// that have no thumbnail generated.
		f := fileQuery.Clone().
			Select(screenshot.FieldHash, screenshot.FieldCreatedAt, screenshot.FieldThumbnail).
			OnlyX(ctx)

		// Set Etag to hash of file
		w.Header().Set(HeaderEtag, f.Hash)

		// Fall back to the full resolution content if no thumbnail was generated.
		content := f.Thumbnail
		if len(content) == 0 {
			content = fileQuery.Clone().Select(screenshot.FieldContent).OnlyX(ctx).Content
		}

		// Set Content-Type and serve content in chunks
		w.Header().Set("Content-Type", "image/png")
		serveChunkedContent(w, content, f.CreatedAt)

		return nil
	})
}
