package builder_test

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"net"
	"testing"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"

	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/tomes"
)

func TestBuilderE2E(t *testing.T) {
	ctx := context.Background()

	// 1. Setup in-memory SQLite database
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// 2. Generate ED25519 key pair and create Builder CA
	_, caPrivKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	caCert, err := builder.CreateCA(caPrivKey)
	require.NoError(t, err)
	require.NotNil(t, caCert)

	// 3. Setup GraphQL server with authentication bypass and Builder CA
	git := tomes.NewGitImporter(graph)
	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": handler.NewDefaultServer(graphql.NewSchema(graph, git, caCert, caPrivKey)),
		},
		tavernhttp.WithAuthenticationBypass(graph),
	)
	gqlClient := client.New(srv, client.Path("/graphql"))

	// 4. Register a builder via GraphQL mutation
	var registerResp struct {
		RegisterBuilder struct {
			Builder struct {
				ID string
			}
			MtlsCert string
			Config   string
		}
	}

	err = gqlClient.Post(`mutation registerNewBuilder($input: CreateBuilderInput!) {
		registerBuilder(input: $input) {
			builder { id }
			mtlsCert
			config
		}
	}`, &registerResp, client.Var("input", map[string]any{
		"supportedTargets": []string{"PLATFORM_LINUX", "PLATFORM_MACOS"},
		"upstream":         "https://tavern.example.com:443",
	}))
	require.NoError(t, err)
	require.NotEmpty(t, registerResp.RegisterBuilder.Builder.ID)
	require.NotEmpty(t, registerResp.RegisterBuilder.MtlsCert)
	require.NotEmpty(t, registerResp.RegisterBuilder.Config)

	// 5. Parse the returned YAML config
	cfg, err := builder.ParseConfigBytes([]byte(registerResp.RegisterBuilder.Config))
	require.NoError(t, err)
	assert.Contains(t, cfg.SupportedTargets, "linux")
	assert.Contains(t, cfg.SupportedTargets, "macos")
	assert.NotEmpty(t, cfg.MTLS)
	assert.NotEmpty(t, cfg.ID)
	assert.Equal(t, "https://tavern.example.com:443", cfg.Upstream)

	// 6. Verify builder exists in DB
	builders, err := graph.Builder.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, builders, 1)
	assert.Equal(t, cfg.ID, builders[0].Identifier)

	// 7. Setup builder gRPC server via bufconn with mTLS auth interceptor
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

	// 8. Test: Unauthenticated request should be rejected
	t.Run("unauthenticated request rejected", func(t *testing.T) {
		conn, err := grpc.NewClient("passthrough:///bufnet",
			grpc.WithContextDialer(bufDialer),
			grpc.WithTransportCredentials(insecure.NewCredentials()),
		)
		require.NoError(t, err)
		defer conn.Close()

		unauthClient := builderpb.NewBuilderClient(conn)
		_, err = unauthClient.ClaimBuildTasks(ctx, &builderpb.ClaimBuildTasksRequest{})
		require.Error(t, err)
		assert.Equal(t, codes.Unauthenticated, status.Code(err))
	})

	// 9. Test: Authenticated request should succeed
	t.Run("authenticated request succeeds", func(t *testing.T) {
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
		require.NotNil(t, claimResp)
	})

	// 10. Test: ClaimBuildTasks returns unclaimed tasks and marks them as claimed
	t.Run("ClaimBuildTasks", func(t *testing.T) {
		// Create an authenticated client
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

		// Create a build task assigned to this builder
		bt := graph.BuildTask.Create().
			SetTargetOs(c2pb.Host_PLATFORM_LINUX).
			SetTargetFormat(builderpb.TargetFormat_TARGET_FORMAT_BIN).
			SetBuildImage("golang:1.21").
			SetBuildScript("echo hello && go build ./...").
			SetCallbackURI("https://callback.example.com").
			SetInterval(10).
			SetTransportType(c2pb.Transport_TRANSPORT_GRPC).
			SetBuilderID(builders[0].ID).
			SaveX(ctx)

		// Claim tasks
		resp, err := authClient.ClaimBuildTasks(ctx, &builderpb.ClaimBuildTasksRequest{})
		require.NoError(t, err)
		require.Len(t, resp.Tasks, 1)
		assert.Equal(t, int64(bt.ID), resp.Tasks[0].Id)
		assert.Equal(t, "golang:1.21", resp.Tasks[0].BuildImage)
		assert.Equal(t, "echo hello && go build ./...", resp.Tasks[0].BuildScript)

		// Verify IMIX_CONFIG env var is set
		require.Len(t, resp.Tasks[0].Env, 1)
		assert.Contains(t, resp.Tasks[0].Env[0], "IMIX_CONFIG=")
		assert.Contains(t, resp.Tasks[0].Env[0], "transports:")
		assert.Contains(t, resp.Tasks[0].Env[0], "URI: https://callback.example.com")
		assert.Contains(t, resp.Tasks[0].Env[0], "interval: 10")
		assert.Contains(t, resp.Tasks[0].Env[0], "type: grpc")

		// Verify claimed_at is set
		reloaded := graph.BuildTask.GetX(ctx, bt.ID)
		assert.False(t, reloaded.ClaimedAt.IsZero())

		// Verify builder's last_seen_at was updated
		reloadedBuilder := graph.Builder.GetX(ctx, builders[0].ID)
		require.NotNil(t, reloadedBuilder.LastSeenAt)
		assert.False(t, reloadedBuilder.LastSeenAt.IsZero())

		// Claim again -> should return empty (already claimed)
		resp2, err := authClient.ClaimBuildTasks(ctx, &builderpb.ClaimBuildTasksRequest{})
		require.NoError(t, err)
		assert.Empty(t, resp2.Tasks)
	})

	// 11. Test: StreamBuildTaskOutput sets output and finished_at
	t.Run("StreamBuildTaskOutput", func(t *testing.T) {
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

		// Create and claim a build task
		bt := graph.BuildTask.Create().
			SetTargetOs(c2pb.Host_PLATFORM_MACOS).
			SetTargetFormat(builderpb.TargetFormat_TARGET_FORMAT_BIN).
			SetBuildImage("rust:1.75").
			SetBuildScript("cargo build --release").
			SetCallbackURI("https://callback.example.com").
			SetTransportType(c2pb.Transport_TRANSPORT_GRPC).
			SetBuilderID(builders[0].ID).
			SaveX(ctx)

		// Claim the task
		claimResp, err := authClient.ClaimBuildTasks(ctx, &builderpb.ClaimBuildTasksRequest{})
		require.NoError(t, err)
		require.Len(t, claimResp.Tasks, 1)

		// Stream output
		stream, err := authClient.StreamBuildTaskOutput(ctx)
		require.NoError(t, err)

		require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
			TaskId: int64(bt.ID),
			Output: "build succeeded",
		}))
		require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
			TaskId:   int64(bt.ID),
			Finished: true,
			ExitCode: 0,
		}))

		_, err = stream.CloseAndRecv()
		require.NoError(t, err)

		// Verify the build task was updated
		reloaded := graph.BuildTask.GetX(ctx, bt.ID)
		assert.Equal(t, "build succeeded", reloaded.Output)
		assert.False(t, reloaded.FinishedAt.IsZero())
		assert.Empty(t, reloaded.Error)
		require.NotNil(t, reloaded.ExitCode)
		assert.Equal(t, 0, *reloaded.ExitCode)
	})

	// 12. Test: StreamBuildTaskOutput with error
	t.Run("StreamBuildTaskOutputWithError", func(t *testing.T) {
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

		// Create and claim a build task
		bt := graph.BuildTask.Create().
			SetTargetOs(c2pb.Host_PLATFORM_LINUX).
			SetTargetFormat(builderpb.TargetFormat_TARGET_FORMAT_BIN).
			SetBuildImage("golang:1.21").
			SetBuildScript("go build ./...").
			SetCallbackURI("https://callback.example.com").
			SetTransportType(c2pb.Transport_TRANSPORT_GRPC).
			SetBuilderID(builders[0].ID).
			SaveX(ctx)

		claimResp, err := authClient.ClaimBuildTasks(ctx, &builderpb.ClaimBuildTasksRequest{})
		require.NoError(t, err)
		require.Len(t, claimResp.Tasks, 1)

		// Stream output with error
		stream, err := authClient.StreamBuildTaskOutput(ctx)
		require.NoError(t, err)

		require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
			TaskId: int64(bt.ID),
			Output: "partial output before failure",
		}))
		require.NoError(t, stream.Send(&builderpb.StreamBuildTaskOutputRequest{
			TaskId:   int64(bt.ID),
			Error:    "compilation failed: missing dependency",
			Finished: true,
			ExitCode: 1,
		}))

		_, err = stream.CloseAndRecv()
		require.NoError(t, err)

		// Verify the build task has both output and error
		reloaded := graph.BuildTask.GetX(ctx, bt.ID)
		assert.Equal(t, "partial output before failure", reloaded.Output)
		assert.Equal(t, "compilation failed: missing dependency", reloaded.Error)
		assert.False(t, reloaded.FinishedAt.IsZero())
		require.NotNil(t, reloaded.ExitCode)
		assert.Equal(t, 1, *reloaded.ExitCode)
	})

	// 13. Test: StreamBuildTaskOutput for another builder's task is rejected
	t.Run("StreamBuildTaskOutputWrongBuilder", func(t *testing.T) {
		// Register a second builder
		var registerResp2 struct {
			RegisterBuilder struct {
				Builder struct {
					ID string
				}
				Config string
			}
		}
		err := gqlClient.Post(`mutation registerNewBuilder($input: CreateBuilderInput!) {
			registerBuilder(input: $input) {
				builder { id }
				config
			}
		}`, &registerResp2, client.Var("input", map[string]any{
			"supportedTargets": []string{"PLATFORM_WINDOWS"},
			"upstream":         "https://tavern.example.com:443",
		}))
		require.NoError(t, err)

		cfg2, err := builder.ParseConfigBytes([]byte(registerResp2.RegisterBuilder.Config))
		require.NoError(t, err)

		// Get the second builder's DB entity
		allBuilders, err := graph.Builder.Query().All(ctx)
		require.NoError(t, err)
		require.Len(t, allBuilders, 2)

		var secondBuilder int
		for _, b := range allBuilders {
			if b.Identifier == cfg2.ID {
				secondBuilder = b.ID
			}
		}

		// Create a build task assigned to the second builder
		bt := graph.BuildTask.Create().
			SetTargetOs(c2pb.Host_PLATFORM_WINDOWS).
			SetTargetFormat(builderpb.TargetFormat_TARGET_FORMAT_BIN).
			SetBuildImage("mcr.microsoft.com/windows:ltsc2022").
			SetBuildScript("msbuild /t:Build").
			SetCallbackURI("https://callback.example.com").
			SetTransportType(c2pb.Transport_TRANSPORT_GRPC).
			SetBuilderID(secondBuilder).
			SaveX(ctx)

		// Try to stream output using the FIRST builder's credentials
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

		stream, err := authClient.StreamBuildTaskOutput(ctx)
		require.NoError(t, err)

		// Send a message for the wrong builder's task
		err = stream.Send(&builderpb.StreamBuildTaskOutputRequest{
			TaskId: int64(bt.ID),
			Output: "should not be allowed",
		})
		// The send might succeed but the server will reject on recv
		if err == nil {
			_, err = stream.CloseAndRecv()
		}
		require.Error(t, err)
		assert.Equal(t, codes.PermissionDenied, status.Code(err))
	})
}
