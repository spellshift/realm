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
	require.Equal(t, codes.NotFound, status.Code(err), "cross-beacon ReportOutput should be denied: %v", err)

	// 2. ReportCredential with another beacon's JWT must be rejected
	_, err = client.ReportCredential(ctx, &c2pb.ReportCredentialRequest{
		Context: &c2pb.ReportCredentialRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(taskA.ID), Jwt: tokenB},
		},
		Credential: &epb.Credential{Principal: "root", Secret: "pwned", Kind: epb.Credential_KIND_PASSWORD},
	})
	require.Equal(t, codes.NotFound, status.Code(err), "cross-beacon ReportCredential should be denied: %v", err)

	// 3. ReportProcessList with another beacon's JWT must be rejected
	_, err = client.ReportProcessList(ctx, &c2pb.ReportProcessListRequest{
		Context: &c2pb.ReportProcessListRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(taskA.ID), Jwt: tokenB},
		},
		List: &epb.ProcessList{List: []*epb.Process{{Pid: 1, Name: "pwned"}}},
	})
	require.Equal(t, codes.NotFound, status.Code(err), "cross-beacon ReportProcessList should be denied: %v", err)

	// 4. ReportFile with another beacon's JWT must be rejected
	stream, err := client.ReportFile(ctx)
	require.NoError(t, err)
	require.NoError(t, stream.Send(&c2pb.ReportFileRequest{
		Context: &c2pb.ReportFileRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(taskA.ID), Jwt: tokenB},
		},
		Chunk: &epb.File{Metadata: &epb.FileMetadata{Path: "/pwned"}},
	}))
	_, err = stream.CloseAndRecv()
	require.Equal(t, codes.NotFound, status.Code(err), "cross-beacon ReportFile should be denied: %v", err)

	// Assets remain shared across beacons with a valid task JWT.
	asset := graph.Asset.Create().SetName("idor-asset").SetContent([]byte("secret")).SaveX(ctx)
	fetchStream, err := client.FetchAsset(ctx, &c2pb.FetchAssetRequest{
		Context: &c2pb.FetchAssetRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(taskA.ID), Jwt: tokenB},
		},
		Name: asset.Name,
	})
	require.NoError(t, err)
	chunk, err := fetchStream.Recv()
	require.NoError(t, err)
	require.Equal(t, asset.Content, chunk.Chunk)
}

// TestCrossBeaconIDORCreatePortal ensures a JWT minted for one beacon cannot
// open a portal on another beacon's task or shell task.
func TestCrossBeaconIDORCreatePortal(t *testing.T) {
	client, graph, close, mintJWT := c2test.New(t)
	defer close()
	ctx := context.Background()

	beaconA := c2test.NewRandomBeacon(ctx, graph)
	beaconB := c2test.NewRandomBeacon(ctx, graph)
	taskA := c2test.NewRandomAssignedTask(ctx, graph, beaconA.Identifier)
	tokenB := mintJWT(beaconB.ID)

	// 1. CreatePortal with a task context referencing another beacon's task
	portalStream, err := client.CreatePortal(ctx)
	require.NoError(t, err)
	require.NoError(t, portalStream.Send(&c2pb.CreatePortalRequest{
		Context: &c2pb.CreatePortalRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(taskA.ID), Jwt: tokenB},
		},
	}))
	_, err = portalStream.Recv()
	require.Error(t, err, "cross-beacon CreatePortal (task) should be denied")

	// 2. CreatePortal with a shell task context referencing another beacon's shell task
	userA := graph.User.Create().
		SetOauthID("idor-user-a").
		SetName("Idor User A").
		SetPhotoURL("http://example.com/a.jpg").
		SaveX(ctx)
	shellA := graph.Shell.Create().
		SetBeacon(beaconA).
		SetOwner(userA).
		SetData([]byte{}).
		SaveX(ctx)
	shellTaskA := graph.ShellTask.Create().
		SetShell(shellA).
		SetCreator(userA).
		SetInput("id").
		SetStreamID("idor-stream").
		SetSequenceID(1).
		SaveX(ctx)

	portalStream, err = client.CreatePortal(ctx)
	require.NoError(t, err)
	require.NoError(t, portalStream.Send(&c2pb.CreatePortalRequest{
		Context: &c2pb.CreatePortalRequest_ShellTaskContext{
			ShellTaskContext: &c2pb.ShellTaskContext{ShellTaskId: int64(shellTaskA.ID), Jwt: tokenB},
		},
	}))
	_, err = portalStream.Recv()
	require.Error(t, err, "cross-beacon CreatePortal (shell task) should be denied")

	// No portals may have been created
	portals, err := graph.Portal.Query().All(ctx)
	require.NoError(t, err)
	require.Empty(t, portals, "no portal should have been created for cross-beacon requests")
}

