package executor

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestCacheKeyFor_StablePerRecipe(t *testing.T) {
	base := BuildSpec{
		BuildImage:  "spellshift/devcontainer:main",
		TargetOS:    "linux",
		BuildScript: "cargo build --release",
	}
	require.Equal(t, cacheKeyFor(base), cacheKeyFor(base),
		"identical recipes must produce the same cache key")

	require.Len(t, cacheKeyFor(base), 16, "key must be a 16-char hex string")
}

func TestCacheKeyFor_DiffersOnInputs(t *testing.T) {
	base := BuildSpec{
		BuildImage:  "spellshift/devcontainer:main",
		TargetOS:    "linux",
		BuildScript: "cargo build --release",
	}

	require.NotEqual(t, cacheKeyFor(base), cacheKeyFor(BuildSpec{
		BuildImage:  "rust:latest",
		TargetOS:    "linux",
		BuildScript: "cargo build --release",
	}), "different build image must change the key")

	require.NotEqual(t, cacheKeyFor(base), cacheKeyFor(BuildSpec{
		BuildImage:  "spellshift/devcontainer:main",
		TargetOS:    "windows",
		BuildScript: "cargo build --release",
	}), "different target OS must change the key")

	require.NotEqual(t, cacheKeyFor(base), cacheKeyFor(BuildSpec{
		BuildImage:  "spellshift/devcontainer:main",
		TargetOS:    "linux",
		BuildScript: "cargo build --release --features extra",
	}), "different build script must change the key")

	require.NotEqual(t, cacheKeyFor(base), cacheKeyFor(BuildSpec{
		BuildImage:     "spellshift/devcontainer:main",
		TargetOS:       "linux",
		BuildScript:    "cargo build --release",
		PreBuildScript: "echo hi",
	}), "a prebuild script must change the key")
}

func TestRemapArtifactPath(t *testing.T) {
	// Default-derived path points at the workspace target dir; with
	// CARGO_TARGET_DIR set, cargo redirects output into the cache volume.
	got := remapArtifactPath(
		"/tmp/realm-build/realm/implants/target/x86_64-unknown-linux-musl/release/imix",
		"/realm-cache/target/abc123",
	)
	require.Equal(t, "/realm-cache/target/abc123/x86_64-unknown-linux-musl/release/imix", got)

	// A user-overridden path that doesn't look default-derived is returned as-is.
	require.Equal(t, "/opt/custom/imix",
		remapArtifactPath("/opt/custom/imix", "/realm-cache/target/abc123"))

	// Empty artifact path is returned unchanged.
	require.Equal(t, "", remapArtifactPath("", "/realm-cache/target/abc123"))
}
