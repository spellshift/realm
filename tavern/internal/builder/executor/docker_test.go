package executor_test

import (
	"context"
	"fmt"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/docker/docker/client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"realm.pub/tavern/internal/builder/executor"
)

// skipIfNoDocker skips the test if Docker is not available or functional.
func skipIfNoDocker(t *testing.T) client.APIClient {
	t.Helper()
	cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())
	if err != nil {
		t.Skipf("skipping docker test: %v", err)
	}
	ctx := context.Background()
	_, err = cli.Ping(ctx)
	if err != nil {
		t.Skipf("skipping docker test: docker not reachable: %v", err)
	}

	// Verify we can actually start a container (handles dind/mount issues)
	exec := executor.NewDockerExecutor(cli)
	spec := executor.BuildSpec{
		TaskID:      999,
		TargetOS:    "linux",
		BuildImage:  "alpine:latest",
		BuildScript: "true",
	}
	outputCh := make(chan string, 10)
	errorCh := make(chan string, 10)
	ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
	defer cancel()

	if _, err := exec.Build(ctx, spec, outputCh, errorCh); err != nil {
		t.Skipf("skipping docker test: docker functional check failed: %v", err)
	}

	return cli
}

func TestDockerExecutor_ImplementsInterface(t *testing.T) {
	var _ executor.Executor = (*executor.DockerExecutor)(nil)
}

func TestDockerExecutor_Build_SimpleEcho(t *testing.T) {
	cli := skipIfNoDocker(t)
	exec := executor.NewDockerExecutor(cli)

	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Minute)
	defer cancel()

	spec := executor.BuildSpec{
		TaskID:      1,
		TargetOS:    "linux",
		BuildImage:  "alpine:latest",
		BuildScript: "echo hello world",
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	_, err := exec.Build(ctx, spec, outputCh, errorCh)
	require.NoError(t, err)

	var output []string
	for line := range outputCh {
		output = append(output, line)
	}
	assert.Contains(t, output, "hello world")
}

func TestDockerExecutor_Build_MultiLineOutput(t *testing.T) {
	cli := skipIfNoDocker(t)
	exec := executor.NewDockerExecutor(cli)

	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Minute)
	defer cancel()

	spec := executor.BuildSpec{
		TaskID:      2,
		TargetOS:    "linux",
		BuildImage:  "alpine:latest",
		BuildScript: "echo line1 && echo line2 && echo line3",
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	_, err := exec.Build(ctx, spec, outputCh, errorCh)
	require.NoError(t, err)

	var output []string
	for line := range outputCh {
		output = append(output, line)
	}
	assert.Equal(t, []string{"==> Running /mnt/scripts/4_build.sh", "line1", "line2", "line3"}, output)
}

func TestDockerExecutor_Build_StderrOutput(t *testing.T) {
	cli := skipIfNoDocker(t)
	exec := executor.NewDockerExecutor(cli)

	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Minute)
	defer cancel()

	spec := executor.BuildSpec{
		TaskID:      3,
		TargetOS:    "linux",
		BuildImage:  "alpine:latest",
		BuildScript: "echo stdout_line && echo stderr_line >&2",
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	_, err := exec.Build(ctx, spec, outputCh, errorCh)
	require.NoError(t, err)

	var output []string
	for line := range outputCh {
		output = append(output, line)
	}
	var errOutput []string
	for line := range errorCh {
		errOutput = append(errOutput, line)
	}

	assert.Contains(t, output, "stdout_line")
	assert.Contains(t, errOutput, "stderr_line")
}

func TestDockerExecutor_Build_NonZeroExit(t *testing.T) {
	cli := skipIfNoDocker(t)
	exec := executor.NewDockerExecutor(cli)

	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Minute)
	defer cancel()

	spec := executor.BuildSpec{
		TaskID:      4,
		TargetOS:    "linux",
		BuildImage:  "alpine:latest",
		BuildScript: "echo before_fail && exit 42",
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	_, err := exec.Build(ctx, spec, outputCh, errorCh)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "42")

	var output []string
	for line := range outputCh {
		output = append(output, line)
	}
	assert.Contains(t, output, "before_fail")
}

