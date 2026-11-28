package builder

import (
	"context"
	"path/filepath"
	"sync"
	"testing"
	"time"

	"github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
	"realm.pub/tavern/internal/ent/buildprofile"
	"realm.pub/tavern/internal/ent/buildtask"
	"realm.pub/tavern/internal/logging"
)

type autoBuildAuditRecorder struct {
	entries []logging.AuditEntry
}

func (r *autoBuildAuditRecorder) Enqueue(entry logging.AuditEntry) {
	r.entries = append(r.entries, entry)
}

func autoQueueFixture(t *testing.T, targets ...c2pb.Host_Platform) (*ent.Client, *ent.Builder, *Server) {
	t.Helper()
	graph, err := ent.Open("sqlite3", "file:"+filepath.Join(t.TempDir(), "auto-build.db")+"?_journal=WAL&_busy_timeout=10000&_fk=1")
	require.NoError(t, err)
	t.Cleanup(func() { graph.Close() })
	require.NoError(t, graph.Schema.Create(context.Background()))
	graph.BuildProfile.Create().SetName(DefaultBuildProfileName).SetDescription("default build profile").
		SetBuildScript("{{.BuildCommand}}").SetArtifactPath("{{.ArtifactPath}}").SaveX(context.Background())
	worker := graph.Builder.Create().SetSupportedTargets(targets).SaveX(context.Background())
	return graph, worker, New(graph, "test-key")
}

func pollAs(t *testing.T, server *Server, worker *ent.Builder) []*builderpb.BuildTaskSpec {
	t.Helper()
	ctx := context.WithValue(context.Background(), builderContextKey{}, worker)
	response, err := server.ClaimBuildTasks(ctx, &builderpb.ClaimBuildTasksRequest{})
	require.NoError(t, err)
	return response.Tasks
}

func TestAutomaticDefaultBuildsFirstRepeatAndFormats(t *testing.T) {
	t.Setenv("DISABLE_AUTO_AGENT_BUILDS", "0")
	graph, worker, server := autoQueueFixture(t, c2pb.Host_PLATFORM_LINUX, c2pb.Host_PLATFORM_WINDOWS, c2pb.Host_PLATFORM_MACOS)
	ctx := context.Background()
	first := pollAs(t, server, worker)
	require.Len(t, first, 5)
	require.Empty(t, pollAs(t, server, worker))
	profile := graph.BuildProfile.Query().OnlyX(ctx)
	graph.BuildProfile.UpdateOne(profile).SetBuildImage("updated:image").SaveX(ctx)
	require.Empty(t, pollAs(t, server, worker))
	tasks := graph.BuildTask.Query().AllX(ctx)
	require.Len(t, tasks, 5)
	paths := map[builderpb.TargetFormat]string{
		builderpb.TargetFormat_TARGET_FORMAT_BIN:             "imix.exe",
		builderpb.TargetFormat_TARGET_FORMAT_CDYLIB:          "imix.dll",
		builderpb.TargetFormat_TARGET_FORMAT_WINDOWS_SERVICE: "imix.exe",
	}
	for _, task := range tasks {
		require.Equal(t, worker.ID, task.QueryBuilder().OnlyIDX(ctx))
		require.NotNil(t, task.AutoBuildKey)
		require.NotNil(t, task.ProfileAtCreation)
		require.NotZero(t, task.ClaimedAt)
		require.NotNil(t, task.QueryArtifact().OnlyX(ctx))
		if task.TargetOs == c2pb.Host_PLATFORM_WINDOWS {
			require.Equal(t, DockerBuildDir+"/realm/implants/target/x86_64-pc-windows-gnu/release/"+paths[task.TargetFormat], task.ArtifactPath)
			require.Equal(t, task.ArtifactPath, task.ProfileAtCreation.ArtifactPath)
		}
		if task.TargetOs == c2pb.Host_PLATFORM_LINUX {
			require.Equal(t, DockerBuildDir+"/realm/implants/target/x86_64-unknown-linux-musl/release/imix", task.ArtifactPath)
		}
		if task.TargetOs == c2pb.Host_PLATFORM_MACOS {
			require.Equal(t, DockerBuildDir+"/realm/implants/target/aarch64-apple-darwin/release/imix", task.ArtifactPath)
		}
	}
}

