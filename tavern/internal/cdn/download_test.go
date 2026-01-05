package cdn_test

import (
	"fmt"
	"io/ioutil"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/crypto/sha3"
	"realm.pub/tavern/internal/cdn"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/errors"
)

// TestDownload asserts that the download handler exhibits expected behavior.
func TestDownload(t *testing.T) {
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	expectedContent := []byte("file_content")
	existingAsset := newAsset(graph, "ExistingTestAsset", expectedContent)

	t.Run("Asset", newDownloadTest(
		graph,
		newDownloadRequest(existingAsset.Name),
		func(t *testing.T, fileContent []byte, err *errors.HTTP) {
			assert.Nil(t, err)
			assert.Equal(t, string(expectedContent), string(fileContent))
		},
	))
	t.Run("CachedAsset", newDownloadTest(
		graph,
		newDownloadRequest(existingAsset.Name, withIfNoneMatchHeader(existingAsset.Hash)),
		func(t *testing.T, fileContent []byte, err *errors.HTTP) {
			require.NotNil(t, err)
			assert.Equal(t, http.StatusNotModified, err.StatusCode)
			assert.ErrorContains(t, err, cdn.ErrFileNotModified.Error())
			assert.Empty(t, string(fileContent))
		},
	))
	t.Run("NonExistentFile", newDownloadTest(
		graph,
		newDownloadRequest("ThisFileDoesNotExist"),
		func(t *testing.T, fileContent []byte, err *errors.HTTP) {
			require.NotNil(t, err)
			assert.Equal(t, http.StatusNotFound, err.StatusCode)
			assert.ErrorContains(t, err, cdn.ErrFileNotFound.Error())
			assert.Empty(t, string(fileContent))
		},
	))
}

// newDownloadTest initializes a new test case for the download handler.
func newDownloadTest(graph *ent.Client, req *http.Request, checks ...func(t *testing.T, fileContent []byte, err *errors.HTTP)) func(*testing.T) {
	return func(t *testing.T) {
		// Initialize Download Handler
		handler := cdn.NewDownloadHandler(graph, "/download/")

		// Send request and record response
		w := httptest.NewRecorder()
		handler.ServeHTTP(w, req)

		// Attempt to read the response body
		result := w.Result()
		body, err := ioutil.ReadAll(result.Body)
		require.NoError(t, err, "failed to parse response body")
		defer result.Body.Close()

		// If successful, ensure Etag was properly set and run checks on the file
		if result.StatusCode == http.StatusOK {
			// Ensure the ETag was properly set by the server on successful requests
			hash := fmt.Sprintf("%x", sha3.Sum256(body))
			assert.Equal(t, hash, result.Header.Get(cdn.HeaderEtag))

			// Run Checks
			for _, check := range checks {
				check(t, body, nil)
			}
			return
		}

		// Parse Error from failed response and run checks
		httpErr := errors.NewHTTP(string(body), result.StatusCode)
		for _, check := range checks {
			check(t, nil, &httpErr)
		}
	}
}

// newDownloadRequest is a helper to create an http.Request for a file download
func newDownloadRequest(fileName string, options ...func(*http.Request)) *http.Request {
	req := httptest.NewRequest(http.MethodGet, "/download/"+fileName, nil)
	for _, opt := range options {
		opt(req)
	}
	return req
}

// withIfNoneMatchHeader is an option to set the If-None-Match header on an http.Request
func withIfNoneMatchHeader(hash string) func(*http.Request) {
	return func(req *http.Request) {
		req.Header.Set(cdn.HeaderIfNoneMatch, hash)
	}
}
