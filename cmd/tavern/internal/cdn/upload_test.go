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
	"time"

	"github.com/kcarretto/realm/cmd/tavern/internal/cdn"
	"github.com/kcarretto/realm/ent"
	"github.com/kcarretto/realm/ent/enttest"

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
	existingFile := newFile(graph, "ExistingTestFile", expectedContent)

	t.Run("NewFile", newUploadTest(
		graph,
		newUploadRequest(t, "NewUploadTestFile", expectedContent),
		func(id int, err error) {
			require.NoError(t, err)
			assert.NotEqual(t, 0, id)

			f, err := graph.File.Get(context.Background(), id)
			require.NoError(t, err)
			assert.Equal(t, id, f.ID)
			assert.Equal(t, "NewUploadTestFile", f.Name)
			assert.Equal(t, len(expectedContent), f.Size)
			assert.Equal(t, fmt.Sprintf("%x", sha3.Sum256(expectedContent)), f.Hash)
			assert.Equal(t, expectedContent, f.Content)
		},
	))
	t.Run("ExistingFile", newUploadTest(
		graph,
		newUploadRequest(t, existingFile.Name, newExpectedContent),
		func(id int, err error) {
			require.NoError(t, err)
			assert.NotEqual(t, 0, id)

			f, err := graph.File.Get(context.Background(), id)
			require.NoError(t, err)
			assert.Equal(t, existingFile.ID, f.ID)
			assert.Equal(t, existingFile.Name, f.Name)
			assert.Equal(t, len(newExpectedContent), f.Size)
			assert.Equal(t, fmt.Sprintf("%x", sha3.Sum256(newExpectedContent)), f.Hash)
			assert.Equal(t, newExpectedContent, f.Content)
		},
	))

}

// newUploadTest initializes a new test case for the upload handler.
func newUploadTest(graph *ent.Client, req *http.Request, checks ...func(id int, err error)) func(*testing.T) {
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

		// Parse FileID from successful response and run checks
		if result.StatusCode == 200 {
			var resp struct {
				Data struct {
					File struct {
						ID int `json:"id"`
					} `json:"file"`
				} `json:"data"`
			}
			require.NoError(t, json.Unmarshal(body, &resp), "failed to unmarshal json with 200 response code: %s", body)

			for _, check := range checks {
				check(resp.Data.File.ID, nil)
			}
			return
		}

		// Parse Error from failed response and run checks
		for _, check := range checks {
			check(0, fmt.Errorf("%s", body))
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

// newFile is a helper to create files directly via ent
func newFile(graph *ent.Client, name string, content []byte) *ent.File {
	return graph.File.Create().
		SetName(name).
		SetSize(len(content)).
		SetHash(fmt.Sprintf("%x", sha3.Sum256(content))).
		SetContent(content).
		SetLastModifiedAt(time.Now()).
		SaveX(context.Background())
}
