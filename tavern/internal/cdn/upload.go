package cdn

import (
	"fmt"
	"io/ioutil"
	"net/http"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
	"realm.pub/tavern/internal/errors"
)

// DefaultMaxUploadSize defines the maximum number of bytes an uploaded asset can be.
const DefaultMaxUploadSize = 100 * 1024 * 1024 // 10 MB

// NewUploadHandler returns an HTTP handler responsible for uploading a asset to the CDN.
func NewUploadHandler(graph *ent.Client) http.Handler {
	return errors.WrapHandler(func(w http.ResponseWriter, req *http.Request) error {
		ctx := req.Context()

		// Get the Asset name
		if err := req.ParseMultipartForm(DefaultMaxUploadSize); err != nil {
			return err
		}
		assetName := req.PostFormValue("fileName")
		if assetName == "" {
			return ErrInvalidFileName
		}

		// Get the Asset content
		f, _, err := req.FormFile("fileContent")
		if err != nil {
			return fmt.Errorf("%w: %v", ErrInvalidFileContent, err)
		}
		defer f.Close()
		assetContent, err := ioutil.ReadAll(f)
		if err != nil {
			return fmt.Errorf("%w: %v", ErrInvalidFileContent, err)
		}

		// Check if it has already been uploaded
		assetQuery := graph.Asset.Query().Where(asset.Name(assetName))
		exists := assetQuery.Clone().ExistX(ctx)

		// Create or Update the asset
		var assetID int

		if exists {
			assetID = assetQuery.OnlyIDX(ctx)
			graph.Asset.UpdateOneID(assetID).
				SetContent(assetContent).
				SaveX(ctx)
		} else {
			assetID = graph.Asset.Create().
				SetName(assetName).
				SetContent(assetContent).
				SaveX(ctx).ID
		}

		// Respond with JSON of the asset ID
		fmt.Fprintf(w, `{"data":{"asset":{"id":%d}}}`, assetID)
		return nil
	})
}
