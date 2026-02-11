package builder_test

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"errors"
	"net"
	"testing"

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
		SetTargetFormat("BIN").
		SetBuildImage("golang:1.21").
		SetBuildScript("go build ./...").
		SetCallbackURI("https://callback.example.com").
		SetTransportType(c2pb.Transport_TRANSPORT_GRPC).
		SetBuilderID(builders[0].ID).
		SaveX(ctx)

	// Setup gRPC
	lis := bufconn.Listen(1024 * 1024)
	grpcSrv := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			builder.NewMTLSAuthInterceptor(caCert, graph),
		),
		grpc.ChainStreamInterceptor(
			builder.NewMTLSStreamAuthInterceptor(caCert, graph),
		),
	)
	builderSrv := builder.New(graph, "dGVzdC1wdWJrZXk=")
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
	mock.BuildFn = func(ctx context.Context, spec executor.BuildSpec, outputCh chan<- string, errorCh chan<- string) (*executor.BuildResult, error) {
		outputCh <- "Building for linux..."
		outputCh <- "Success"
		return &executor.BuildResult{}, nil
	}

	task := claimResp.Tasks[0]
	outputCh := make(chan string, 64)
	errorCh := make(chan string, 64)

	_, buildErr := mock.Build(ctx, executor.BuildSpec{
		TaskID:      task.Id,
		TargetOS:    task.TargetOs,
		BuildImage:  task.BuildImage,
		BuildScript: task.BuildScript,
	}, outputCh, errorCh)
	require.NoError(t, buildErr)

	// Channels are closed by Build; drain and stream them.
	stream, err := authClient.StreamBuildTaskOutput(ctx)
	require.NoError(t, err)

	for line := range outputCh {
		require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
			TaskId: task.Id,
			Output: line,
		}))
	}

	// Send final message
	require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
		TaskId:   task.Id,
		Finished: true,
	}))

	resp, err := stream.CloseAndRecv()
	require.NoError(t, err)
	require.NotNil(t, resp)

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
		SetTargetFormat("BIN").
		SetBuildImage("golang:1.21").
		SetBuildScript("go build ./...").
		SetCallbackURI("https://callback.example.com").
		SetTransportType(c2pb.Transport_TRANSPORT_GRPC).
		SetBuilderID(builders[0].ID).
		SaveX(ctx)

	lis := bufconn.Listen(1024 * 1024)
	grpcSrv := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			builder.NewMTLSAuthInterceptor(caCert, graph),
		),
		grpc.ChainStreamInterceptor(
			builder.NewMTLSStreamAuthInterceptor(caCert, graph),
		),
	)
	builderSrv := builder.New(graph, "dGVzdC1wdWJrZXk=")
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
	mock.BuildFn = func(ctx context.Context, spec executor.BuildSpec, outputCh chan<- string, errorCh chan<- string) (*executor.BuildResult, error) {
		outputCh <- "Compiling..."
		errorCh <- "fatal: cannot find package"
		return &executor.BuildResult{}, errors.New("exit code 1")
	}

	task := claimResp.Tasks[0]
	outputCh := make(chan string, 64)
	errorCh := make(chan string, 64)

	_, buildErr := mock.Build(ctx, executor.BuildSpec{
		TaskID:      task.Id,
		TargetOS:    task.TargetOs,
		BuildImage:  task.BuildImage,
		BuildScript: task.BuildScript,
	}, outputCh, errorCh)
	require.Error(t, buildErr)

	// Channels are closed by Build; drain and stream them.
	stream, err := authClient.StreamBuildTaskOutput(ctx)
	require.NoError(t, err)

	for line := range outputCh {
		require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
			TaskId: task.Id,
			Output: line,
		}))
	}
	for line := range errorCh {
		require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
			TaskId: task.Id,
			Error:  line,
		}))
	}

	// Send final message with build error
	require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
		TaskId:   task.Id,
		Error:    buildErr.Error(),
		Finished: true,
	}))

	resp, err := stream.CloseAndRecv()
	require.NoError(t, err)
	require.NotNil(t, resp)

	// Verify the task was updated
	reloaded := graph.BuildTask.GetX(ctx, bt.ID)
	assert.Equal(t, "Compiling...", reloaded.Output)
	assert.Contains(t, reloaded.Error, "fatal: cannot find package")
	assert.Contains(t, reloaded.Error, "exit code 1")
	assert.False(t, reloaded.FinishedAt.IsZero())
}

func TestExecutorIntegration_StreamBuildOutput(t *testing.T) {
	ctx := context.Background()

	graph := enttest.Open(t, "sqlite3", "file:ent_stream_output?mode=memory&cache=shared&_fk=1")
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
		SetTargetFormat("BIN").
		SetBuildImage("golang:1.21").
		SetBuildScript("go build ./...").
		SetCallbackURI("https://callback.example.com").
		SetTransportType(c2pb.Transport_TRANSPORT_GRPC).
		SetBuilderID(builders[0].ID).
		SaveX(ctx)

	lis := bufconn.Listen(1024 * 1024)
	grpcSrv := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			builder.NewMTLSAuthInterceptor(caCert, graph),
		),
		grpc.ChainStreamInterceptor(
			builder.NewMTLSStreamAuthInterceptor(caCert, graph),
		),
	)
	builderSrv := builder.New(graph, "dGVzdC1wdWJrZXk=")
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

	task := claimResp.Tasks[0]

	// Open stream
	stream, err := authClient.StreamBuildTaskOutput(ctx)
	require.NoError(t, err)

	// Send output messages
	require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
		TaskId: task.Id,
		Output: "Building for linux...",
	}))
	require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
		TaskId: task.Id,
		Output: "Compiling main.go",
	}))

	// Send final message
	require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
		TaskId:   task.Id,
		Finished: true,
	}))

	// Close and receive response
	resp, err := stream.CloseAndRecv()
	require.NoError(t, err)
	require.NotNil(t, resp)

	// Verify DB state
	reloaded := graph.BuildTask.GetX(ctx, bt.ID)
	assert.Equal(t, "Building for linux...\nCompiling main.go", reloaded.Output)
	assert.False(t, reloaded.FinishedAt.IsZero())
	assert.False(t, reloaded.StartedAt.IsZero())
	assert.Empty(t, reloaded.Error)
}

