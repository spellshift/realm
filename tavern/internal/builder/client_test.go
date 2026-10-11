package builder

import (
	"context"
	"errors"
	"testing"
	"time"

	"google.golang.org/grpc"

	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/builder/executor"
)

// fakeStreamOutputClient is a minimal builderpb.Builder_StreamBuildTaskOutputClient
// whose Send always fails, simulating a broken gRPC output stream (network
// drop, server restart, etc).
type fakeStreamOutputClient struct {
	grpc.ClientStream
}

func (f *fakeStreamOutputClient) Send(*builderpb.StreamBuildTaskOutputRequest) error {
	return errors.New("simulated stream send failure")
}

func (f *fakeStreamOutputClient) CloseAndRecv() (*builderpb.StreamBuildTaskOutputResponse, error) {
	return &builderpb.StreamBuildTaskOutputResponse{}, nil
}

// fakeBuilderClient only implements the StreamBuildTaskOutput RPC used by
// this test; all other methods are inherited (nil) and must not be called.
type fakeBuilderClient struct {
	builderpb.BuilderClient
}

func (f *fakeBuilderClient) StreamBuildTaskOutput(ctx context.Context, opts ...grpc.CallOption) (builderpb.Builder_StreamBuildTaskOutputClient, error) {
	return &fakeStreamOutputClient{}, nil
}

// TestExecuteTask_CancelsBuildContextOnStreamSendFailure verifies that when
// the output stream's Send fails, executeTask cancels the context passed to
// the executor instead of leaving it running forever. Without this,
// streamBuildLines' select on ctx.Done() can never fire, and a build
// producing more output than the channel buffer deadlocks permanently.
func TestExecuteTask_CancelsBuildContextOnStreamSendFailure(t *testing.T) {
	buildCtxCancelled := make(chan struct{})

	mock := executor.NewMockExecutor()
	mock.BuildFn = func(ctx context.Context, spec executor.BuildSpec, outputCh chan<- string, errorCh chan<- string) (*executor.BuildResult, error) {
		// Triggers the (failing) stream.Send in executeTask's collector goroutine.
		outputCh <- "line 1"

		select {
		case <-ctx.Done():
			close(buildCtxCancelled)
		case <-time.After(5 * time.Second):
			t.Error("build context was not cancelled after stream send failure")
		}
		return &executor.BuildResult{}, nil
	}

	done := make(chan struct{})
	go func() {
		executeTask(context.Background(), &fakeBuilderClient{}, mock, &builderpb.BuildTaskSpec{Id: 1})
		close(done)
	}()

	select {
	case <-done:
	case <-time.After(5 * time.Second):
		t.Fatal("executeTask did not return in time")
	}

	select {
	case <-buildCtxCancelled:
	default:
		t.Error("expected build context to have been cancelled")
	}
}
