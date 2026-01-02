package cdn_test

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/cdn"
	"realm.pub/tavern/internal/ent/enttest"
)

// TestDownloadHostFile asserts that the download handler exhibits expected behavior.
func TestDownloadHostFile(t *testing.T) {
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	ctx := context.Background()
	existingHost := graph.Host.Create().
		SetIdentifier("test-host").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)
	existingBeacon := graph.Beacon.Create().
		SetHost(existingHost).
		SetIdentifier("ABCDEFG").
		SetTransport(c2pb.ActiveTransport_TRANSPORT_UNSPECIFIED).
		SaveX(ctx)
	existingTome := graph.Tome.Create().
		SetName("Wowza").
		SetDescription("Why did we require this?").
		SetAuthor("kcarretto").
		SetEldritch("blah").
		SaveX(ctx)
	existingQuest := graph.Quest.Create().
		SetName("HelloWorld").
		SetTome(existingTome).
		SaveX(ctx)
	existingTask := graph.Task.Create().
		SetBeacon(existingBeacon).
		SetQuest(existingQuest).
		SaveX(ctx)

	existingHostFile := graph.HostFile.Create().
		SetPath("/existing/file").
		SetContent([]byte(`some data`)).
		SetHost(existingHost).
		SetTask(existingTask).
		SaveX(ctx)

	handler := cdn.NewHostFileDownloadHandler(graph, "/download/")

	tests := []struct {
		name string

		reqURL     string
		reqMethod  string
		reqBody    io.Reader
		reqHeaders map[string][]string

		wantStatus int
		wantBody   []byte
		wantErr    error
	}{
		{
			name:     "Valid",
			reqURL:   fmt.Sprintf("/download/%d", existingHostFile.ID),
			wantBody: existingHostFile.Content,
		},
		{
			name:       "NotFound",
			reqURL:     "/download/123",
			wantBody:   []byte(fmt.Sprintf("%s\n", cdn.ErrFileNotFound.Error())),
			wantStatus: cdn.ErrFileNotFound.StatusCode,
		},
		{
			name:       "InvalidID/Alphabet",
			reqURL:     "/download/abcd",
			wantBody:   []byte(fmt.Sprintf("%s\n", cdn.ErrInvalidFileID.Error())),
			wantStatus: cdn.ErrInvalidFileID.StatusCode,
		},
		{
			name:       "InvalidID/Empty",
			reqURL:     "/download/",
			wantBody:   []byte(fmt.Sprintf("%s\n", cdn.ErrInvalidFileID.Error())),
			wantStatus: cdn.ErrInvalidFileID.StatusCode,
		},
		{
			name:   "Cached",
			reqURL: fmt.Sprintf("/download/%d", existingHostFile.ID),
			reqHeaders: map[string][]string{
				cdn.HeaderIfNoneMatch: {existingHostFile.Hash},
			},
			wantBody:   []byte(fmt.Sprintf("%s\n", cdn.ErrFileNotModified.Error())),
			wantStatus: cdn.ErrFileNotModified.StatusCode,
		},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			// Build Request
			req, reqErr := http.NewRequest(tc.reqMethod, tc.reqURL, tc.reqBody)
			require.NoError(t, reqErr)
			for key, vals := range tc.reqHeaders {
				for _, val := range vals {
					req.Header.Add(key, val)
				}
			}

			// Default to wanting OK status
			if tc.wantStatus == 0 {
				tc.wantStatus = http.StatusOK
			}

			// Send request and record response
			w := httptest.NewRecorder()
			handler.ServeHTTP(w, req)

			result := w.Result()
			assert.Equal(t, tc.wantStatus, result.StatusCode)

			// Attempt to read the response body
			body, err := io.ReadAll(result.Body)
			require.NoError(t, err, "failed to parse response body")
			defer result.Body.Close()

			assert.Equal(t, tc.wantBody, body)
		})
	}
}