func TestExecutorIntegration_StreamBuildOutputWithError(t *testing.T) {
	ctx := context.Background()

	graph := enttest.Open(t, "sqlite3", "file:ent_stream_output_err?mode=memory&cache=shared&_fk=1")
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
		SetTargetFormat("BIN").
		SetBuildImage("golang:1.21").
		SetBuildScript("go build ./...").
		SetCallbackURI("https://callback.example.com").
		SetTransportType(c2pb.Transport_TRANSPORT_GRPC).
		SetBuilderID(builders[0].ID).
		SaveX(ctx)

	lis := bufconn.Listen(1024 * 1024)
	grpcSrv := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			builder.NewMTLSAuthInterceptor(caCert, graph),
		),
		grpc.ChainStreamInterceptor(
			builder.NewMTLSStreamAuthInterceptor(caCert, graph),
		),
	)
	builderSrv := builder.New(graph, "dGVzdC1wdWJrZXk=")
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

	task := claimResp.Tasks[0]

	// Open stream
	stream, err := authClient.StreamBuildTaskOutput(ctx)
	require.NoError(t, err)

	// Send output + error lines
	require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
		TaskId: task.Id,
		Output: "Compiling...",
	}))
	require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
		TaskId: task.Id,
		Error:  "fatal: cannot find package",
	}))
	require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
		TaskId:   task.Id,
		Error:    "exit code 1",
		Finished: true,
	}))

	resp, err := stream.CloseAndRecv()
	require.NoError(t, err)
	require.NotNil(t, resp)

	reloaded := graph.BuildTask.GetX(ctx, bt.ID)
	assert.Equal(t, "Compiling...", reloaded.Output)
	assert.Contains(t, reloaded.Error, "fatal: cannot find package")
	assert.Contains(t, reloaded.Error, "exit code 1")
	assert.False(t, reloaded.FinishedAt.IsZero())
	assert.False(t, reloaded.StartedAt.IsZero())
}

func TestExecutorIntegration_UploadBuildArtifact(t *testing.T) {
	ctx := context.Background()

	graph := enttest.Open(t, "sqlite3", "file:ent_upload_artifact?mode=memory&cache=shared&_fk=1")
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

	graph.BuildTask.Create().
		SetTargetOs(c2pb.Host_PLATFORM_LINUX).
		SetTargetFormat("BIN").
		SetBuildImage("golang:1.21").
		SetBuildScript("go build -o /app/output/binary ./...").
		SetCallbackURI("https://callback.example.com").
		SetTransportType(c2pb.Transport_TRANSPORT_GRPC).
		SetArtifactPath("/app/output/binary").
		SetBuilderID(builders[0].ID).
		SaveX(ctx)

	lis := bufconn.Listen(1024 * 1024)
	grpcSrv := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			builder.NewMTLSAuthInterceptor(caCert, graph),
		),
		grpc.ChainStreamInterceptor(
			builder.NewMTLSStreamAuthInterceptor(caCert, graph),
		),
	)
	builderSrv := builder.New(graph, "dGVzdC1wdWJrZXk=")
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
	assert.Equal(t, "/app/output/binary", claimResp.Tasks[0].ArtifactPath)

	task := claimResp.Tasks[0]

	// Stream build output and finish
	stream, err := authClient.StreamBuildTaskOutput(ctx)
	require.NoError(t, err)

	require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
		TaskId: task.Id,
		Output: "Build completed",
	}))
	require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
		TaskId:   task.Id,
		Finished: true,
	}))

	_, err = stream.CloseAndRecv()
	require.NoError(t, err)

	// Upload artifact
	artifactData := []byte("fake-binary-content-for-testing")
	artifactStream, err := authClient.UploadBuildArtifact(ctx)
	require.NoError(t, err)

	require.NoError(t, artifactStream.Send(&builderpb.UploadBuildArtifactRequest{
		TaskId:       task.Id,
		ArtifactName: "binary",
		Chunk:        artifactData,
	}))

	artifactResp, err := artifactStream.CloseAndRecv()
	require.NoError(t, err)
	require.NotNil(t, artifactResp)
	assert.Greater(t, artifactResp.AssetId, int64(0))

	// Verify the asset was created
	asset, err := graph.Asset.Get(ctx, int(artifactResp.AssetId))
	require.NoError(t, err)
	assert.Contains(t, asset.Name, "build/linux/BIN/imix-")
	assert.Equal(t, artifactData, asset.Content)
	assert.Equal(t, len(artifactData), asset.Size)
}