// TestCrossBeaconIDORShellContext ensures a JWT minted for one beacon cannot
// be used with a ShellTaskContext to read or write another beacon's shell
// tasks. Asset fetches remain shared across beacons.
func TestCrossBeaconIDORShellContext(t *testing.T) {
	client, graph, close, mintJWT := c2test.New(t)
	defer close()
	ctx := context.Background()

	beaconA := c2test.NewRandomBeacon(ctx, graph)
	beaconB := c2test.NewRandomBeacon(ctx, graph)
	tokenB := mintJWT(beaconB.ID)

	userA := graph.User.Create().
		SetOauthID("shell-ctx-user-a").
		SetName("Shell Ctx User A").
		SetPhotoURL("http://example.com/a.jpg").
		SaveX(ctx)
	shellA := graph.Shell.Create().
		SetBeacon(beaconA).
		SetOwner(userA).
		SetData([]byte{}).
		SaveX(ctx)
	shellTaskA := graph.ShellTask.Create().
		SetShell(shellA).
		SetCreator(userA).
		SetInput("id").
		SetStreamID("shell-ctx-stream").
		SetSequenceID(1).
		SaveX(ctx)
	shellTaskID := int64(shellTaskA.ID)

	// 1. ReportOutput via another beacon's shell task context must be rejected
	_, err := client.ReportOutput(ctx, &c2pb.ReportOutputRequest{
		Message: &c2pb.ReportOutputRequest_ShellTaskOutput{
			ShellTaskOutput: &c2pb.ReportShellTaskOutputMessage{
				Context: &c2pb.ShellTaskContext{ShellTaskId: shellTaskID, Jwt: tokenB},
				Output:  &c2pb.ShellTaskOutput{Id: shellTaskID, Output: "pwned"},
			},
		},
	})
	require.Equal(t, codes.NotFound, status.Code(err), "cross-beacon ReportOutput (shell task) should be denied: %v", err)

	// 2. ReportCredential via another beacon's shell task context must be rejected
	_, err = client.ReportCredential(ctx, &c2pb.ReportCredentialRequest{
		Context: &c2pb.ReportCredentialRequest_ShellTaskContext{
			ShellTaskContext: &c2pb.ShellTaskContext{ShellTaskId: shellTaskID, Jwt: tokenB},
		},
		Credential: &epb.Credential{Principal: "root", Secret: "pwned", Kind: epb.Credential_KIND_PASSWORD},
	})
	require.Equal(t, codes.NotFound, status.Code(err), "cross-beacon ReportCredential (shell task) should be denied: %v", err)

	// 3. ReportProcessList via another beacon's shell task context must be rejected
	_, err = client.ReportProcessList(ctx, &c2pb.ReportProcessListRequest{
		Context: &c2pb.ReportProcessListRequest_ShellTaskContext{
			ShellTaskContext: &c2pb.ShellTaskContext{ShellTaskId: shellTaskID, Jwt: tokenB},
		},
		List: &epb.ProcessList{List: []*epb.Process{{Pid: 1, Name: "pwned"}}},
	})
	require.Equal(t, codes.NotFound, status.Code(err), "cross-beacon ReportProcessList (shell task) should be denied: %v", err)

	// 4. ReportFile via another beacon's shell task context must be rejected
	stream, err := client.ReportFile(ctx)
	require.NoError(t, err)
	require.NoError(t, stream.Send(&c2pb.ReportFileRequest{
		Context: &c2pb.ReportFileRequest_ShellTaskContext{
			ShellTaskContext: &c2pb.ShellTaskContext{ShellTaskId: shellTaskID, Jwt: tokenB},
		},
		Chunk: &epb.File{Metadata: &epb.FileMetadata{Path: "/pwned"}},
	}))
	_, err = stream.CloseAndRecv()
	require.Equal(t, codes.NotFound, status.Code(err), "cross-beacon ReportFile (shell task) should be denied: %v", err)

	// Assets remain shared across beacons with a valid shell task JWT.
	asset := graph.Asset.Create().SetName("shell-asset").SetContent([]byte("shell-data")).SaveX(ctx)
	fetchStream, err := client.FetchAsset(ctx, &c2pb.FetchAssetRequest{
		Context: &c2pb.FetchAssetRequest_ShellTaskContext{
			ShellTaskContext: &c2pb.ShellTaskContext{ShellTaskId: shellTaskID, Jwt: tokenB},
		},
		Name: asset.Name,
	})
	require.NoError(t, err)
	chunk, err := fetchStream.Recv()
	require.NoError(t, err)
	require.Equal(t, asset.Content, chunk.Chunk)
}