func TestAutomaticDefaultBuildsPriorTasksAndOtherBuilder(t *testing.T) {
	t.Setenv("DISABLE_AUTO_AGENT_BUILDS", "0")
	graph, first, server := autoQueueFixture(t, c2pb.Host_PLATFORM_WINDOWS)
	ctx := context.Background()
	second := graph.Builder.Create().SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_WINDOWS}).SaveX(ctx)
	prior, err := CreateDefaultBuildTask(ctx, graph, first.ID, c2pb.Host_PLATFORM_WINDOWS, builderpb.TargetFormat_TARGET_FORMAT_BIN)
	require.NoError(t, err)
	graph.BuildTask.UpdateOne(prior).SetClaimedAt(time.Now()).SaveX(ctx)
	// A failed task still counts and remains assigned to its original builder.
	graph.BuildTask.UpdateOne(prior).SetError("failed").SetExitCode(1).SetFinishedAt(time.Now()).SaveX(ctx)
	claimed := pollAs(t, server, second)
	require.Len(t, claimed, 2)
	require.Len(t, graph.BuildTask.Query().AllX(ctx), 3)
	require.Equal(t, first.ID, graph.BuildTask.GetX(ctx, prior.ID).QueryBuilder().OnlyIDX(ctx))
	require.Empty(t, pollAs(t, server, first))
	require.Empty(t, pollAs(t, server, second))
}

func TestAutomaticDefaultBuildsKeepsOfflineAssignment(t *testing.T) {
	t.Setenv("DISABLE_AUTO_AGENT_BUILDS", "0")
	graph, first, server := autoQueueFixture(t, c2pb.Host_PLATFORM_LINUX)
	ctx := context.Background()
	second := graph.Builder.Create().SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_LINUX}).SaveX(ctx)
	prior, err := CreateDefaultBuildTask(ctx, graph, first.ID, c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN)
	require.NoError(t, err)
	// The first builder has never polled and is offline. The second must not
	// claim or reassign its pending task.
	require.Empty(t, pollAs(t, server, second))
	require.Zero(t, graph.BuildTask.GetX(ctx, prior.ID).ClaimedAt)
	require.Equal(t, first.ID, graph.BuildTask.GetX(ctx, prior.ID).QueryBuilder().OnlyIDX(ctx))
	require.Len(t, pollAs(t, server, first), 1)
	require.Equal(t, 1, graph.BuildTask.Query().CountX(ctx))
}

func TestAutomaticDefaultBuildsLegacyDuplicateProfiles(t *testing.T) {
	t.Setenv("DISABLE_AUTO_AGENT_BUILDS", "0")
	graph, worker, server := autoQueueFixture(t, c2pb.Host_PLATFORM_LINUX)
	ctx := context.Background()
	first := graph.BuildProfile.Query().Order(ent.Asc(buildprofile.FieldID)).FirstX(ctx)
	second := graph.BuildProfile.Create().SetName(DefaultBuildProfileName).SetDescription("legacy duplicate").SaveX(ctx)
	format := builderpb.TargetFormat_TARGET_FORMAT_BIN
	prior, err := CreateBuildTask(ctx, graph, CreateBuildTaskInput{
		ProfileID: second.ID, TargetOS: c2pb.Host_PLATFORM_LINUX, TargetFormat: &format,
	}, &worker.ID)
	require.NoError(t, err)
	require.Len(t, pollAs(t, server, worker), 1)
	require.Equal(t, 1, graph.BuildTask.Query().CountX(ctx))
	require.Equal(t, prior.ID, graph.BuildTask.Query().OnlyIDX(ctx))
	created, err := CreateDefaultBuildTask(ctx, graph, worker.ID, c2pb.Host_PLATFORM_LINUX, format)
	require.NoError(t, err)
	require.Equal(t, first.ID, created.QueryProfile().OnlyIDX(ctx))
}

func TestAutomaticDefaultBuildAuditActor(t *testing.T) {
	t.Setenv("DISABLE_AUTO_AGENT_BUILDS", "0")
	_, worker, server := autoQueueFixture(t, c2pb.Host_PLATFORM_LINUX)
	recorder := &autoBuildAuditRecorder{}
	logging.SetAuditPersister(recorder)
	t.Cleanup(func() { logging.SetAuditPersister(nil) })
	require.Len(t, pollAs(t, server, worker), 1)
	var created []logging.AuditEntry
	for _, entry := range recorder.entries {
		if entry.Action == "build_task.created" {
			created = append(created, entry)
		}
	}
	require.Len(t, created, 1)
	require.Equal(t, logging.ActorBuilder, created[0].Actor.Kind)
	require.Equal(t, worker.Identifier, created[0].Actor.Name)
}

