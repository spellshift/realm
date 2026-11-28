package builder

import (
	"context"
	"database/sql"
	"sync"
	"testing"
	"time"

	entgo "entgo.io/ent"
	entsql "entgo.io/ent/dialect/sql"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
)

// Tavern's default DSN uses SQLite shared-cache memory mode. Its one-connection
// pool is required because SQLITE_LOCKED does not honor SQLite's busy timeout.
func TestConcurrentPollsDefaultSQLiteDSN(t *testing.T) {
	t.Setenv("DISABLE_AUTO_AGENT_BUILDS", "0")
	db, err := sql.Open("sqlite3", "file:ent?mode=memory&cache=shared&_fk=1&_loc=UTC")
	require.NoError(t, err)
	db.SetMaxOpenConns(1)
	db.SetMaxIdleConns(1)
	t.Cleanup(func() { db.Close() })
	graph := ent.NewClient(ent.Driver(entsql.OpenDB("sqlite3", db)))
	require.NoError(t, graph.Schema.Create(context.Background()))
	ctx := context.Background()
	graph.BuildProfile.Create().SetName(DefaultBuildProfileName).SetDescription("default").SaveX(ctx)
	worker := graph.Builder.Create().SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_WINDOWS}).SaveX(ctx)
	server := New(graph, "test-key")

	start := make(chan struct{})
	type result struct {
		response *builderpb.ClaimBuildTasksResponse
		err      error
	}
	results := make(chan result, 8)
	for range 8 {
		go func() {
			<-start
			pollCtx := context.WithValue(ctx, builderContextKey{}, worker)
			response, err := server.ClaimBuildTasks(pollCtx, &builderpb.ClaimBuildTasksRequest{})
			results <- result{response, err}
		}()
	}
	close(start)
	seen := make(map[int64]bool)
	for range 8 {
		result := <-results
		require.NoError(t, result.err)
		for _, task := range result.response.Tasks {
			require.False(t, seen[task.Id], "task returned by two concurrent polls")
			seen[task.Id] = true
		}
	}
	require.Len(t, seen, 3)
	require.Equal(t, 3, graph.BuildTask.Query().CountX(ctx))
}

// Both polls must finish reading the same pending task before either can
// update it. This reproduces a real overlap without relying on scheduler luck.
func TestConcurrentPollsSameBuilderClaimOnce(t *testing.T) {
	t.Setenv("DISABLE_AUTO_AGENT_BUILDS", "1")
	graph, worker, server := autoQueueFixture(t, c2pb.Host_PLATFORM_LINUX)
	ctx := context.Background()
	task, err := CreateDefaultBuildTask(ctx, graph, worker.ID, c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN)
	require.NoError(t, err)

	var ready sync.WaitGroup
	ready.Add(2)
	release := make(chan struct{})
	graph.BuildTask.Intercept(entgo.InterceptFunc(func(next entgo.Querier) entgo.Querier {
		return entgo.QuerierFunc(func(ctx context.Context, query entgo.Query) (entgo.Value, error) {
			value, err := next.Query(ctx, query)
			if err == nil {
				if tasks, ok := value.([]*ent.BuildTask); ok && len(tasks) == 1 && tasks[0].ID == task.ID && tasks[0].ClaimedAt.IsZero() {
					ready.Done()
					<-release
				}
			}
			return value, err
		})
	}))

	type result struct {
		response *builderpb.ClaimBuildTasksResponse
		err      error
	}
	results := make(chan result, 2)
	for range 2 {
		go func() {
			pollCtx := context.WithValue(ctx, builderContextKey{}, worker)
			response, err := server.ClaimBuildTasks(pollCtx, &builderpb.ClaimBuildTasksRequest{})
			results <- result{response, err}
		}()
	}
	readDone := make(chan struct{})
	go func() { ready.Wait(); close(readDone) }()
	select {
	case <-readDone:
	case <-time.After(5 * time.Second):
		close(release)
		t.Fatal("polls did not both read the pending task")
	}
	close(release)
	claimed := 0
	for range 2 {
		result := <-results
		require.NoError(t, result.err)
		for _, spec := range result.response.Tasks {
			require.Equal(t, int64(task.ID), spec.Id)
			claimed++
		}
	}
	require.Equal(t, 1, claimed, "the same task was returned to overlapping polls")
	require.NotZero(t, graph.BuildTask.GetX(ctx, task.ID).ClaimedAt)
}
