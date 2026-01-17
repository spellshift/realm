package c2_test

import (
	"bytes"
	"context"
	"crypto/rand"
	"errors"
	"fmt"
	"io"
	"testing"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/c2test"
)

func TestFetchAsset(t *testing.T) {
	// Setup Dependencies
	ctx := context.Background()
	client, graph, close := c2test.New(t)
	defer close()

	// Test Cases
	type testCase struct {
		name     string
		fileName string
		fileSize int
		req      *c2pb.FetchAssetRequest
		wantCode codes.Code
	}
	tests := []testCase{
		{
			name:     "Small_File",
			fileName: "small_file",
			fileSize: 100,
			req:      &c2pb.FetchAssetRequest{Name: "small_file"},
			wantCode: codes.OK,
		},
		{
			name:     "Large_File",
			fileName: "large_file",
			fileSize: 1024 * 1024 * 10, // 10 MB
			req:      &c2pb.FetchAssetRequest{Name: "large_file"},
			wantCode: codes.OK,
		},
		{
			name:     "File Not Found",
			fileName: "n/a",
			req:      &c2pb.FetchAssetRequest{Name: "this_file_does_not_exist"},
			wantCode: codes.NotFound,
		},
	}

	testHandler := func(t *testing.T, tc testCase) {
		// Generate Random Content
		data := make([]byte, tc.fileSize)
		_, err := rand.Read(data)
		require.NoError(t, err)

		// Create Asset
		a := graph.Asset.Create().
			SetName(tc.fileName).
			SetContent(data).
			SaveX(ctx)

		// Send Request
		fileClient, err := client.FetchAsset(ctx, tc.req)
		require.NoError(t, err)

		// Read All Chunks
		var buf bytes.Buffer
		for {
			// Receive Chunk
			resp, err := fileClient.Recv()
			if errors.Is(err, io.EOF) {
				break
			}

			// Check Status
			require.Equal(t, tc.wantCode.String(), status.Code(err).String())
			if status.Code(err) != codes.OK {
				// Do not continue if we expected error code
				return
			}

			// Write Chunk
			_, err = buf.Write(resp.Chunk)
			require.NoError(t, err)
		}

		// Assert Content
		assert.Equal(t, a.Content, buf.Bytes())

		// Assert Headers
		metadata, err := fileClient.Header()
		require.NoError(t, err)
		require.Len(t, metadata.Get("sha3-256-checksum"), 1)
		assert.Equal(t, a.Hash, metadata.Get("sha3-256-checksum")[0])
		require.Len(t, metadata.Get("file-size"), 1)
		assert.Equal(t, fmt.Sprintf("%d", a.Size), metadata.Get("file-size")[0])
	}

	// Run Tests
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			testHandler(t, tc)
		})
	}
}
