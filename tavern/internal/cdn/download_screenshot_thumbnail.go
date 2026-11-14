package cdn

import (
	"net/http"
	"strconv"
	"strings"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/screenshot"
	"realm.pub/tavern/internal/errors"
)

// NewScreenshotThumbnailDownloadHandler streams the low resolution thumbnail of a Screenshot
// from the CDN. If the screenshot does not have a generated thumbnail (e.g. it was created
// before thumbnails were introduced, or its content could not be decoded as an image), the
// full resolution content is served instead.
func NewScreenshotThumbnailDownloadHandler(graph *ent.Client, prefix string) http.Handler {
	return errors.WrapHandler(func(w http.ResponseWriter, req *http.Request) error {
		screenshotIDStr := strings.TrimPrefix(req.URL.Path, prefix)
		if screenshotIDStr == "" || screenshotIDStr == "." || screenshotIDStr == "/" || len(screenshotIDStr) > maxFileIDLen {
			return ErrInvalidFileID
		}
		screenshotID, err := strconv.Atoi(screenshotIDStr)
		if err != nil || screenshotID <= 0 {
			return ErrInvalidFileID
		}

		ctx := req.Context()

		screenshotQuery := graph.Screenshot.Query().Where(screenshot.ID(screenshotID))

		// If hash was provided, check to see if the file has been updated.
		if hash := req.Header.Get(HeaderIfNoneMatch); hash != "" {
			if exists := screenshotQuery.Clone().Where(screenshot.Hash(hash)).ExistX(ctx); exists {
				return ErrFileNotModified
			}
		}

		if !screenshotQuery.Clone().ExistX(ctx) {
			return ErrFileNotFound
		}

		// Only load the fields needed to serve the thumbnail. The (potentially much larger)
		// full resolution content is loaded separately, and only as a fallback for screenshots
		// that have no thumbnail generated.
		item := screenshotQuery.Clone().
			Select(screenshot.FieldHash, screenshot.FieldCreatedAt, screenshot.FieldThumbnail).
			OnlyX(ctx)

		// Set Etag to hash of file
		w.Header().Set(HeaderEtag, item.Hash)

		// Fall back to the full resolution content if no thumbnail was generated.
		content := item.Thumbnail
		if len(content) == 0 {
			content = screenshotQuery.Clone().Select(screenshot.FieldContent).OnlyX(ctx).Content
		}

		// Set Content-Type and serve content in chunks
		w.Header().Set("Content-Type", "image/png")
		serveChunkedContent(w, content, item.CreatedAt)

		return nil
	})
}
