package cdn

import (
	"crypto/md5"
	"encoding/hex"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strconv"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
	"realm.pub/tavern/internal/errors"
)

// DefaultMaxUploadSize defines the maximum number of bytes an uploaded asset can be.
const DefaultMaxUploadSize = 512 * 1024 * 1024 // 512MB
// MaxMemory defines the max memory used when parsing the multipart form
const MaxMemory = 64 * 1024 * 1024 // 64MB

// NewUploadHandler returns an HTTP handler responsible for uploading an asset to the CDN.
func NewUploadHandler(graph *ent.Client) http.Handler {
	return errors.WrapHandler(func(w http.ResponseWriter, req *http.Request) error {
		ctx := req.Context()

		// 1. Parse the multipart form using a smaller memory footprint
		if err := req.ParseMultipartForm(MaxMemory); err != nil {
			return err
		}

		assetName := req.PostFormValue("fileName")
		if assetName == "" {
			return ErrInvalidFileName // Assuming this is defined elsewhere
		}

		// 2. Extract chunk metadata
		chunkIndexStr := req.PostFormValue("chunkIndex")
		totalChunksStr := req.PostFormValue("totalChunks")

		chunkIndex, _ := strconv.Atoi(chunkIndexStr)
		totalChunks, _ := strconv.Atoi(totalChunksStr)

		// 3. Get the Asset content chunk
		file, _, err := req.FormFile("fileContent")
		if err != nil {
			return fmt.Errorf("%w: %v", ErrInvalidFileContent, err) // Assuming ErrInvalidFileContent is defined
		}
		defer file.Close()

		// Get the creator
		var creatorID *int
		creatorSuffix := "anon"
		if creator := auth.UserFromContext(ctx); creator != nil {
			creatorID = &creator.ID
			creatorSuffix = strconv.Itoa(creator.ID)
		}

		// 4. Create a unique temporary file path for this specific upload
		// Hashing the name + creator ensures no path traversal issues and groups chunks correctly
		hashInput := fmt.Sprintf("%s_%s", creatorSuffix, assetName)
		hash := md5.Sum([]byte(hashInput))
		tempFileName := hex.EncodeToString(hash[:])
		tempFilePath := filepath.Join(os.TempDir(), tempFileName)

		// If it's the first chunk, ensure we start with a fresh file (in case of a previous failed upload)
		if chunkIndex == 0 {
			os.Remove(tempFilePath)
		}

		// Open temp file in append mode
		tempFile, err := os.OpenFile(tempFilePath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
		if err != nil {
			return fmt.Errorf("failed to open temp file: %v", err)
		}

		// Write the current chunk to the temp file
		if _, err := io.Copy(tempFile, file); err != nil {
			tempFile.Close()
			return fmt.Errorf("failed to write chunk: %v", err)
		}
		tempFile.Close()

		// 5. If this is NOT the last chunk, return early with a success status
		if chunkIndex < totalChunks-1 {
			w.WriteHeader(http.StatusOK)
			fmt.Fprintf(w, `{"status":"chunk %d received"}`, chunkIndex)
			return nil
		}

		// 6. If it IS the last chunk, process the complete file
		assetContent, err := os.ReadFile(tempFilePath)
		if err != nil {
			return fmt.Errorf("failed to read complete file: %v", err)
		}

		// Clean up the temp file now that we have it in memory
		defer os.Remove(tempFilePath)

		// Check if it has already been uploaded
		assetQuery := graph.Asset.Query().Where(asset.Name(assetName))
		exists := assetQuery.Clone().ExistX(ctx)

		// Create or Update the asset in the database
		var assetID int
		if exists {
			assetID = assetQuery.OnlyIDX(ctx)
			graph.Asset.UpdateOneID(assetID).
				SetContent(assetContent).
				SetNillableCreatorID(creatorID).
				SaveX(ctx)
		} else {
			assetID = graph.Asset.Create().
				SetName(assetName).
				SetContent(assetContent).
				SetNillableCreatorID(creatorID).
				SaveX(ctx).ID
		}

		// Respond with JSON of the asset ID
		w.WriteHeader(http.StatusOK)
		fmt.Fprintf(w, `{"data":{"asset":{"id":%d}}}`, assetID)
		return nil
	})
}
