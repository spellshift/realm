package builder_test

import (
	"context"
	"testing"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/asset"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestCreateDefaultBuildTaskWithExplicitBuilder(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	profile := graph.BuildProfile.Create().SetName(builder.DefaultBuildProfileName).
		SetDescription("seeded profile").SetBuildScript("{{.BuildCommand}}").
		SetArtifactPath("{{.ArtifactPath}}").SaveX(ctx)
	worker := graph.Builder.Create().SetSupportedTargets([]c2pb.Host_Platform{
		c2pb.Host_PLATFORM_LINUX, c2pb.Host_PLATFORM_WINDOWS,
	}).SaveX(ctx)
	assetInput := graph.Asset.Create().SetName("config.txt").SetContent([]byte("initial")).SaveX(ctx)
	tome := graph.Tome.Create().SetName("test-tome").SetDescription("test").SetAuthor("test").
		SetEldritch("print('initial')").AddAssets(assetInput).SaveX(ctx)
	graph.BuildProfile.UpdateOne(profile).SetTomes([]builderpb.BuildProfileTome{{TomeID: tome.ID}}).SaveX(ctx)

	// The selected builder has never checked in; explicit assignment must still work.
	task, err := builder.CreateDefaultBuildTask(ctx, graph, worker.ID, c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN)
	require.NoError(t, err)
	assert.Equal(t, worker.ID, task.QueryBuilder().OnlyIDX(ctx))
	assert.Equal(t, profile.ID, task.QueryProfile().OnlyIDX(ctx))
	assert.Equal(t, builderpb.TargetFormat_TARGET_FORMAT_BIN, task.TargetFormat)
	assert.Contains(t, task.BuildScript, "cargo build")
	assert.Contains(t, task.ArtifactPath, "x86_64-unknown-linux-musl")
	require.NotNil(t, task.ProfileAtCreation)
	assert.Equal(t, "Default", task.ProfileAtCreation.Name)
	require.Len(t, task.ProfileAtCreation.Tomes, 1)
	assert.Equal(t, "test-tome", task.ProfileAtCreation.Tomes[0].Name)
	bundle := task.QueryBundle().OnlyX(ctx)
	require.NotEmpty(t, bundle.Content)
	artifact := task.QueryArtifact().OnlyX(ctx)
	assert.Equal(t, asset.StatusIN_PROGRESS, artifact.Status)
	assert.Empty(t, artifact.Content)

	for _, tc := range []struct {
		format  builderpb.TargetFormat
		command string
	}{
		{builderpb.TargetFormat_TARGET_FORMAT_CDYLIB, "cargo build --release --lib"},
		{builderpb.TargetFormat_TARGET_FORMAT_WINDOWS_SERVICE, "cargo build --release --features win_service"},
	} {
		windowsTask, err := builder.CreateDefaultBuildTask(ctx, graph, worker.ID, c2pb.Host_PLATFORM_WINDOWS, tc.format)
		require.NoError(t, err)
		assert.Equal(t, tc.format, windowsTask.TargetFormat)
		assert.Contains(t, windowsTask.BuildScript, tc.command)
		assert.Equal(t, asset.StatusIN_PROGRESS, windowsTask.QueryArtifact().OnlyX(ctx).Status)
	}

	graph.BuildProfile.UpdateOne(profile).SetName("changed").SaveX(ctx)
	graph.Tome.UpdateOne(tome).SetName("changed-tome").SaveX(ctx)
	persisted := graph.BuildTask.GetX(ctx, task.ID)
	assert.Equal(t, "Default", persisted.ProfileAtCreation.Name)
	assert.Equal(t, "test-tome", persisted.ProfileAtCreation.Tomes[0].Name)
}

func TestCreateBuildTaskRollsBackWithExplicitBuilder(t *testing.T) {
	ctx := context.Background()
	graph, db := enttest.OpenTempDBWithDB(t)
	defer graph.Close()
	profile := graph.BuildProfile.Create().SetName(builder.DefaultBuildProfileName).
		SetDescription("seeded profile").SaveX(ctx)
	worker := graph.Builder.Create().SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_LINUX}).SaveX(ctx)
	tome := graph.Tome.Create().SetName("rollback-tome").SetDescription("test").SetAuthor("test").
		SetEldritch("print('rollback')").SaveX(ctx)
	graph.BuildProfile.UpdateOne(profile).SetTomes([]builderpb.BuildProfileTome{{TomeID: tome.ID}}).SaveX(ctx)
	_, err := db.ExecContext(ctx, `CREATE TRIGGER fail_build_task_insert BEFORE INSERT ON build_tasks
		BEGIN SELECT RAISE(ABORT, 'forced build task failure'); END`)
	require.NoError(t, err)
	assetsBefore := graph.Asset.Query().CountX(ctx)
	tasksBefore := graph.BuildTask.Query().CountX(ctx)
	_, err = builder.CreateDefaultBuildTask(ctx, graph, worker.ID, c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN)
	require.ErrorContains(t, err, "forced build task failure")
	assert.Equal(t, assetsBefore, graph.Asset.Query().CountX(ctx))
	assert.Equal(t, tasksBefore, graph.BuildTask.Query().CountX(ctx))
	assert.Equal(t, profile.ID, graph.BuildProfile.Query().OnlyIDX(ctx))
}
