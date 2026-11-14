package c2_test

import (
	"bytes"
	"context"
	"image"
	"image/color"
	"image/draw"
	"image/png"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/c2test"
	"realm.pub/tavern/internal/c2/epb"
)

// newTestScreenshotPNG builds a valid, solid-color PNG of the given dimensions.
func newTestScreenshotPNG(t *testing.T, width, height int) []byte {
	t.Helper()
	img := image.NewRGBA(image.Rect(0, 0, width, height))
	draw.Draw(img, img.Bounds(), &image.Uniform{C: color.RGBA{10, 20, 30, 255}}, image.Point{}, draw.Src)

	var buf bytes.Buffer
	require.NoError(t, png.Encode(&buf, img))
	return buf.Bytes()
}

// TestReportFile_Screenshot asserts that reporting a screenshot via ReportFile generates and
// persists a downscaled thumbnail alongside the full resolution content.
func TestReportFile_Screenshot(t *testing.T) {
	client, graph, close, mintJWT := c2test.New(t)
	defer close()
	ctx := context.Background()

	existingBeacon := c2test.NewRandomBeacon(ctx, graph)
	existingHost := existingBeacon.QueryHost().OnlyX(ctx)
	existingTask := c2test.NewRandomAssignedTask(ctx, graph, existingBeacon.Identifier)
	token := mintJWT(existingBeacon.ID)

	t.Run("ValidImage_GeneratesThumbnail", func(t *testing.T) {
		content := newTestScreenshotPNG(t, 1920, 1080)

		rClient, err := client.ReportFile(ctx)
		require.NoError(t, err)
		require.NoError(t, rClient.Send(&c2pb.ReportFileRequest{
			Context: &c2pb.ReportFileRequest_TaskContext{
				TaskContext: &c2pb.TaskContext{TaskId: int64(existingTask.ID), Jwt: token},
			},
			Kind: c2pb.ReportFileKind_REPORT_FILE_KIND_SCREENSHOT,
			Chunk: &epb.File{
				Metadata: &epb.FileMetadata{Path: "screenshot_1.png"},
				Chunk:    content,
			},
		}))
		resp, err := rClient.CloseAndRecv()
		st, _ := status.FromError(err)
		require.Equal(t, codes.OK.String(), st.Code().String(), err)
		require.NotNil(t, resp)

		screenshots := existingHost.QueryScreenshots().AllX(ctx)
		require.Len(t, screenshots, 1)

		saved := screenshots[0]
		assert.Equal(t, content, saved.Content)
		require.NotEmpty(t, saved.Thumbnail)
		assert.NotEqual(t, saved.Content, saved.Thumbnail)

		thumb, _, err := image.Decode(bytes.NewReader(saved.Thumbnail))
		require.NoError(t, err)
		bounds := thumb.Bounds()
		assert.LessOrEqual(t, bounds.Dx(), 320)
		assert.LessOrEqual(t, bounds.Dy(), 240)
	})

	t.Run("InvalidImage_SkipsThumbnail", func(t *testing.T) {
		rClient, err := client.ReportFile(ctx)
		require.NoError(t, err)
		require.NoError(t, rClient.Send(&c2pb.ReportFileRequest{
			Context: &c2pb.ReportFileRequest_TaskContext{
				TaskContext: &c2pb.TaskContext{TaskId: int64(existingTask.ID), Jwt: token},
			},
			Kind: c2pb.ReportFileKind_REPORT_FILE_KIND_SCREENSHOT,
			Chunk: &epb.File{
				Metadata: &epb.FileMetadata{Path: "screenshot_invalid.png"},
				Chunk:    []byte("not a real image"),
			},
		}))
		resp, err := rClient.CloseAndRecv()
		st, _ := status.FromError(err)
		require.Equal(t, codes.OK.String(), st.Code().String(), err)
		require.NotNil(t, resp)

		saved := existingHost.QueryScreenshots().AllX(ctx)
		var found bool
		for _, s := range saved {
			if s.Name == "screenshot_invalid.png" {
				found = true
				assert.Empty(t, s.Thumbnail)
			}
		}
		assert.True(t, found, "expected to find saved screenshot with invalid image content")
	})
}