func TestDockerExecutor_Build_ContextCancellation(t *testing.T) {
	cli := skipIfNoDocker(t)
	exec := executor.NewDockerExecutor(cli)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	spec := executor.BuildSpec{
		TaskID:      5,
		TargetOS:    "linux",
		BuildImage:  "alpine:latest",
		BuildScript: "echo started && sleep 120",
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	var buildErr error
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		_, buildErr = exec.Build(ctx, spec, outputCh, errorCh)
	}()

	wg.Wait()

	// The build should have been cancelled or errored due to timeout.
	require.Error(t, buildErr)
}

func TestDockerExecutor_Build_CacheVolumePersistsAcrossBuilds(t *testing.T) {
	cli := skipIfNoDocker(t)
	exec := executor.NewDockerExecutor(cli)

	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Minute)
	defer cancel()

	// Unique volume per test run so we don't collide with other tests or hosts.
	cacheVolume := fmt.Sprintf("realm-test-cache-%d", time.Now().UnixNano())
	cachePath := executor.DefaultCachePath
	marker := "cargo-deps-cached"

	buildScript := fmt.Sprintf(`marker=%s
echo "REALM_CACHE_DIR=$REALM_CACHE_DIR"
echo "CARGO_HOME=$CARGO_HOME"
echo "CARGO_TARGET_DIR=$CARGO_TARGET_DIR"
echo "GOMODCACHE=$GOMODCACHE"
mkdir -p "$REALM_CACHE_DIR/deps"
if [ -f "$REALM_CACHE_DIR/deps/marker" ]; then
	echo "CACHE_HIT=$(cat "$REALM_CACHE_DIR/deps/marker")"
else
	echo "$marker" > "$REALM_CACHE_DIR/deps/marker"
	echo "CACHE_MISS"
fi`, marker)

	collect := func(spec executor.BuildSpec) []string {
		outputCh := make(chan string, 200)
		errorCh := make(chan string, 200)
		_, err := exec.Build(ctx, spec, outputCh, errorCh)
		require.NoError(t, err)
		var out []string
		for line := range outputCh {
			out = append(out, line)
		}
		return out
	}

	// First build should populate the cache volume.
	first := collect(executor.BuildSpec{
		TaskID:      101,
		TargetOS:    "linux",
		BuildImage:  "alpine:latest",
		BuildScript: buildScript,
		CacheVolume: cacheVolume,
	})
	require.Contains(t, first, "CACHE_MISS", "first build should miss the cache")
	// Cache env vars should point at the mounted volume. Each output line is a
	// separate slice element, so check for the expected substring in the
	// joined output rather than an exact element match.
	require.Contains(t, strings.Join(first, "\n"), "REALM_CACHE_DIR="+cachePath)
	require.Contains(t, strings.Join(first, "\n"), fmt.Sprintf("CARGO_TARGET_DIR=%s/target/", cachePath))

	// Second build reuses the same volume: the marker must persist across
	// container teardown, proving dependency/artifact caching works.
	second := collect(executor.BuildSpec{
		TaskID:      102,
		TargetOS:    "linux",
		BuildImage:  "alpine:latest",
		BuildScript: buildScript,
		CacheVolume: cacheVolume,
	})
	require.Contains(t, second, "CACHE_HIT="+marker, "cache should persist across builds")

	// Clean up the test volume.
	if err := cli.VolumeRemove(ctx, cacheVolume, true); err != nil {
		t.Logf("failed to remove test cache volume %q: %v", cacheVolume, err)
	}
}

func TestDockerExecutor_Build_StreamsOutputInRealTime(t *testing.T) {
	cli := skipIfNoDocker(t)
	exec := executor.NewDockerExecutor(cli)

	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Minute)
	defer cancel()

	spec := executor.BuildSpec{
		TaskID:      6,
		TargetOS:    "linux",
		BuildImage:  "alpine:latest",
		BuildScript: "for i in 1 2 3 4 5; do echo line_$i; sleep 0.1; done",
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	var wg sync.WaitGroup
	wg.Add(1)

	var output []string
	go func() {
		defer wg.Done()
		for line := range outputCh {
			output = append(output, line)
		}
	}()

	_, err := exec.Build(ctx, spec, outputCh, errorCh)
	// Build closes outputCh, which unblocks the range loop in the goroutine.
	wg.Wait()

	require.NoError(t, err)
	require.Len(t, output, 6)
	assert.Equal(t, "==> Running /mnt/scripts/4_build.sh", output[0])
	for i, line := range output[1:] {
		assert.True(t, strings.HasPrefix(line, "line_"), "unexpected line %d: %s", i+1, line)
	}
}
