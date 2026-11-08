package c2_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/c2test"
	"realm.pub/tavern/internal/c2/epb"
)

func TestReportScreenshotQueuesOCR(t *testing.T) {
	for _, enabled := range []bool{false, true} {
		t.Run(map[bool]string{false: "disabled", true: "enabled"}[enabled], func(t *testing.T) {
			var opts []c2.Option
			if enabled {
				opts = append(opts, c2.WithAIEnabled())
			}
			client, graph, close, token := c2test.New(t, opts...)
			defer close()
			ctx := context.Background()
			beacon := c2test.NewRandomBeacon(ctx, graph)
			task := c2test.NewRandomAssignedTask(ctx, graph, beacon.Identifier)
			stream, err := client.ReportFile(ctx)
			require.NoError(t, err)
			require.NoError(t, stream.Send(&c2pb.ReportFileRequest{
				Context: &c2pb.ReportFileRequest_TaskContext{TaskContext: &c2pb.TaskContext{TaskId: int64(task.ID), Jwt: token}},
				Kind:    c2pb.ReportFileKind_REPORT_FILE_KIND_SCREENSHOT,
				Chunk: &epb.File{
					Metadata: &epb.FileMetadata{Path: "screenshot.png"},
					Chunk:    []byte("image data"),
				},
			}))
			_, err = stream.CloseAndRecv()
			require.NoError(t, err)
			images := graph.Screenshot.Query().AllX(ctx)
			require.Len(t, images, 1)
			require.Equal(t, enabled, images[0].OcrRequested)
			require.Nil(t, images[0].OcrText)
		})
	}
}
