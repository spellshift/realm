package cdn

import (
	"fmt"
	"io/ioutil"
	"net/http"
	"time"

	"github.com/kcarretto/realm/cmd/tavern/internal/errors"
	"github.com/kcarretto/realm/ent"
	"github.com/kcarretto/realm/ent/file"
	"golang.org/x/crypto/sha3"
)

// DefaultMaxUploadSize defines the maximum number of bytes an uploaded file can be.
const DefaultMaxUploadSize = 10 << 20

// NewUploadHandler returns an HTTP handler responsible for uploading a file to the CDN.
func NewUploadHandler(graph *ent.Client) http.Handler {
	return errors.WrapHandler(func(w http.ResponseWriter, req *http.Request) error {
		ctx := req.Context()

		// Get the File name
		if err := req.ParseMultipartForm(DefaultMaxUploadSize); err != nil {
			return err
		}
		fileName := req.PostFormValue("fileName")
		if fileName == "" {
			return ErrInvalidFileName
		}

		// Get the File content
		f, _, err := req.FormFile("fileContent")
		if err != nil {
			return fmt.Errorf("%w: %v", ErrInvalidFileContent, err)
		}
		defer f.Close()
		fileContent, err := ioutil.ReadAll(f)
		if err != nil {
			return fmt.Errorf("%w: %v", ErrInvalidFileContent, err)
		}

		// Check if it has already been uploaded
		fileQuery := graph.File.Query().Where(file.Name(fileName))
		exists := fileQuery.Clone().ExistX(ctx)

		// Create or Update the file
		var fileID int
		hash := fmt.Sprintf("%x", sha3.Sum256(fileContent))

		if exists {
			fileID = fileQuery.OnlyIDX(ctx)
			graph.File.UpdateOneID(fileID).
				SetSize(len(fileContent)).
				SetHash(hash).
				SetContent(fileContent).
				SetLastModifiedAt(time.Now()).
				SaveX(ctx)
		} else {
			fileID = graph.File.Create().
				SetName(fileName).
				SetSize(len(fileContent)).
				SetHash(hash).
				SetContent(fileContent).
				SetLastModifiedAt(time.Now()).
				SaveX(ctx).ID
		}

		// Respond with JSON of the file ID
		fmt.Fprintf(w, `{"data":{"file":{"id":%d}}}`, fileID)
		return nil
	})
}
