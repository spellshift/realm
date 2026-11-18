package builder_test

import (
	"context"
	"fmt"
	"strings"
	"testing"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/builder/executor"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestResolveBuildRecipe_BuildDir(t *testing.T) {
	profile := &ent.BuildProfile{
		BuildScript:  "cd {{.BuildDir}}/realm/implants/imix && {{.BuildCommand}}",
		ArtifactPath: "{{.ArtifactPath}}",
	}
	setupScript := "cd {{.BuildDir}}/realm/implants/imix && rm -rf install_scripts/* && cp -r $REALM_TOMES_DIR/* install_scripts/ 2>/dev/null || true"

	resolvedSetup, resolvedBuild, resolvedArtifact, err := builder.ResolveBuildRecipe(
		profile, setupScript, c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN,
	)
	require.NoError(t, err)

	const prefix = "cd "
	require.True(t, strings.HasPrefix(resolvedSetup, prefix))
	firstArg := strings.SplitN(strings.TrimPrefix(resolvedSetup, prefix), " ", 2)[0]
	buildDir := strings.TrimSuffix(firstArg, "/realm/implants/imix")
	require.NotEmpty(t, buildDir)

	assert.Contains(t, resolvedBuild, buildDir)
	assert.Contains(t, resolvedArtifact, buildDir)
	assert.Contains(t, resolvedArtifact, "x86_64-unknown-linux-musl")
}

func TestDefaultSetupScript_StagesTomesIntoExtractedAgentSource(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	asset := graph.Asset.Create().
		SetName("files/payload.txt").
		SetContent([]byte("staged-asset-data")).
		SaveX(ctx)
	tome := graph.Tome.Create().
		SetName("stage-test-tome").
		SetDescription("test tome staging").
		SetAuthor("test").
		SetEldritch("print('staged-eldritch')").
		AddAssets(asset).
		SaveX(ctx)

	tomeRefs := []builderpb.BuildProfileTome{{
		TomeID: tome.ID,
		Params: `{"interval":"10"}`,
	}}

	buildCheckScript := fmt.Sprintf(strings.Join([]string{
		"test -f Cargo.toml",
		"test -f {{.BuildDir}}/realm/implants/imix/Cargo.toml",
		"test ! -e {{.BuildDir}}/realm/implants/imix/install_scripts/persist_service",
		"test -f {{.BuildDir}}/realm/implants/imix/install_scripts/%d/main.eldritch",
		"test -f {{.BuildDir}}/realm/implants/imix/install_scripts/%d/params.json",
		"test -f {{.BuildDir}}/realm/implants/imix/install_scripts/%d/files/payload.txt",
		"mkdir -p $(dirname {{.ArtifactPath}})",
		"cat {{.BuildDir}}/realm/implants/imix/install_scripts/%d/main.eldritch {{.BuildDir}}/realm/implants/imix/install_scripts/%d/params.json {{.BuildDir}}/realm/implants/imix/install_scripts/%d/files/payload.txt > {{.ArtifactPath}}",
	}, " && "), tome.ID, tome.ID, tome.ID, tome.ID, tome.ID, tome.ID)

	profile := graph.BuildProfile.Create().
		SetName("default-setup-integration").
		SetDescription("verifies default setup_script with extracted source").
		SetBuildScript(buildCheckScript).
		SetTomes(tomeRefs).
		SaveX(ctx)

	expectedDefaultSetup := "cd {{.BuildDir}}/realm/implants/imix && rm -rf install_scripts/* && cp -r $REALM_TOMES_DIR/* install_scripts/ 2>/dev/null || true"
	require.Equal(t, expectedDefaultSetup, profile.Setupscript)
	require.NotContains(t, profile.Setupscript, "git clone")

	resolvedSetup, resolvedBuild, resolvedArtifact, err := builder.ResolveBuildRecipe(
		profile, profile.Setupscript, c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN,
	)
	require.NoError(t, err)
	require.NotContains(t, resolvedSetup, "git clone")
	require.Equal(t, "cd /tmp/realm-build/realm/implants/imix && rm -rf install_scripts/* && cp -r $REALM_TOMES_DIR/* install_scripts/ 2>/dev/null || true", resolvedSetup)

	snapshot := builder.NewProfileSnapshot(profile)
	bundle, err := builder.SnapshotProfile(ctx, graph, snapshot, profile.Tomes)
	require.NoError(t, err)
	tomeArchive, err := builder.FrozenTome(bundle, tome.ID)
	require.NoError(t, err)

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)
	result, err := executor.NewLocalExecutor().Build(ctx, executor.BuildSpec{
		TaskID:          1,
		TargetOS:        "linux",
		SetupScript:     resolvedSetup,
		PreBuildScript:  profile.Prebuildscript,
		BuildScript:     resolvedBuild,
		PostBuildScript: profile.Postbuildscript,
		ArtifactPath:    resolvedArtifact,
		Tomes: []executor.TomeData{{
			ID:       int64(tome.ID),
			Name:     tome.Name,
			Contents: tomeArchive,
			Params:   `{"interval":"10"}`,
		}},
	}, outputCh, errorCh)
	require.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, int64(0), result.ExitCode)
	assert.Equal(t, "print('staged-eldritch'){\"interval\":\"10\"}staged-asset-data", string(result.Artifact))
}

// TestResolveBuildRecipe_SetupScriptWithoutTemplate verifies that a setup
// script with no Go template syntax is passed through unchanged, so
// profiles created before BuildDir templating existed keep working.
func TestResolveBuildRecipe_SetupScriptWithoutTemplate(t *testing.T) {
	profile := &ent.BuildProfile{
		BuildScript:  "{{.BuildCommand}}",
		ArtifactPath: "{{.ArtifactPath}}",
	}
	setupScript := "echo plain setup, no templating here"

	resolvedSetup, _, _, err := builder.ResolveBuildRecipe(
		profile, setupScript, c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN,
	)
	require.NoError(t, err)
	assert.Equal(t, setupScript, resolvedSetup)
}

// Recipes have a stable path independent of the selected executor.
func TestResolveBuildRecipe_StableBuildDir(t *testing.T) {
	profile := &ent.BuildProfile{BuildScript: "cd {{.BuildDir}}/realm/implants/imix && {{.BuildCommand}}", ArtifactPath: "{{.ArtifactPath}}"}
	setup := "mkdir -p {{.BuildDir}}"
	firstSetup, firstBuild, firstArtifact, err := builder.ResolveBuildRecipe(profile, setup, c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN)
	require.NoError(t, err)
	secondSetup, secondBuild, secondArtifact, err := builder.ResolveBuildRecipe(profile, setup, c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN)
	require.NoError(t, err)
	assert.Equal(t, firstSetup, secondSetup)
	assert.Equal(t, firstBuild, secondBuild)
	assert.Equal(t, firstArtifact, secondArtifact)
	assert.Contains(t, firstSetup, builder.DockerBuildDir)
}

// script (e.g. explicitly overridden to "" at the task level) is left as
// an empty string rather than erroring out of renderRecipe's
// empty-result check.
func TestResolveBuildRecipe_EmptySetupScript(t *testing.T) {
	profile := &ent.BuildProfile{
		BuildScript:  "{{.BuildCommand}}",
		ArtifactPath: "{{.ArtifactPath}}",
	}

	resolvedSetup, _, _, err := builder.ResolveBuildRecipe(
		profile, "", c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN,
	)
	require.NoError(t, err)
	assert.Empty(t, resolvedSetup)
}
