package cdn_test

import (
	"archive/zip"
	"bytes"
	"context"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/cdn"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestDownloadHostScreenshots(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()
	ctx := context.Background()

	host := graph.Host.Create().SetIdentifier("screen-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	other := graph.Host.Create().SetIdentifier("other-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	first := graph.Screenshot.Create().SetName("first.png").SetContent([]byte("first image")).SetHost(host).SaveX(ctx)
	second := graph.Screenshot.Create().SetName("../../second.png").SetContent([]byte("second image")).SetHost(host).SaveX(ctx)
	graph.Screenshot.Create().SetName("other.png").SetContent([]byte("other image")).SetHost(other).SaveX(ctx)

	handler := cdn.NewHostScreenshotsDownloadHandler(graph, "/download/")
	req := httptest.NewRequest(http.MethodGet, fmt.Sprintf("/download/%d", host.ID), nil)
	w := httptest.NewRecorder()
	handler.ServeHTTP(w, req)

	require.Equal(t, http.StatusOK, w.Code)
	require.Equal(t, "application/zip", w.Header().Get("Content-Type"))
	require.Contains(t, w.Header().Get("Content-Disposition"), "attachment;")
	archive, err := zip.NewReader(bytes.NewReader(w.Body.Bytes()), int64(w.Body.Len()))
	require.NoError(t, err)
	require.Len(t, archive.File, 2)
	contents := map[string]string{}
	for _, item := range archive.File {
		reader, err := item.Open()
		require.NoError(t, err)
		body, err := io.ReadAll(reader)
		require.NoError(t, err)
		require.NoError(t, reader.Close())
		contents[item.Name] = string(body)
	}
	require.Equal(t, map[string]string{
		fmt.Sprintf("%d-first.png", first.ID):   "first image",
		fmt.Sprintf("%d-second.png", second.ID): "second image",
	}, contents)

	for _, tc := range []struct {
		path string
		code int
	}{
		{"/download/abc", cdn.ErrInvalidFileID.StatusCode},
		{"/download/99999", cdn.ErrFileNotFound.StatusCode},
	} {
		w := httptest.NewRecorder()
		handler.ServeHTTP(w, httptest.NewRequest(http.MethodGet, tc.path, nil))
		require.Equal(t, tc.code, w.Code)
	}
}
