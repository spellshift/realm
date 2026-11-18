package c2_test

import (
	"bytes"
	"context"
	"io"
	"sync"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/c2test"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/tomebundle"
)

func TestScheduledFetchAssetUsesQuestSnapshot(t *testing.T) {
	ctx := context.Background()
	client, graph, closeFn, mintJWT := c2test.New(t)
	defer closeFn()
	_, err := client.ClaimTasks(ctx, newClaimTasksReq("snapshot-beacon", "snapshot-host", "host"))
	require.NoError(t, err)
	b := graph.Beacon.Query().Where(beacon.Identifier("snapshot-beacon")).OnlyX(ctx)
	source := graph.Asset.Create().SetName("snapshot.txt").SetContent([]byte("original bytes")).SaveX(ctx)
	tome := graph.Tome.Create().SetName("fetch-snapshot-tome").SetDescription("test").
		SetAuthor("test").SetEldritch("print('snapshot')").AddAssets(source).SaveX(ctx)
	st := graph.ScheduledTask.Create().SetName("fetch-snapshot-schedule").SetDescription("test").SetTome(tome).SaveX(ctx)
	bundle, err := tomebundle.Create(ctx, graph, []*ent.Asset{source})
	require.NoError(t, err)
	q := graph.Quest.Create().SetName("snapshot quest").SetTome(tome).SetScheduledTask(st).
		SetEldritchAtCreation(tome.Eldritch).SetBundle(bundle).SaveX(ctx)
	task := graph.Task.Create().SetQuest(q).SetBeacon(b).SaveX(ctx)

	graph.Asset.UpdateOne(source).SetContent([]byte("changed bytes")).ExecX(ctx)
	graph.Asset.Create().SetName("unrelated.txt").SetContent([]byte("unrelated")).SaveX(ctx)
	resp, err := client.ClaimTasks(ctx, newClaimTasksReq("snapshot-beacon", "snapshot-host", "host"))
	require.NoError(t, err)
	require.Len(t, resp.Tasks, 1)
	require.Equal(t, []string{"snapshot.txt"}, resp.Tasks[0].Tome.FileNames)

	stream, err := client.FetchAsset(ctx, &c2pb.FetchAssetRequest{
		Name: "snapshot.txt", Context: &c2pb.FetchAssetRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(task.ID), Jwt: resp.Tasks[0].Jwt},
		},
	})
	require.NoError(t, err)
	var got bytes.Buffer
	for {
		chunk, err := stream.Recv()
		if err == io.EOF {
			break
		}
		require.NoError(t, err)
		got.Write(chunk.Chunk)
	}
	require.Equal(t, "original bytes", got.String())
	headers, err := stream.Header()
	require.NoError(t, err)
	require.Equal(t, []string{"14"}, headers.Get("file-size"))
	require.Equal(t, []string{source.Hash}, headers.Get("sha1-checksum"))

	missing, err := client.FetchAsset(ctx, &c2pb.FetchAssetRequest{
		Name: "unrelated.txt", Context: &c2pb.FetchAssetRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(task.ID), Jwt: resp.Tasks[0].Jwt},
		},
	})
	require.NoError(t, err)
	_, err = missing.Recv()
	require.Equal(t, codes.NotFound, status.Code(err))

	other := graph.Beacon.Create().SetIdentifier("other-beacon").SetHost(b.QueryHost().OnlyX(ctx)).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)
	foreign, err := client.FetchAsset(ctx, &c2pb.FetchAssetRequest{
		Name: "snapshot.txt", Context: &c2pb.FetchAssetRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(task.ID), Jwt: mintJWT(other.ID)},
		},
	})
	require.NoError(t, err)
	_, err = foreign.Recv()
	require.Equal(t, codes.NotFound, status.Code(err))
}

func TestConcurrentClaimTasksDispatchesScheduledTaskOnce(t *testing.T) {
	ctx := context.Background()
	client, graph, closeFn, _ := c2test.New(t)
	defer closeFn()
	_, err := client.ClaimTasks(ctx, newClaimTasksReq("claim-beacon", "claim-host", "host"))
	require.NoError(t, err)
	b := graph.Beacon.Query().Where(beacon.Identifier("claim-beacon")).OnlyX(ctx)
	tome := graph.Tome.Create().SetName("claim-tome").SetDescription("test").
		SetAuthor("test").SetEldritch("print('claim')").SaveX(ctx)
	st := graph.ScheduledTask.Create().SetName("claim-schedule").SetDescription("test").SetTome(tome).SaveX(ctx)
	bundle, err := tomebundle.Create(ctx, graph, nil)
	require.NoError(t, err)
	q := graph.Quest.Create().SetName("claim quest").SetTome(tome).SetScheduledTask(st).
		SetEldritchAtCreation(tome.Eldritch).SetBundle(bundle).SaveX(ctx)
	graph.Task.Create().SetQuest(q).SetBeacon(b).SaveX(ctx)

	responses := make([]*c2pb.ClaimTasksResponse, 2)
	errs := make([]error, 2)
	var wg sync.WaitGroup
	for i := range responses {
		wg.Add(1)
		go func(i int) {
			defer wg.Done()
			responses[i], errs[i] = client.ClaimTasks(ctx, newClaimTasksReq("claim-beacon", "claim-host", "host"))
		}(i)
	}
	wg.Wait()
	for _, err := range errs {
		require.NoError(t, err)
	}
	require.Equal(t, 1, len(responses[0].Tasks)+len(responses[1].Tasks))
}