func TestRetryAutoBuildOnSQLiteLock(t *testing.T) {
	attempts := 0
	err := retryAutoBuildOnLock(context.Background(), func() error {
		attempts++
		if attempts < 3 {
			return sqlite3.Error{Code: sqlite3.ErrBusy}
		}
		return nil
	})
	require.NoError(t, err)
	require.Equal(t, 3, attempts)
}

func TestAutomaticDefaultBuildsDisabledAndFailure(t *testing.T) {
	t.Setenv("DISABLE_AUTO_AGENT_BUILDS", "1")
	graph, worker, server := autoQueueFixture(t, c2pb.Host_PLATFORM_LINUX)
	ctx := context.Background()
	require.Empty(t, pollAs(t, server, worker))
	require.Zero(t, graph.BuildTask.Query().CountX(ctx))
	// Queue failure must not prevent claiming a task already assigned to the builder.
	prior, err := CreateDefaultBuildTask(ctx, graph, worker.ID, c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN)
	require.NoError(t, err)
	t.Setenv("DISABLE_AUTO_AGENT_BUILDS", "0")
	// Another supported format is missing; force its creation to fail.
	worker = graph.Builder.UpdateOne(worker).SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_LINUX, c2pb.Host_PLATFORM_WINDOWS}).SaveX(ctx)
	profile := graph.BuildProfile.Query().OnlyX(ctx)
	graph.BuildProfile.UpdateOne(profile).SetBuildScript("{{.Missing}}").SaveX(ctx)
	require.Equal(t, int64(prior.ID), pollAs(t, server, worker)[0].Id)
	require.Equal(t, 1, graph.BuildTask.Query().CountX(ctx))
	graph.BuildProfile.UpdateOne(profile).SetBuildScript("{{.BuildCommand}}").SaveX(ctx)
	require.Len(t, pollAs(t, server, worker), 3)
	require.Equal(t, 4, graph.BuildTask.Query().CountX(ctx))
}

func TestAutomaticDefaultBuildsConcurrentBuilders(t *testing.T) {
	t.Setenv("DISABLE_AUTO_AGENT_BUILDS", "0")
	graph, first, server := autoQueueFixture(t, c2pb.Host_PLATFORM_LINUX)
	ctx := context.Background()
	second := graph.Builder.Create().SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_LINUX}).SaveX(ctx)
	start := make(chan struct{})
	errors := make(chan error, 2)
	var wg sync.WaitGroup
	for _, worker := range []*ent.Builder{first, second} {
		wg.Add(1)
		go func(worker *ent.Builder) {
			defer wg.Done()
			<-start
			_, err := server.ClaimBuildTasks(context.WithValue(ctx, builderContextKey{}, worker), &builderpb.ClaimBuildTasksRequest{})
			errors <- err
		}(worker)
	}
	close(start)
	wg.Wait()
	close(errors)
	for err := range errors {
		require.NoError(t, err)
	}
	require.Equal(t, 1, graph.BuildTask.Query().Where(buildtask.TargetOsEQ(c2pb.Host_PLATFORM_LINUX)).CountX(ctx))
	created := graph.BuildTask.Query().OnlyX(ctx)
	require.NotNil(t, created.AutoBuildKey)
	assetCount := graph.Asset.Query().Where(asset.StatusEQ(asset.StatusIN_PROGRESS)).CountX(ctx)
	format := builderpb.TargetFormat_TARGET_FORMAT_BIN
	_, err := CreateBuildTask(ctx, graph, CreateBuildTaskInput{
		ProfileID: created.QueryProfile().OnlyIDX(ctx), TargetOS: c2pb.Host_PLATFORM_LINUX,
		TargetFormat: &format, AutoBuildKey: created.AutoBuildKey,
	}, &first.ID)
	require.Error(t, err)
	require.Equal(t, 1, graph.BuildTask.Query().CountX(ctx))
	require.Equal(t, assetCount, graph.Asset.Query().Where(asset.StatusEQ(asset.StatusIN_PROGRESS)).CountX(ctx))
}
