package cdn_test

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"mime/multipart"
	"net/http"
	"net/http/httptest"
	"testing"

	"realm.pub/tavern/internal/cdn"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/crypto/sha3"
)

// TestUpload asserts that the upload handler exhibits expected behavior.
func TestUpload(t *testing.T) {
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	expectedContent := []byte("file_content")
	newExpectedContent := []byte("new_file_content")
	existingAsset := newAsset(graph, "ExistingTestAsset", expectedContent)

	t.Run("NewAsset", newUploadTest(
		graph,
		newUploadRequest(t, "NewUploadTestAsset", expectedContent),
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotEqual(t, 0, id)

			a, err := graph.Asset.Get(context.Background(), id)
			require.NoError(t, err)
			assert.Equal(t, id, a.ID)
			assert.Equal(t, "NewUploadTestAsset", a.Name)
			assert.Equal(t, len(expectedContent), a.Size)
			assert.Equal(t, fmt.Sprintf("%x", sha3.Sum256(expectedContent)), a.Hash)
			assert.Equal(t, expectedContent, a.Content)
		},
	))
	t.Run("ExistingAsset", newUploadTest(
		graph,
		newUploadRequest(t, existingAsset.Name, newExpectedContent),
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotEqual(t, 0, id)

			a, err := graph.Asset.Get(context.Background(), id)
			require.NoError(t, err)
			assert.Equal(t, existingAsset.ID, a.ID)
			assert.Equal(t, existingAsset.Name, a.Name)
			assert.Equal(t, len(newExpectedContent), a.Size)
			assert.Equal(t, fmt.Sprintf("%x", sha3.Sum256(newExpectedContent)), a.Hash)
			assert.Equal(t, newExpectedContent, a.Content)
		},
	))

}

// newUploadTest initializes a new test case for the upload handler.
func newUploadTest(graph *ent.Client, req *http.Request, checks ...func(t *testing.T, id int, err error)) func(*testing.T) {
	return func(t *testing.T) {
		// Initialize Upload Handler
		handler := cdn.NewUploadHandler(graph)

		// Send request and record response
		w := httptest.NewRecorder()
		handler.ServeHTTP(w, req)

		// Attempt to read the response body
		result := w.Result()
		body, err := ioutil.ReadAll(result.Body)
		require.NoError(t, err, "failed to parse response body")
		defer result.Body.Close()

		// Parse AssetID from successful response and run checks
		if result.StatusCode == 200 {
			var resp struct {
				Data struct {
					Asset struct {
						ID int `json:"id"`
					} `json:"asset"`
				} `json:"data"`
			}
			require.NoError(t, json.Unmarshal(body, &resp), "failed to unmarshal json with 200 response code: %s", body)

			for _, check := range checks {
				check(t, resp.Data.Asset.ID, nil)
			}
			return
		}

		// Parse Error from failed response and run checks
		for _, check := range checks {
			check(t, 0, fmt.Errorf("%s", body))
		}
	}
}

// newUploadRequest is a helper to create an http.Request for a file upload
func newUploadRequest(t *testing.T, fileName string, fileContent []byte) *http.Request {
	// Create upload form
	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)

	// Add file content
	fileWriter, err := writer.CreateFormFile("fileContent", fileName)
	require.NoError(t, err)
	n, err := fileWriter.Write(fileContent)
	require.NoError(t, err)
	require.Equal(t, len(fileContent), n)

	// Add file name
	err = writer.WriteField("fileName", fileName)
	require.NoError(t, err)

	// Close the form writer
	require.NoError(t, writer.Close())

	// Create the request
	req := httptest.NewRequest(http.MethodPost, "/upload", body)
	req.Header.Set("Content-Type", writer.FormDataContentType())

	return req
}

// newAsset is a helper to create assets directly via ent
func newAsset(graph *ent.Client, name string, content []byte) *ent.Asset {
	return graph.Asset.Create().
		SetName(name).
		SetContent(content).
		SaveX(context.Background())
}
