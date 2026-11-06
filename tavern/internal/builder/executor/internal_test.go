package executor

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestCacheKeyFor_StablePerToolchain(t *testing.T) {
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

	require.Equal(t, cacheKeyFor(base), cacheKeyFor(BuildSpec{
		BuildImage:     "spellshift/devcontainer:main",
		TargetOS:       "linux",
		BuildScript:    "cargo build --release --features extra",
		PreBuildScript: "echo hi",
	}), "recipe differences share Cargo's cache and file locks")
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

	// Saved tasks from before stable recipe paths still need artifact extraction.
	require.Equal(t, "/realm-cache/target/abc123/x86_64-unknown-linux-musl/release/imix",
		remapArtifactPath("/tmp/realm-build-01234567-89ab-cdef-0123-456789abcdef/realm/implants/target/x86_64-unknown-linux-musl/release/imix", "/realm-cache/target/abc123"))
	require.Equal(t, "/tmp/realm-build-custom/realm/implants/target/release/imix",
		remapArtifactPath("/tmp/realm-build-custom/realm/implants/target/release/imix", "/realm-cache/target/abc123"))

	// Empty artifact path is returned unchanged.
	require.Equal(t, "", remapArtifactPath("", "/realm-cache/target/abc123"))
}

func TestCachedArtifactSpecKeepsScriptAndExtractionPathsEqual(t *testing.T) {
	spec := BuildSpec{
		ArtifactPath: "/tmp/realm-build/realm/implants/target/x86_64-unknown-linux-musl/release/imix",
		Env:          []string{"IMIX_CONFIG=example", "ARTIFACT_PATH=/tmp/realm-build/realm/implants/target/x86_64-unknown-linux-musl/release/imix"},
	}
	got := cachedArtifactSpec(spec, "/realm-cache/target/shared")
	require.Equal(t, "/realm-cache/target/shared/x86_64-unknown-linux-musl/release/imix", got.ArtifactPath)
	require.Equal(t, "ARTIFACT_PATH="+got.ArtifactPath, got.Env[1])
	require.Equal(t, "ARTIFACT_PATH="+spec.ArtifactPath, spec.Env[1], "the task spec must not be mutated")
}

func TestPrepareCacheDir(t *testing.T) {
	dir := filepath.Join(t.TempDir(), "cache")
	require.NoError(t, prepareCacheDir(dir, "key"))
	for _, path := range []string{"cargo", "target/key", "gopath/pkg/mod", "go-cache"} {
		info, err := os.Stat(filepath.Join(dir, path))
		require.NoError(t, err)
		require.Equal(t, os.FileMode(0o777), info.Mode().Perm(), path)
	}
	require.NoError(t, prepareCacheDir(dir, "key"))
	require.Error(t, prepareCacheDir("relative", "key"))
	file := filepath.Join(t.TempDir(), "file")
	require.NoError(t, os.WriteFile(file, nil, 0o600))
	require.Error(t, prepareCacheDir(file, "key"))
}
