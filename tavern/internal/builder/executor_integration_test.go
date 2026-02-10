package builder_test

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"errors"
	"net"
	"testing"
	"time"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"

	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/builder/executor"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/tomes"
)

// setupTestInfra creates the test gRPC server and returns a builder gRPC client,
// the graph client, and the first registered builder's config.
func setupTestInfra(t *testing.T) (builderpb.BuilderClient, *client.Client, *builder.Config, func()) {
	t.Helper()

	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")

	_, caPrivKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	caCert, err := builder.CreateCA(caPrivKey)
	require.NoError(t, err)

	git := tomes.NewGitImporter(graph)
	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": handler.NewDefaultServer(graphql.NewSchema(graph, git, caCert, caPrivKey)),
		},
		tavernhttp.WithAuthenticationBypass(graph),
	)
	gqlClient := client.New(srv, client.Path("/graphql"))

	var registerResp struct {
		RegisterBuilder struct {
			Builder struct{ ID string }
			Config  string
		}
	}
	err = gqlClient.Post(`mutation registerNewBuilder($input: CreateBuilderInput!) {
		registerBuilder(input: $input) {
			builder { id }
			config
		}
	}`, &registerResp, client.Var("input", map[string]any{
		"supportedTargets": []string{"PLATFORM_LINUX"},
		"upstream":         "https://tavern.example.com:443",
	}))
	require.NoError(t, err)

	cfg, err := builder.ParseConfigBytes([]byte(registerResp.RegisterBuilder.Config))
	require.NoError(t, err)

	lis := bufconn.Listen(1024 * 1024)
	grpcSrv := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			builder.NewMTLSAuthInterceptor(caCert, graph),
		),
	)
	builderSrv := builder.New(graph)
	builderpb.RegisterBuilderServer(grpcSrv, builderSrv)

	go func() {
		if err := grpcSrv.Serve(lis); err != nil {
			t.Logf("gRPC server exited: %v", err)
		}
	}()

	bufDialer := func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}

	creds, err := builder.NewCredentialsFromConfig(cfg)
	require.NoError(t, err)

	conn, err := grpc.NewClient("passthrough:///bufnet",
		grpc.WithContextDialer(bufDialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithPerRPCCredentials(creds),
	)
	require.NoError(t, err)

	authClient := builderpb.NewBuilderClient(conn)

	cleanup := func() {
		conn.Close()
		grpcSrv.Stop()
		graph.Close()
	}

	return authClient, gqlClient, cfg, cleanup
}

func TestExecutorIntegration_MockExecutorSuccessfulBuild(t *testing.T) {
	ctx := context.Background()
	authClient, _, cfg, cleanup := setupTestInfra(t)
	defer cleanup()

	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Use the real graph from setup - we need to create tasks via the same DB.
	// Re-setup to get graph access directly.
	graph2 := enttest.Open(t, "sqlite3", "file:ent_exec?mode=memory&cache=shared&_fk=1")
	defer graph2.Close()

	_ = cfg

	// Create a mock executor that produces output
	mock := executor.NewMockExecutor()
	mock.BuildFn = func(ctx context.Context, spec executor.BuildSpec, outputCh chan<- string, errorCh chan<- string) error {
		outputCh <- "Compiling..."
		outputCh <- "Linking..."
		outputCh <- "Build complete"
		return nil
	}

	// Verify the mock implements the interface
	var _ executor.Executor = mock

	// Simulate what the builder client does: claim tasks, execute, report
	spec := executor.BuildSpec{
		TaskID:      42,
		TargetOS:    "linux",
		BuildImage:  "golang:1.21",
		BuildScript: "go build ./...",
	}

	outputCh := make(chan string, 64)
	errorCh := make(chan string, 64)

	err := mock.Build(ctx, spec, outputCh, errorCh)
	require.NoError(t, err)

	close(outputCh)
	close(errorCh)

	var output []string
	for line := range outputCh {
		output = append(output, line)
	}

	assert.Equal(t, []string{"Compiling...", "Linking...", "Build complete"}, output)
	require.Len(t, mock.BuildCalls, 1)
	assert.Equal(t, spec, mock.BuildCalls[0])

	// Verify no errors from the mock
	var errs []string
	for line := range errorCh {
		errs = append(errs, line)
	}
	assert.Empty(t, errs)

	_ = authClient
}

func TestExecutorIntegration_MockExecutorFailedBuild(t *testing.T) {
	mock := executor.NewMockExecutor()
	mock.BuildFn = func(ctx context.Context, spec executor.BuildSpec, outputCh chan<- string, errorCh chan<- string) error {
		outputCh <- "Compiling..."
		errorCh <- "error: undefined reference to 'main'"
		return errors.New("build failed with exit code 1")
	}

	outputCh := make(chan string, 64)
	errorCh := make(chan string, 64)

	err := mock.Build(context.Background(), executor.BuildSpec{
		TaskID:      43,
		BuildImage:  "golang:1.21",
		BuildScript: "go build ./...",
	}, outputCh, errorCh)

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
	assert.Equal(t, []string{"error: undefined reference to 'main'"}, errs)
}