func TestScheduledClaimPreservesEmptyScriptSnapshot(t *testing.T) {
	ctx := context.Background()
	client, graph, closeFn, _ := c2test.New(t)
	defer closeFn()
	_, err := client.ClaimTasks(ctx, newClaimTasksReq("empty-script-beacon", "empty-script-host", "host"))
	require.NoError(t, err)
	b := graph.Beacon.Query().Where(beacon.Identifier("empty-script-beacon")).OnlyX(ctx)
	tome := graph.Tome.Create().SetName("empty-script-tome").SetDescription("test").
		SetAuthor("test").SetEldritch("print('live')").SaveX(ctx)
	st := graph.ScheduledTask.Create().SetName("empty-script-schedule").SetDescription("test").SetTome(tome).SaveX(ctx)
	bundle, err := tomebundle.Create(ctx, graph, nil)
	require.NoError(t, err)
	q := graph.Quest.Create().SetName("empty script quest").SetTome(tome).SetScheduledTask(st).
		SetEldritchAtCreation("").SetHasScheduledSnapshot(true).SetBundle(bundle).SaveX(ctx)
	graph.Task.Create().SetQuest(q).SetBeacon(b).SaveX(ctx)

	response, err := client.ClaimTasks(ctx, newClaimTasksReq("empty-script-beacon", "empty-script-host", "host"))
	require.NoError(t, err)
	require.Len(t, response.Tasks, 1)
	require.Empty(t, response.Tasks[0].Tome.Eldritch)

	// A Quest created before the marker migration still has a snapshot if it has a bundle.
	preMarker := graph.Quest.Create().SetName("pre-marker empty script quest").SetTome(tome).
		SetScheduledTask(st).SetEldritchAtCreation("").SetBundle(bundle).SaveX(ctx)
	graph.Task.Create().SetQuest(preMarker).SetBeacon(b).SaveX(ctx)
	response, err = client.ClaimTasks(ctx, newClaimTasksReq("empty-script-beacon", "empty-script-host", "host"))
	require.NoError(t, err)
	require.Len(t, response.Tasks, 1)
	require.Empty(t, response.Tasks[0].Tome.Eldritch)
}

func TestScheduledSnapshotMissingBundleFailsClosed(t *testing.T) {
	ctx := context.Background()
	client, graph, closeFn, mintJWT := c2test.New(t)
	defer closeFn()
	_, err := client.ClaimTasks(ctx, newClaimTasksReq("missing-bundle-beacon", "missing-bundle-host", "host"))
	require.NoError(t, err)
	b := graph.Beacon.Query().Where(beacon.Identifier("missing-bundle-beacon")).OnlyX(ctx)
	source := graph.Asset.Create().SetName("missing-bundle.txt").SetContent([]byte("live bytes")).SaveX(ctx)
	tome := graph.Tome.Create().SetName("missing-bundle-tome").SetDescription("test").
		SetAuthor("test").SetEldritch("print('live')").AddAssets(source).SaveX(ctx)
	st := graph.ScheduledTask.Create().SetName("missing-bundle-schedule").SetDescription("test").SetTome(tome).SaveX(ctx)
	bundle, err := tomebundle.Create(ctx, graph, []*ent.Asset{source})
	require.NoError(t, err)
	q := graph.Quest.Create().SetName("missing bundle quest").SetTome(tome).SetScheduledTask(st).
		SetEldritchAtCreation(tome.Eldritch).SetHasScheduledSnapshot(true).SetBundle(bundle).SaveX(ctx)
	task := graph.Task.Create().SetQuest(q).SetBeacon(b).SaveX(ctx)
	graph.Asset.DeleteOne(bundle).ExecX(ctx) // Simulate a bundle removed outside deleteAsset.

	response, err := client.ClaimTasks(ctx, newClaimTasksReq("missing-bundle-beacon", "missing-bundle-host", "host"))
	require.Nil(t, response)
	require.Equal(t, codes.FailedPrecondition, status.Code(err))

	stream, err := client.FetchAsset(ctx, &c2pb.FetchAssetRequest{
		Name: "missing-bundle.txt", Context: &c2pb.FetchAssetRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(task.ID), Jwt: mintJWT(b.ID)},
		},
	})
	require.NoError(t, err)
	_, err = stream.Recv()
	require.Equal(t, codes.FailedPrecondition, status.Code(err))
}
