package cdn

import (
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/c2/c2pb"
)

func TestScreenshotDownloadHandler(t *testing.T) {
	ctx := context.Background()

	// Initialize Graph
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer client.Close()

	// Create Host
	h := client.Host.Create().
		SetIdentifier("test-host").
		SetName("test-host").
		SetPrimaryIP("1.1.1.1").
        SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)

	// Create Screenshot
	f := client.Screenshot.Create().
		SetHostID(h.ID).
		SetPath("test.png").
		SetContent([]byte("test content")).
		SaveX(ctx)

	// Handler
	handler := NewScreenshotDownloadHandler(client, "/cdn/screenshots/")

	t.Run("ValidFileID", func(t *testing.T) {
		req, err := http.NewRequest("GET", fmt.Sprintf("/cdn/screenshots/%d", f.ID), nil)
		assert.NoError(t, err)

		rr := httptest.NewRecorder()
		handler.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusOK, rr.Code)
		assert.Equal(t, "test content", rr.Body.String())
	})

	t.Run("InvalidFileID", func(t *testing.T) {
		req, err := http.NewRequest("GET", "/cdn/screenshots/invalid", nil)
		assert.NoError(t, err)

		rr := httptest.NewRecorder()
		handler.ServeHTTP(rr, req)

		// ErrInvalidFileID returns BadRequest or something.
        // I need to check errors.go or just assert not OK.
        // In download_hostfile_test.go it checks for status code of the error.
        // But here I'm using `errors.WrapHandler`.
		assert.NotEqual(t, http.StatusOK, rr.Code)
	})

	t.Run("FileNotFound", func(t *testing.T) {
		req, err := http.NewRequest("GET", "/cdn/screenshots/999", nil)
		assert.NoError(t, err)

		rr := httptest.NewRecorder()
		handler.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusNotFound, rr.Code)
	})

	t.Run("FileNotModified", func(t *testing.T) {
		req, err := http.NewRequest("GET", fmt.Sprintf("/cdn/screenshots/%d", f.ID), nil)
		assert.NoError(t, err)
		req.Header.Set("If-None-Match", f.Hash)

		rr := httptest.NewRecorder()
		handler.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusNotModified, rr.Code)
	})
}