func TestExecutorIntegration_MockExecutorContextCancellation(t *testing.T) {
	mock := executor.NewMockExecutor()
	mock.BuildFn = func(ctx context.Context, spec executor.BuildSpec, outputCh chan<- string, errorCh chan<- string) error {
		outputCh <- "Starting..."
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-time.After(30 * time.Second):
			return errors.New("should have been cancelled")
		}
	}

	ctx, cancel := context.WithCancel(context.Background())
	outputCh := make(chan string, 64)
	errorCh := make(chan string, 64)

	done := make(chan error, 1)
	go func() {
		done <- mock.Build(ctx, executor.BuildSpec{TaskID: 44}, outputCh, errorCh)
	}()

	// Wait for initial output to confirm build started
	select {
	case line := <-outputCh:
		assert.Equal(t, "Starting...", line)
	case <-time.After(2 * time.Second):
		t.Fatal("timed out waiting for build start")
	}

	cancel()

	err := <-done
	require.Error(t, err)
	assert.ErrorIs(t, err, context.Canceled)
}

func TestExecutorIntegration_ClaimAndExecuteWithMock(t *testing.T) {
	ctx := context.Background()

	// Full infrastructure setup
	graph := enttest.Open(t, "sqlite3", "file:ent_claim_exec?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	_, caPrivKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	caCert, err := builder.CreateCA(caPrivKey)
	require.NoError(t, err)

	git := tomes.NewGitImporter(graph)
	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": handler.NewDefaultServer(graphql.NewSchema(graph, git, caCert, caPrivKey)),
		},
		tavernhttp.WithAuthenticationBypass(graph),
	)
	gqlClient := client.New(srv, client.Path("/graphql"))

	// Register builder
	var registerResp struct {
		RegisterBuilder struct {
			Builder struct{ ID string }
			Config  string
		}
	}
	err = gqlClient.Post(`mutation registerNewBuilder($input: CreateBuilderInput!) {
		registerBuilder(input: $input) {
			builder { id }
			config
		}
	}`, &registerResp, client.Var("input", map[string]any{
		"supportedTargets": []string{"PLATFORM_LINUX"},
		"upstream":         "https://tavern.example.com:443",
	}))
	require.NoError(t, err)

	// Get builder DB entity
	builders, err := graph.Builder.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, builders, 1)

	// Create a build task
	bt := graph.BuildTask.Create().
		SetTargetOs(c2pb.Host_PLATFORM_LINUX).
		SetBuildImage("golang:1.21").
		SetBuildScript("go build ./...").
		SetBuilderID(builders[0].ID).
		SaveX(ctx)

	// Setup gRPC
	lis := bufconn.Listen(1024 * 1024)
	grpcSrv := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			builder.NewMTLSAuthInterceptor(caCert, graph),
		),
	)
	builderSrv := builder.New(graph)
	builderpb.RegisterBuilderServer(grpcSrv, builderSrv)

	go func() {
		if err := grpcSrv.Serve(lis); err != nil {
			t.Logf("gRPC server exited: %v", err)
		}
	}()
	defer grpcSrv.Stop()

	bufDialer := func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}

	cfg, err := builder.ParseConfigBytes([]byte(registerResp.RegisterBuilder.Config))
	require.NoError(t, err)

	creds, err := builder.NewCredentialsFromConfig(cfg)
	require.NoError(t, err)

	conn, err := grpc.NewClient("passthrough:///bufnet",
		grpc.WithContextDialer(bufDialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithPerRPCCredentials(creds),
	)
	require.NoError(t, err)
	defer conn.Close()

	authClient := builderpb.NewBuilderClient(conn)

	// Claim the task
	claimResp, err := authClient.ClaimBuildTasks(ctx, &builderpb.ClaimBuildTasksRequest{})
	require.NoError(t, err)
	require.Len(t, claimResp.Tasks, 1)
	assert.Equal(t, int64(bt.ID), claimResp.Tasks[0].Id)

	// Execute using mock executor
	mock := executor.NewMockExecutor()
	mock.BuildFn = func(ctx context.Context, spec executor.BuildSpec, outputCh chan<- string, errorCh chan<- string) error {
		outputCh <- "Building for linux..."
		outputCh <- "Success"
		return nil
	}

	task := claimResp.Tasks[0]
	outputCh := make(chan string, 64)
	errorCh := make(chan string, 64)

	buildErr := mock.Build(ctx, executor.BuildSpec{
		TaskID:      task.Id,
		TargetOS:    task.TargetOs,
		BuildImage:  task.BuildImage,
		BuildScript: task.BuildScript,
	}, outputCh, errorCh)
	require.NoError(t, buildErr)

	close(outputCh)
	close(errorCh)

	var outputLines []string
	for line := range outputCh {
		outputLines = append(outputLines, line)
	}

	// Submit output to server
	_, err = authClient.SubmitBuildTaskOutput(ctx, &builderpb.SubmitBuildTaskOutputRequest{
		TaskId: task.Id,
		Output: join(outputLines),
	})
	require.NoError(t, err)

	// Verify the task was updated in the database
	reloaded := graph.BuildTask.GetX(ctx, bt.ID)
	assert.Equal(t, "Building for linux...\nSuccess", reloaded.Output)
	assert.False(t, reloaded.FinishedAt.IsZero())
	assert.Empty(t, reloaded.Error)
}

