package executor_test

import (
	"context"
	"errors"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"realm.pub/tavern/internal/builder/executor"
)

func TestMockExecutor_DefaultBehavior(t *testing.T) {
	mock := executor.NewMockExecutor()
	spec := executor.BuildSpec{
		TaskID:      1,
		TargetOS:    "linux",
		BuildImage:  "golang:1.21",
		BuildScript: "go build ./...",
	}

	outputCh := make(chan string, 10)
	errorCh := make(chan string, 10)

	err := mock.Build(context.Background(), spec, outputCh, errorCh)
	require.NoError(t, err)

	// Should have recorded the call.
	require.Len(t, mock.BuildCalls, 1)
	assert.Equal(t, spec, mock.BuildCalls[0])

	// No output or errors by default.
	assert.Empty(t, outputCh)
	assert.Empty(t, errorCh)
}

func TestMockExecutor_RecordsMultipleCalls(t *testing.T) {
	mock := executor.NewMockExecutor()
	outputCh := make(chan string, 10)
	errorCh := make(chan string, 10)

	specs := []executor.BuildSpec{
		{TaskID: 1, BuildImage: "golang:1.21", BuildScript: "go build ./..."},
		{TaskID: 2, BuildImage: "rust:1.75", BuildScript: "cargo build --release"},
		{TaskID: 3, BuildImage: "node:20", BuildScript: "npm run build"},
	}

	for _, s := range specs {
		err := mock.Build(context.Background(), s, outputCh, errorCh)
		require.NoError(t, err)
	}

	require.Len(t, mock.BuildCalls, 3)
	for i, s := range specs {
		assert.Equal(t, s, mock.BuildCalls[i])
	}
}

func TestMockExecutor_CustomBuildFn_StreamsOutput(t *testing.T) {
	mock := executor.NewMockExecutor()
	mock.BuildFn = func(ctx context.Context, spec executor.BuildSpec, outputCh chan<- string, errorCh chan<- string) error {
		outputCh <- "Compiling main.go"
		outputCh <- "Linking binary"
		outputCh <- "Build succeeded"
		return nil
	}

	outputCh := make(chan string, 10)
	errorCh := make(chan string, 10)

	err := mock.Build(context.Background(), executor.BuildSpec{TaskID: 1}, outputCh, errorCh)
	require.NoError(t, err)

	close(outputCh)
	close(errorCh)

	var output []string
	for line := range outputCh {
		output = append(output, line)
	}
	assert.Equal(t, []string{"Compiling main.go", "Linking binary", "Build succeeded"}, output)
	assert.Empty(t, errorCh)
}

func TestMockExecutor_CustomBuildFn_StreamsErrors(t *testing.T) {
	mock := executor.NewMockExecutor()
	mock.BuildFn = func(ctx context.Context, spec executor.BuildSpec, outputCh chan<- string, errorCh chan<- string) error {
		outputCh <- "Compiling..."
		errorCh <- "warning: unused variable"
		errorCh <- "error: type mismatch"
		return errors.New("build failed with exit code 1")
	}

	outputCh := make(chan string, 10)
	errorCh := make(chan string, 10)

	err := mock.Build(context.Background(), executor.BuildSpec{TaskID: 2}, outputCh, errorCh)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "build failed")

	close(outputCh)
	close(errorCh)

	var output []string
	for line := range outputCh {
		output = append(output, line)
	}
	assert.Equal(t, []string{"Compiling..."}, output)

	var errs []string
	for line := range errorCh {
		errs = append(errs, line)
	}
	assert.Equal(t, []string{"warning: unused variable", "error: type mismatch"}, errs)
}

func TestMockExecutor_CustomBuildFn_ContextCancellation(t *testing.T) {
	mock := executor.NewMockExecutor()
	mock.BuildFn = func(ctx context.Context, spec executor.BuildSpec, outputCh chan<- string, errorCh chan<- string) error {
		outputCh <- "Starting build..."
		<-ctx.Done()
		return ctx.Err()
	}

	ctx, cancel := context.WithCancel(context.Background())
	outputCh := make(chan string, 10)
	errorCh := make(chan string, 10)

	var wg sync.WaitGroup
	var buildErr error
	wg.Add(1)
	go func() {
		defer wg.Done()
		buildErr = mock.Build(ctx, executor.BuildSpec{TaskID: 3}, outputCh, errorCh)
	}()

	// Wait for the first output line to confirm Build started.
	select {
	case line := <-outputCh:
		assert.Equal(t, "Starting build...", line)
	case <-time.After(2 * time.Second):
		t.Fatal("timed out waiting for build to start")
	}

	cancel()
	wg.Wait()

	require.Error(t, buildErr)
	assert.ErrorIs(t, buildErr, context.Canceled)
}

func TestMockExecutor_CustomBuildFn_InterleavedOutputAndErrors(t *testing.T) {
	mock := executor.NewMockExecutor()
	mock.BuildFn = func(ctx context.Context, spec executor.BuildSpec, outputCh chan<- string, errorCh chan<- string) error {
		outputCh <- "step 1"
		errorCh <- "warn 1"
		outputCh <- "step 2"
		errorCh <- "warn 2"
		outputCh <- "step 3"
		return nil
	}

	outputCh := make(chan string, 10)
	errorCh := make(chan string, 10)

	err := mock.Build(context.Background(), executor.BuildSpec{TaskID: 4}, outputCh, errorCh)
	require.NoError(t, err)

	close(outputCh)
	close(errorCh)

	var output []string
	for line := range outputCh {
		output = append(output, line)
	}
	assert.Equal(t, []string{"step 1", "step 2", "step 3"}, output)

	var errs []string
	for line := range errorCh {
		errs = append(errs, line)
	}
	assert.Equal(t, []string{"warn 1", "warn 2"}, errs)
}

func TestMockExecutor_ImplementsInterface(t *testing.T) {
	var _ executor.Executor = (*executor.MockExecutor)(nil)
}
