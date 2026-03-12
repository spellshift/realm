package executor_test

import (
	"context"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/docker/docker/client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/builder/executor"
)

func TestConstructTomeScript(t *testing.T) {
	tome := &builderpb.BuildTaskTome{
		Name:   "test_tome",
		Script: "print(input_params[\"key1\"])\n",
		Params: map[string]string{
			"key1": "value1",
			"key2": "value2",
		},
	}
	script := executor.ConstructTomeScript(tome)

	// Check the dictionary is constructed and string values are properly quoted
	assert.Contains(t, script, `input_params = {`)
	assert.Contains(t, script, `"key1": "value1",`)
	assert.Contains(t, script, `"key2": "value2",`)
	assert.Contains(t, script, `}`)
	assert.Contains(t, script, `print(input_params["key1"])`)
}

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
	assert.Equal(t, []string{"line1", "line2", "line3"}, output)
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
	// Because of our custom runner wrapper, it executes `/bin/sh -c set -e; for f in ...`
	// This wrapper creates its own output which might include setup commands or other statements.
	// Just ensure we got all lines from our script.
	expectedLines := []string{"line_1", "line_2", "line_3", "line_4", "line_5"}
	for _, expected := range expectedLines {
		found := false
		for _, line := range output {
			if strings.Contains(line, expected) {
				found = true
				break
			}
		}
		assert.True(t, found, "expected output to contain: %s\nGot: %v", expected, output)
	}

	// Print actual output for manual debugging if needed
	if t.Failed() {
		t.Logf("Full output:\n%v", strings.Join(output, "\n"))
	}
}
