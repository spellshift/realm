package executor_test

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"context"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"realm.pub/tavern/internal/builder/executor"
)

func TestLocalExecutor_ImplementsInterface(t *testing.T) {
	var _ executor.Executor = (*executor.LocalExecutor)(nil)
}

func TestLocalExecutor_Build_SimpleEcho(t *testing.T) {
	exec := executor.NewLocalExecutor()

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	spec := executor.BuildSpec{
		TaskID:      1,
		TargetOS:    "linux",
		BuildScript: "echo hello world",
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	result, err := exec.Build(ctx, spec, outputCh, errorCh)
	require.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, int64(0), result.ExitCode)

	var output []string
	for line := range outputCh {
		output = append(output, line)
	}
	assert.Contains(t, output, "hello world")
}

func TestLocalExecutor_Build_MultiLineOutput(t *testing.T) {
	exec := executor.NewLocalExecutor()

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	spec := executor.BuildSpec{
		TaskID:      2,
		TargetOS:    "linux",
		BuildScript: "echo line1 && echo line2 && echo line3",
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	result, err := exec.Build(ctx, spec, outputCh, errorCh)
	require.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, int64(0), result.ExitCode)

	var output []string
	for line := range outputCh {
		output = append(output, line)
	}
	assert.Contains(t, output, "line1")
	assert.Contains(t, output, "line2")
	assert.Contains(t, output, "line3")
}

func TestLocalExecutor_Build_Stderr(t *testing.T) {
	exec := executor.NewLocalExecutor()

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	spec := executor.BuildSpec{
		TaskID:      3,
		TargetOS:    "linux",
		BuildScript: "echo errline1 >&2 && echo errline2 >&2",
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	result, err := exec.Build(ctx, spec, outputCh, errorCh)
	require.NoError(t, err)
	require.NotNil(t, result)

	var errLines []string
	for line := range errorCh {
		errLines = append(errLines, line)
	}
	assert.Contains(t, errLines, "errline1")
	assert.Contains(t, errLines, "errline2")
}

func TestLocalExecutor_Build_NonZeroExit(t *testing.T) {
	exec := executor.NewLocalExecutor()

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	spec := executor.BuildSpec{
		TaskID:      4,
		TargetOS:    "linux",
		BuildScript: "exit 42",
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	result, err := exec.Build(ctx, spec, outputCh, errorCh)
	require.Error(t, err)
	require.NotNil(t, result)
	assert.Equal(t, int64(42), result.ExitCode)
	assert.Contains(t, err.Error(), "42")
}

func TestLocalExecutor_Build_ScriptStages(t *testing.T) {
	exec := executor.NewLocalExecutor()

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	spec := executor.BuildSpec{
		TaskID:          5,
		TargetOS:        "linux",
		SetupScript:     "echo stage_setup",
		PreBuildScript:  "echo stage_pre",
		BuildScript:     "echo stage_build",
		PostBuildScript: "echo stage_post",
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	result, err := exec.Build(ctx, spec, outputCh, errorCh)
	require.NoError(t, err)
	require.NotNil(t, result)

	var output []string
	for line := range outputCh {
		output = append(output, line)
	}

	var stageIndices []int
	stages := []string{"stage_setup", "stage_pre", "stage_build", "stage_post"}
	for _, st := range stages {
		found := false
		for i, line := range output {
			if line == st {
				stageIndices = append(stageIndices, i)
				found = true
				break
			}
		}
		require.True(t, found, "expected stage %q in output", st)
	}

	// Verify sequential ordering
	for i := 1; i < len(stageIndices); i++ {
		assert.Greater(t, stageIndices[i], stageIndices[i-1], "stages executed out of order")
	}
}

func TestLocalExecutor_Build_ExtractArtifact_Relative(t *testing.T) {
	exec := executor.NewLocalExecutor()

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	spec := executor.BuildSpec{
		TaskID:       6,
		TargetOS:     "linux",
		BuildScript:  "mkdir -p output && echo 'mock binary payload' > output/my-binary",
		ArtifactPath: "output/my-binary",
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	result, err := exec.Build(ctx, spec, outputCh, errorCh)
	require.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, int64(0), result.ExitCode)
	assert.Equal(t, "my-binary", result.ArtifactName)
	assert.Equal(t, "mock binary payload\n", string(result.Artifact))
}

func TestLocalExecutor_Build_ExtractArtifact_Absolute(t *testing.T) {
	exec := executor.NewLocalExecutor()

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	tempFile := filepath.Join(t.TempDir(), "abs-output.bin")

	spec := executor.BuildSpec{
		TaskID:       7,
		TargetOS:     "linux",
		BuildScript:  "echo 'abs binary payload' > " + tempFile,
		ArtifactPath: tempFile,
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	result, err := exec.Build(ctx, spec, outputCh, errorCh)
	require.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, int64(0), result.ExitCode)
	assert.Equal(t, "abs-output.bin", result.ArtifactName)
	assert.Equal(t, "abs binary payload\n", string(result.Artifact))
}

func TestLocalExecutor_Build_TomesExtraction(t *testing.T) {
	exec := executor.NewLocalExecutor()

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	// Create a sample tar.gz archive with main.eldritch
	var buf bytes.Buffer
	gw := gzip.NewWriter(&buf)
	tw := tar.NewWriter(gw)

	eldritchContent := []byte("print('hello from tome')")
	hdr := &tar.Header{
		Name: "main.eldritch",
		Mode: 0644,
		Size: int64(len(eldritchContent)),
	}
	require.NoError(t, tw.WriteHeader(hdr))
	_, err := tw.Write(eldritchContent)
	require.NoError(t, err)
	require.NoError(t, tw.Close())
	require.NoError(t, gw.Close())

	spec := executor.BuildSpec{
		TaskID:   8,
		TargetOS: "linux",
		Tomes: []executor.TomeData{
			{
				ID:       101,
				Name:     "sample-tome",
				Contents: buf.Bytes(),
				Params:   `{"foo":"bar"}`,
			},
		},
		BuildScript: "cat tomes/101/main.eldritch && echo '' && cat tomes/101/params.json",
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	result, err := exec.Build(ctx, spec, outputCh, errorCh)
	require.NoError(t, err)
	require.NotNil(t, result)

	var output []string
	for line := range outputCh {
		output = append(output, line)
	}
	assert.Contains(t, output, "print('hello from tome')")
	assert.Contains(t, output, `{"foo":"bar"}`)
}

func TestLocalExecutor_Build_ContextCancel(t *testing.T) {
	exec := executor.NewLocalExecutor()

	ctx, cancel := context.WithCancel(context.Background())

	spec := executor.BuildSpec{
		TaskID:      9,
		TargetOS:    "linux",
		BuildScript: "sleep 30",
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	start := time.Now()
	go func() {
		time.Sleep(100 * time.Millisecond)
		cancel()
	}()

	result, err := exec.Build(ctx, spec, outputCh, errorCh)
	elapsed := time.Since(start)

	assert.Error(t, err)
	assert.NotNil(t, result)
	assert.Less(t, elapsed, 5*time.Second, "cancellation did not terminate sleep in time")
}
