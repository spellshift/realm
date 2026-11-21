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
	"realm.pub/tavern/internal/ent/beacon"
)

func TestScheduledClaimAndFetchAssetUseLiveTome(t *testing.T) {
	ctx := context.Background()
	client, graph, closeFn, _ := c2test.New(t)
	defer closeFn()
	_, err := client.ClaimTasks(ctx, newClaimTasksReq("live-beacon", "live-host", "host"))
	require.NoError(t, err)
	b := graph.Beacon.Query().Where(beacon.Identifier("live-beacon")).OnlyX(ctx)
	asset := graph.Asset.Create().SetName("live.txt").SetContent([]byte("original bytes")).SaveX(ctx)
	removed := graph.Asset.Create().SetName("removed.txt").SetContent([]byte("removed bytes")).SaveX(ctx)
	tome := graph.Tome.Create().SetName("live-tome").SetDescription("test").
		SetAuthor("test").SetEldritch("print('original')").AddAssets(asset, removed).SaveX(ctx)
	st := graph.ScheduledTask.Create().SetName("live-schedule").SetDescription("test").SetTome(tome).SaveX(ctx)
	q := graph.Quest.Create().SetName("live quest").SetTome(tome).SetScheduledTask(st).
		SetEldritchAtCreation(tome.Eldritch).SaveX(ctx)
	task := graph.Task.Create().SetQuest(q).SetBeacon(b).SaveX(ctx)

	graph.Tome.UpdateOne(tome).SetEldritch("print('updated')").ExecX(ctx)
	graph.Asset.UpdateOne(asset).SetContent([]byte("updated bytes")).ExecX(ctx)
	graph.Asset.DeleteOne(removed).ExecX(ctx)
	added := graph.Asset.Create().SetName("added.txt").SetContent([]byte("added bytes")).SaveX(ctx)
	graph.Tome.UpdateOne(tome).AddAssets(added).ExecX(ctx)

	resp, err := client.ClaimTasks(ctx, newClaimTasksReq("live-beacon", "live-host", "host"))
	require.NoError(t, err)
	require.Len(t, resp.Tasks, 1)
	require.Equal(t, task.ID, int(resp.Tasks[0].Id))
	require.Equal(t, "print('updated')", resp.Tasks[0].Tome.Eldritch)
	require.ElementsMatch(t, []string{"live.txt", "added.txt"}, resp.Tasks[0].Tome.FileNames)
	require.NotContains(t, resp.Tasks[0].Tome.FileNames, "removed.txt")

	// FetchAsset requires a JWT context, but asset lookup uses only the name.
	fetch := func(name string) (c2pb.C2_FetchAssetClient, error) {
		return client.FetchAsset(ctx, &c2pb.FetchAssetRequest{
			Name: name, Context: &c2pb.FetchAssetRequest_ShellTaskContext{
				ShellTaskContext: &c2pb.ShellTaskContext{Jwt: resp.Tasks[0].Jwt},
			},
		})
	}
	stream, err := fetch("live.txt")
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
	require.Equal(t, "updated bytes", got.String())
	headers, err := stream.Header()
	require.NoError(t, err)
	liveAsset := graph.Asset.GetX(ctx, asset.ID)
	require.Equal(t, []string{"13"}, headers.Get("file-size"))
	require.Equal(t, []string{liveAsset.Hash}, headers.Get("sha1-checksum"))

	addedStream, err := fetch("added.txt")
	require.NoError(t, err)
	got.Reset()
	for {
		chunk, err := addedStream.Recv()
		if err == io.EOF {
			break
		}
		require.NoError(t, err)
		got.Write(chunk.Chunk)
	}
	require.Equal(t, "added bytes", got.String())

	removedStream, err := fetch("removed.txt")
	require.NoError(t, err)
	_, err = removedStream.Recv()
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
	q := graph.Quest.Create().SetName("claim quest").SetTome(tome).SetScheduledTask(st).
		SetEldritchAtCreation(tome.Eldritch).SaveX(ctx)
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
