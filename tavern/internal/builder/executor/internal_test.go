package executor

import (
	"context"
	"fmt"
	"testing"

	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/api/types/network"
	"github.com/docker/docker/client"
	ocispec "github.com/opencontainers/image-spec/specs-go/v1"
	"github.com/stretchr/testify/require"
)

type cacheHelperClient struct {
	client.APIClient
	users     []string
	exitCodes []int64
}

func (c *cacheHelperClient) ContainerCreate(_ context.Context, config *container.Config, _ *container.HostConfig, _ *network.NetworkingConfig, _ *ocispec.Platform, _ string) (container.CreateResponse, error) {
	c.users = append(c.users, config.User)
	return container.CreateResponse{ID: fmt.Sprintf("helper-%d", len(c.users))}, nil
}

func (c *cacheHelperClient) ContainerStart(context.Context, string, container.StartOptions) error {
	return nil
}
func (c *cacheHelperClient) ContainerRemove(context.Context, string, container.RemoveOptions) error {
	return nil
}
func (c *cacheHelperClient) ContainerWait(_ context.Context, _ string, _ container.WaitCondition) (<-chan container.WaitResponse, <-chan error) {
	results := make(chan container.WaitResponse, 1)
	results <- container.WaitResponse{StatusCode: c.exitCodes[len(c.users)-1]}
	return results, make(chan error)
}

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

func TestEnsureVolumeWritableChecksBuildUser(t *testing.T) {
	for _, tc := range []struct {
		name      string
		exitCodes []int64
		wantUsers []string
		wantError bool
	}{
		{name: "chmod fails", exitCodes: []int64{1}, wantUsers: []string{"root"}, wantError: true},
		{name: "build user cannot write", exitCodes: []int64{0, 1}, wantUsers: []string{"root", ""}, wantError: true},
		{name: "writable", exitCodes: []int64{0, 0}, wantUsers: []string{"root", ""}},
	} {
		t.Run(tc.name, func(t *testing.T) {
			cli := &cacheHelperClient{exitCodes: tc.exitCodes}
			err := NewDockerExecutor(cli).ensureVolumeWritable(context.Background(), "cache", "/realm-cache", "image", "key")
			if tc.wantError {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
			require.Equal(t, tc.wantUsers, cli.users)
		})
	}
}