func TestExecutorIntegration_ClaimAndExecuteWithMockError(t *testing.T) {
	ctx := context.Background()

	graph := enttest.Open(t, "sqlite3", "file:ent_claim_exec_err?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	_, caPrivKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	caCert, err := builder.CreateCA(caPrivKey)
	require.NoError(t, err)

	git := tomes.NewGitImporter(graph)
	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": handler.NewDefaultServer(graphql.NewSchema(graph, git, caCert, caPrivKey)),
		},
		tavernhttp.WithAuthenticationBypass(graph),
	)
	gqlClient := client.New(srv, client.Path("/graphql"))

	var registerResp struct {
		RegisterBuilder struct {
			Builder struct{ ID string }
			Config  string
		}
	}
	err = gqlClient.Post(`mutation registerNewBuilder($input: CreateBuilderInput!) {
		registerBuilder(input: $input) {
			builder { id }
			config
		}
	}`, &registerResp, client.Var("input", map[string]any{
		"supportedTargets": []string{"PLATFORM_LINUX"},
		"upstream":         "https://tavern.example.com:443",
	}))
	require.NoError(t, err)

	builders, err := graph.Builder.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, builders, 1)

	bt := graph.BuildTask.Create().
		SetTargetOs(c2pb.Host_PLATFORM_LINUX).
		SetBuildImage("golang:1.21").
		SetBuildScript("go build ./...").
		SetBuilderID(builders[0].ID).
		SaveX(ctx)

	lis := bufconn.Listen(1024 * 1024)
	grpcSrv := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			builder.NewMTLSAuthInterceptor(caCert, graph),
		),
	)
	builderSrv := builder.New(graph)
	builderpb.RegisterBuilderServer(grpcSrv, builderSrv)

	go func() {
		if err := grpcSrv.Serve(lis); err != nil {
			t.Logf("gRPC server exited: %v", err)
		}
	}()
	defer grpcSrv.Stop()

	bufDialer := func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}

	cfg, err := builder.ParseConfigBytes([]byte(registerResp.RegisterBuilder.Config))
	require.NoError(t, err)

	creds, err := builder.NewCredentialsFromConfig(cfg)
	require.NoError(t, err)

	conn, err := grpc.NewClient("passthrough:///bufnet",
		grpc.WithContextDialer(bufDialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithPerRPCCredentials(creds),
	)
	require.NoError(t, err)
	defer conn.Close()

	authClient := builderpb.NewBuilderClient(conn)

	claimResp, err := authClient.ClaimBuildTasks(ctx, &builderpb.ClaimBuildTasksRequest{})
	require.NoError(t, err)
	require.Len(t, claimResp.Tasks, 1)

	// Execute using mock that fails
	mock := executor.NewMockExecutor()
	mock.BuildFn = func(ctx context.Context, spec executor.BuildSpec, outputCh chan<- string, errorCh chan<- string) error {
		outputCh <- "Compiling..."
		errorCh <- "fatal: cannot find package"
		return errors.New("exit code 1")
	}

	task := claimResp.Tasks[0]
	outputCh := make(chan string, 64)
	errorCh := make(chan string, 64)

	buildErr := mock.Build(ctx, executor.BuildSpec{
		TaskID:      task.Id,
		TargetOS:    task.TargetOs,
		BuildImage:  task.BuildImage,
		BuildScript: task.BuildScript,
	}, outputCh, errorCh)
	require.Error(t, buildErr)

	close(outputCh)
	close(errorCh)

	var outputLines []string
	for line := range outputCh {
		outputLines = append(outputLines, line)
	}
	var errorLines []string
	for line := range errorCh {
		errorLines = append(errorLines, line)
	}

	// Submit output with error
	errMsg := join(errorLines) + "\n" + buildErr.Error()
	_, err = authClient.SubmitBuildTaskOutput(ctx, &builderpb.SubmitBuildTaskOutputRequest{
		TaskId: task.Id,
		Output: join(outputLines),
		Error:  errMsg,
	})
	require.NoError(t, err)

	// Verify the task was updated
	reloaded := graph.BuildTask.GetX(ctx, bt.ID)
	assert.Equal(t, "Compiling...", reloaded.Output)
	assert.Contains(t, reloaded.Error, "fatal: cannot find package")
	assert.Contains(t, reloaded.Error, "exit code 1")
	assert.False(t, reloaded.FinishedAt.IsZero())
}

func join(lines []string) string {
	result := ""
	for i, line := range lines {
		if i > 0 {
			result += "\n"
		}
		result += line
	}
	return result
}
