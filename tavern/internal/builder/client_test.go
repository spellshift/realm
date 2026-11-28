package builder

import (
	"context"
	"errors"
	"os"
	"testing"
	"time"

	"golang.org/x/sync/semaphore"
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

type recordingBuildOutputStream struct {
	grpc.ClientStream
	final *builderpb.StreamBuildTaskOutputRequest
}

func (s *recordingBuildOutputStream) Send(req *builderpb.StreamBuildTaskOutputRequest) error {
	if req.Finished {
		s.final = req
	}
	return nil
}

func (s *recordingBuildOutputStream) CloseAndRecv() (*builderpb.StreamBuildTaskOutputResponse, error) {
	return &builderpb.StreamBuildTaskOutputResponse{}, nil
}

type recordingBuilderClient struct {
	builderpb.BuilderClient
	stream *recordingBuildOutputStream
}

func (c *recordingBuilderClient) StreamBuildTaskOutput(context.Context, ...grpc.CallOption) (builderpb.Builder_StreamBuildTaskOutputClient, error) {
	return c.stream, nil
}

func TestExecuteTask_ArtifactlessBuild(t *testing.T) {
	for _, tc := range []struct {
		name         string
		artifactPath string
		wantError    string
	}{
		{name: "no artifact requested"},
		{name: "expected artifact missing", artifactPath: "output/imix", wantError: "build produced no artifact"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			stream := &recordingBuildOutputStream{}
			client := &recordingBuilderClient{stream: stream}
			executeTask(context.Background(), "", client, executor.NewMockExecutor(), &builderpb.BuildTaskSpec{
				Id:           1,
				ArtifactPath: tc.artifactPath,
			})
			if stream.final == nil {
				t.Fatal("missing final build message")
			}
			if stream.final.Error != tc.wantError {
				t.Fatalf("final error = %q, want %q", stream.final.Error, tc.wantError)
			}
			if stream.final.ExitCode != 0 {
				t.Fatalf("final exit code = %d, want 0", stream.final.ExitCode)
			}
		})
	}
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
		executeTask(context.Background(), "", &fakeBuilderClient{}, mock, &builderpb.BuildTaskSpec{Id: 1})
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

// fakeClaimClient implements just enough of builderpb.BuilderClient to drive
// claimAndExecuteTasks: ClaimBuildTasks returns a fixed set of tasks, and
// StreamBuildTaskOutput returns a stream whose Send always fails (the test
// doesn't care about streamed output, only whether/when the build starts).
type fakeClaimClient struct {
	builderpb.BuilderClient
	tasks    []*builderpb.BuildTaskSpec
	hostname string
}

func (f *fakeClaimClient) ClaimBuildTasks(ctx context.Context, req *builderpb.ClaimBuildTasksRequest, opts ...grpc.CallOption) (*builderpb.ClaimBuildTasksResponse, error) {
	f.hostname = req.GetHostname()
	return &builderpb.ClaimBuildTasksResponse{Tasks: f.tasks}, nil
}

func (f *fakeClaimClient) StreamBuildTaskOutput(ctx context.Context, opts ...grpc.CallOption) (builderpb.Builder_StreamBuildTaskOutputClient, error) {
	return &fakeStreamOutputClient{}, nil
}

func TestClaimAndExecuteTasksReportsHostname(t *testing.T) {
	hostname, err := os.Hostname()
	if err != nil {
		t.Skipf("hostname unavailable: %v", err)
	}
	client := &fakeClaimClient{}
	if err := claimAndExecuteTasks(context.Background(), "", client, nil, semaphore.NewWeighted(1)); err != nil {
		t.Fatal(err)
	}
	if client.hostname != hostname {
		t.Fatalf("reported hostname %q, want %q", client.hostname, hostname)
	}
}

// TestClaimAndExecuteTasks_DoesNotBlockWhenAtCapacity verifies that
// claimAndExecuteTasks returns promptly even when the build semaphore is
// fully occupied, instead of blocking on sem.Acquire. This is what lets the
// builder's polling loop keep calling ClaimBuildTasks (which refreshes the
// builder's heartbeat) while at max concurrency. See spellshift/realm#2400.
func TestClaimAndExecuteTasks_DoesNotBlockWhenAtCapacity(t *testing.T) {
	ctx := context.Background()
	sem := semaphore.NewWeighted(1)

	// Occupy the only slot, simulating a builder already at max concurrency.
	if err := sem.Acquire(ctx, 1); err != nil {
		t.Fatalf("failed to pre-acquire semaphore: %v", err)
	}

	buildStarted := make(chan struct{})
	mock := executor.NewMockExecutor()
	mock.BuildFn = func(ctx context.Context, spec executor.BuildSpec, outputCh chan<- string, errorCh chan<- string) (*executor.BuildResult, error) {
		close(buildStarted)
		return &executor.BuildResult{}, nil
	}

	client := &fakeClaimClient{tasks: []*builderpb.BuildTaskSpec{{Id: 1}}}

	done := make(chan error, 1)
	go func() {
		done <- claimAndExecuteTasks(ctx, "", client, mock, sem)
	}()

	select {
	case err := <-done:
		if err != nil {
			t.Fatalf("claimAndExecuteTasks returned error: %v", err)
		}
	case <-time.After(2 * time.Second):
		t.Fatal("claimAndExecuteTasks blocked instead of returning immediately while at capacity")
	}

	// The build must not have started yet: the only semaphore slot is still held.
	select {
	case <-buildStarted:
		t.Fatal("build started before a semaphore slot was available")
	default:
	}

	// Freeing the slot should let the queued task run.
	sem.Release(1)

	select {
	case <-buildStarted:
	case <-time.After(2 * time.Second):
		t.Fatal("queued build never started after semaphore slot became available")
	}
}
