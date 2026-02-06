package builder_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/builder/buildertest"
)

// TestBuilderE2E_ClientConnectsToServer verifies that a builder client can
// connect to the builder gRPC server and call all three RPCs.
func TestBuilderE2E_ClientConnectsToServer(t *testing.T) {
	client, _, cleanup := buildertest.New(t)
	defer cleanup()

	ctx := context.Background()

	t.Run("BuildAgent", func(t *testing.T) {
		resp, err := client.BuildAgent(ctx, &builderpb.BuildAgentRequest{
			Name:        "test-agent",
			TargetOs:    "linux",
			TargetArch:  "amd64",
			CallbackUri: "https://example.com/c2",
		})

		// Expect Unimplemented since it's a stub
		require.Error(t, err)
		st, ok := status.FromError(err)
		require.True(t, ok, "expected gRPC status error")
		assert.Equal(t, codes.Unimplemented, st.Code())
		assert.Contains(t, st.Message(), "BuildAgent not yet implemented")
		assert.Nil(t, resp)
	})

	t.Run("ReportBuildStatus", func(t *testing.T) {
		resp, err := client.ReportBuildStatus(ctx, &builderpb.ReportBuildStatusRequest{
			BuildId: "build-123",
			Status:  builderpb.BuildStatus_BUILD_STATUS_IN_PROGRESS,
			Message: "compiling...",
		})

		// Expect Unimplemented since it's a stub
		require.Error(t, err)
		st, ok := status.FromError(err)
		require.True(t, ok, "expected gRPC status error")
		assert.Equal(t, codes.Unimplemented, st.Code())
		assert.Contains(t, st.Message(), "ReportBuildStatus not yet implemented")
		assert.Nil(t, resp)
	})

	t.Run("ReportBuildArtifact", func(t *testing.T) {
		stream, err := client.ReportBuildArtifact(ctx)
		require.NoError(t, err, "opening stream should succeed")

		// Send a chunk
		err = stream.Send(&builderpb.ReportBuildArtifactRequest{
			BuildId:      "build-123",
			ArtifactName: "agent.bin",
			Chunk:        []byte("fake binary data"),
		})
		require.NoError(t, err, "sending chunk should succeed")

		// Close and receive response
		resp, err := stream.CloseAndRecv()
		require.NoError(t, err, "close and recv should succeed for streaming stub")
		assert.NotEmpty(t, resp.GetSha3_256Checksum())
	})
}
