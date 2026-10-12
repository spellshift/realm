package c2_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/c2test"
	"realm.pub/tavern/internal/c2/epb"
)

// TestCrossBeaconIDOR ensures a JWT minted for one beacon cannot be used to
// read/write another beacon's tasks (regression test for cross-beacon IDOR).
func TestCrossBeaconIDOR(t *testing.T) {
	client, graph, close, mintJWT := c2test.New(t)
	defer close()
	ctx := context.Background()

	beaconA := c2test.NewRandomBeacon(ctx, graph)
	beaconB := c2test.NewRandomBeacon(ctx, graph)
	taskA := c2test.NewRandomAssignedTask(ctx, graph, beaconA.Identifier)
	tokenB := mintJWT(beaconB.ID)

	// 1. ReportOutput with another beacon's JWT must be rejected
	_, err := client.ReportOutput(ctx, &c2pb.ReportOutputRequest{
		Message: &c2pb.ReportOutputRequest_TaskOutput{
			TaskOutput: &c2pb.ReportTaskOutputMessage{
				Context: &c2pb.TaskContext{TaskId: int64(taskA.ID), Jwt: tokenB},
				Output:  &c2pb.TaskOutput{Id: int64(taskA.ID), Output: "pwned"},
			},
		},
	})
	require.Equal(t, codes.PermissionDenied, status.Code(err), "cross-beacon ReportOutput should be denied: %v", err)

	// 2. ReportCredential with another beacon's JWT must be rejected
	_, err = client.ReportCredential(ctx, &c2pb.ReportCredentialRequest{
		Context: &c2pb.ReportCredentialRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(taskA.ID), Jwt: tokenB},
		},
		Credential: &epb.Credential{Principal: "root", Secret: "pwned", Kind: epb.Credential_KIND_PASSWORD},
	})
	require.Equal(t, codes.PermissionDenied, status.Code(err), "cross-beacon ReportCredential should be denied: %v", err)

	// 3. ReportProcessList with another beacon's JWT must be rejected
	_, err = client.ReportProcessList(ctx, &c2pb.ReportProcessListRequest{
		Context: &c2pb.ReportProcessListRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(taskA.ID), Jwt: tokenB},
		},
		List: &epb.ProcessList{List: []*epb.Process{{Pid: 1, Name: "pwned"}}},
	})
	require.Equal(t, codes.PermissionDenied, status.Code(err), "cross-beacon ReportProcessList should be denied: %v", err)

	// 4. ReportFile with another beacon's JWT must be rejected (validated before buffering)
	stream, err := client.ReportFile(ctx)
	require.NoError(t, err)
	require.NoError(t, stream.Send(&c2pb.ReportFileRequest{
		Context: &c2pb.ReportFileRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(taskA.ID), Jwt: tokenB},
		},
		Chunk: &epb.File{Metadata: &epb.FileMetadata{Path: "/pwned"}},
	}))
	_, err = stream.CloseAndRecv()
	require.Equal(t, codes.PermissionDenied, status.Code(err), "cross-beacon ReportFile should be denied: %v", err)

	// 5. FetchAsset referencing another beacon's task must be rejected
	asset := graph.Asset.Create().SetName("idor-asset").SetContent([]byte("secret")).SaveX(ctx)
	fetchStream, err := client.FetchAsset(ctx, &c2pb.FetchAssetRequest{
		Context: &c2pb.FetchAssetRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(taskA.ID), Jwt: tokenB},
		},
		Name: asset.Name,
	})
	require.NoError(t, err)
	_, err = fetchStream.Recv()
	require.Equal(t, codes.PermissionDenied, status.Code(err), "cross-beacon FetchAsset should be denied: %v", err)

	// 6. Own-beacon JWT still works (sanity check)
	tokenA := mintJWT(beaconA.ID)
	_, err = client.ReportOutput(ctx, &c2pb.ReportOutputRequest{
		Message: &c2pb.ReportOutputRequest_TaskOutput{
			TaskOutput: &c2pb.ReportTaskOutputMessage{
				Context: &c2pb.TaskContext{TaskId: int64(taskA.ID), Jwt: tokenA},
				Output:  &c2pb.TaskOutput{Id: int64(taskA.ID), Output: "legit"},
			},
		},
	})
	require.NoError(t, err, "own-beacon ReportOutput should succeed")
}
