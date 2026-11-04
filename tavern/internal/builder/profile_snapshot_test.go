package builder_test

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
)

// TestResolveBuildRecipe_BuildDir verifies that {{.BuildDir}} resolves to the
// same generated directory across the setup script, build script, and
// artifact path, so a setup script can clone into an arbitrary directory
// without the build/artifact templates having to agree on it separately.
func TestResolveBuildRecipe_BuildDir(t *testing.T) {
	profile := &ent.BuildProfile{
		BuildScript:  "cd {{.BuildDir}}/realm/implants/imix && {{.BuildCommand}}",
		ArtifactPath: "{{.ArtifactPath}}",
	}
	setupScript := "mkdir -p {{.BuildDir}} && cd {{.BuildDir}} && git clone https://example.com/realm.git realm"

	resolvedSetup, resolvedBuild, resolvedArtifact, err := builder.ResolveBuildRecipe(
		profile, setupScript, c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN, "local",
	)
	require.NoError(t, err)

	// Extract the generated BuildDir from the rendered setup script and
	// confirm it's the exact same value used in the build script and
	// artifact path.
	const prefix = "mkdir -p "
	require.True(t, strings.HasPrefix(resolvedSetup, prefix))
	buildDir := strings.SplitN(strings.TrimPrefix(resolvedSetup, prefix), " ", 2)[0]
	require.NotEmpty(t, buildDir)

	assert.Contains(t, resolvedBuild, buildDir)
	assert.Contains(t, resolvedArtifact, buildDir)
	assert.Contains(t, resolvedArtifact, "x86_64-unknown-linux-musl")
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
		profile, setupScript, c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN, "local",
	)
	require.NoError(t, err)
	assert.Equal(t, setupScript, resolvedSetup)
}

// TestResolveBuildRecipe_DockerStaticBuildDir verifies that the Docker
// executor resolves {{.BuildDir}} to the fixed DockerBuildDir (so the cargo
// cache key is stable across builds and cached artifacts are reused), while
// the Local executor keeps a random, unique BuildDir.
func TestResolveBuildRecipe_DockerStaticBuildDir(t *testing.T) {
	profile := &ent.BuildProfile{
		BuildScript:  "cd {{.BuildDir}}/realm/implants/imix && {{.BuildCommand}}",
		ArtifactPath: "{{.ArtifactPath}}",
	}
	setupScript := "mkdir -p {{.BuildDir}} && cd {{.BuildDir}} && git clone https://example.com/realm.git realm"

	dockerSetup, dockerBuild, dockerArtifact, err := builder.ResolveBuildRecipe(
		profile, setupScript, c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN, "docker",
	)
	require.NoError(t, err)
	assert.Contains(t, dockerSetup, builder.DockerBuildDir)
	assert.Contains(t, dockerBuild, builder.DockerBuildDir)
	assert.Contains(t, dockerArtifact, builder.DockerBuildDir)

	localSetup, _, localArtifact, err := builder.ResolveBuildRecipe(
		profile, setupScript, c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN, "local",
	)
	require.NoError(t, err)
	// Local must remain a random, unique directory under /tmp (not the static
	// docker path).
	assert.Contains(t, localSetup, "/tmp/realm-build-")
	assert.Contains(t, localArtifact, "/tmp/realm-build-")
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
		profile, "", c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN, "local",
	)
	require.NoError(t, err)
	assert.Empty(t, resolvedSetup)
}
