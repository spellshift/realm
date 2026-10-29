package builder_test

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"encoding/pem"
	"net"
	"sync"
	"sync/atomic"
	"testing"

	"github.com/99designs/gqlgen/client"
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
)

func TestBuilderE2E(t *testing.T) {
	ctx := context.Background()

	// 1. Setup in-memory SQLite database
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	// 2. Generate ED25519 key pair and create Builder CA
	_, caPrivKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	caCert, err := builder.CreateCA(caPrivKey)
	require.NoError(t, err)
	require.NotNil(t, caCert)

	// 3. Setup GraphQL server with authentication bypass and Builder CA
	gqlClient, upstream := newTestBuilderServer(t, graph, caCert, caPrivKey)

	// An invalid CSR must be rejected before any builder is created, and
	// must not consume the token.
	token := createTestEnrollmentToken(t, gqlClient)
	var rejected struct {
		ExchangeBuilderEnrollmentToken struct{ BuilderID string }
	}
	err = gqlClient.Post(`mutation($token: String!, $input: CreateBuilderInput!, $csr: String!) {
		exchangeBuilderEnrollmentToken(token: $token, input: $input, csr: $csr) { builderID }
	}`, &rejected, client.Var("token", token), client.Var("input", map[string]any{"supportedTargets": []string{"PLATFORM_LINUX"}}), client.Var("csr", "invalid"))
	require.ErrorContains(t, err, "invalid builder CSR")
	count, err := graph.Builder.Query().Count(ctx)
	require.NoError(t, err)
	require.Zero(t, count)

	csr, keyPEM := generateTestCSR(t)
	for _, tc := range []struct {
		name  string
		input map[string]any
		want  string
	}{
		{
			name:  "empty targets",
			input: map[string]any{"supportedTargets": []string{}},
			want:  "at least one supported_target",
		},
		{
			name:  "empty upstream",
			input: map[string]any{"supportedTargets": []string{"PLATFORM_LINUX"}, "upstream": ""},
			want:  "upstream server address",
		},
	} {
		t.Run(tc.name, func(t *testing.T) {
			err := gqlClient.Post(`mutation($token: String!, $input: CreateBuilderInput!, $csr: String!) {
				exchangeBuilderEnrollmentToken(token: $token, input: $input, csr: $csr) { builderID }
			}`, &rejected, client.Var("token", token), client.Var("input", tc.input), client.Var("csr", csr))
			require.ErrorContains(t, err, tc.want)
			count, err := graph.Builder.Query().Count(ctx)
			require.NoError(t, err)
			require.Zero(t, count)
		})
	}

	// The token must still be usable: every exchange above failed validation
	// before consuming it.
	var exchangeResp struct {
		ExchangeBuilderEnrollmentToken struct {
			BuilderID string
			MtlsCert  string
		}
	}
	err = gqlClient.Post(`mutation exchange($token: String!, $input: CreateBuilderInput!, $csr: String!) {
		exchangeBuilderEnrollmentToken(token: $token, input: $input, csr: $csr) {
			builderID
			mtlsCert
		}
	}`, &exchangeResp, client.Var("csr", csr), client.Var("token", token), client.Var("input", map[string]any{
		"supportedTargets": []string{"PLATFORM_LINUX", "PLATFORM_MACOS"},
		"upstream":         "https://tavern.example.com:443",
		"pollInterval":     30,
	}))
	require.NoError(t, err)
	require.NotEmpty(t, exchangeResp.ExchangeBuilderEnrollmentToken.BuilderID)
	require.NotEmpty(t, exchangeResp.ExchangeBuilderEnrollmentToken.MtlsCert)
	assert.NotContains(t, exchangeResp.ExchangeBuilderEnrollmentToken.MtlsCert, "PRIVATE KEY")

	// Replaying the same token (even with the same CSR) must now fail, and
	// must not create a second builder.
	err = gqlClient.Post(`mutation($token: String!, $input: CreateBuilderInput!, $csr: String!) {
		exchangeBuilderEnrollmentToken(token: $token, input: $input, csr: $csr) { builderID }
	}`, &rejected, client.Var("token", token), client.Var("csr", csr), client.Var("input", map[string]any{"supportedTargets": []string{"PLATFORM_LINUX"}}))
	require.ErrorContains(t, err, "invalid or used")
	count, err = graph.Builder.Query().Count(ctx)
	require.NoError(t, err)
	require.Equal(t, 1, count)

	// 5. Build a usable Config/credentials from the exchange response.
	builderID := exchangeResp.ExchangeBuilderEnrollmentToken.BuilderID
	cfg := saveIdentityAndConfig(t, builderID, exchangeResp.ExchangeBuilderEnrollmentToken.MtlsCert, keyPEM, upstream, "linux", "macos")
	assert.Contains(t, cfg.SupportedTargets, "linux")

	// 6. Verify builder exists in DB
	builders, err := graph.Builder.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, builders, 1)
	assert.Equal(t, builderID, builders[0].Identifier)

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

	profile := graph.BuildProfile.Create().
		SetName("default").
		SetBuildImage("rust:1.75").
		SetDescription("The default profile").
		SaveX(ctx)

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
		explictProfile := graph.BuildProfile.Create().
			SetName("explicit").
			SetBuildImage("golang:1.21").
			SetDescription("An explicit profile").
			SetTransports([]builderpb.BuildProfileTransport{{
				URI:      "https://callback.example.com",
				Interval: 10,
				Type:     c2pb.Transport_TRANSPORT_GRPC,
			}}).
			SaveX(ctx)

		// Create a build task assigned to this builder
		bt := graph.BuildTask.Create().
			SetTargetOs(c2pb.Host_PLATFORM_LINUX).
			SetTargetFormat(builderpb.TargetFormat_TARGET_FORMAT_BIN).
			SetBuildScript("echo hello && go build ./...").
			SetBuilderID(builders[0].ID).
			SetProfileID(explictProfile.ID).
			SetProfileAtCreation(snapshotForTest(t, graph, explictProfile)).
			SaveX(ctx)

		// Claim tasks
		resp, err := authClient.ClaimBuildTasks(ctx, &builderpb.ClaimBuildTasksRequest{})
		require.NoError(t, err)
		require.Len(t, resp.Tasks, 1)
		assert.Equal(t, int64(bt.ID), resp.Tasks[0].Id)
		assert.Equal(t, "golang:1.21", resp.Tasks[0].BuildImage)
		assert.Equal(t, "echo hello && go build ./...", resp.Tasks[0].BuildScript)

		// Verify env vars are set (IMIX_CONFIG + ARTIFACT_PATH)
		require.Len(t, resp.Tasks[0].Env, 2)
		assert.Contains(t, resp.Tasks[0].Env[0], "IMIX_CONFIG=")
		assert.Contains(t, resp.Tasks[0].Env[0], "transports:")
		assert.Contains(t, resp.Tasks[0].Env[0], "URI: https://callback.example.com")
		assert.Contains(t, resp.Tasks[0].Env[0], "interval: 10")
		assert.Contains(t, resp.Tasks[0].Env[0], "type: grpc")
		assert.Contains(t, resp.Tasks[0].Env[1], "ARTIFACT_PATH=")

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

	t.Run("ProfileSnapshotLifecycle", func(t *testing.T) {
		creds, err := builder.NewCredentialsFromConfig(cfg)
		require.NoError(t, err)
		conn, err := grpc.NewClient("passthrough:///bufnet",
			grpc.WithContextDialer(bufDialer),
			grpc.WithTransportCredentials(insecure.NewCredentials()),
			grpc.WithPerRPCCredentials(creds),
		)
		require.NoError(t, err)
		defer conn.Close()
		testProfileTemplateValidation(t, graph)
		testProfileSnapshotLifecycle(t, graph, gqlClient, builderpb.NewBuilderClient(conn), builders[0].ID)
		testTaskOverridesApplyToSnapshot(t, graph, gqlClient, builderpb.NewBuilderClient(conn), builders[0].ID)
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
			SetBuildScript("cargo build --release").
			SetProfileID(profile.ID).
			SetProfileAtCreation(snapshotForTest(t, graph, profile)).
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
			SetBuildScript("go build ./...").
			SetProfileID(profile.ID).
			SetProfileAtCreation(snapshotForTest(t, graph, profile)).
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
		cfg2 := enrollTestBuilder(t, ctx, gqlClient, upstream, "windows")

		// Get the second builder's DB entity
		allBuilders, err := graph.Builder.Query().All(ctx)
		require.NoError(t, err)
		require.Len(t, allBuilders, 2)

		identity2, err := builder.LoadIdentity(cfg2.IdentityFile)
		require.NoError(t, err)

		var secondBuilder int
		for _, b := range allBuilders {
			if b.Identifier == identity2.ID {
				secondBuilder = b.ID
			}
		}
		require.NotZero(t, secondBuilder)

		// Create a build task assigned to the second builder
		bt := graph.BuildTask.Create().
			SetTargetOs(c2pb.Host_PLATFORM_WINDOWS).
			SetTargetFormat(builderpb.TargetFormat_TARGET_FORMAT_BIN).
			SetBuildScript("msbuild /t:Build").
			SetProfileID(profile.ID).
			SetProfileAtCreation(snapshotForTest(t, graph, profile)).
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

// TestExchangeBuilderEnrollmentToken_ConcurrentRedemption verifies that when
// many requests race to redeem the same one-time token, exactly one succeeds
// and exactly one builder is created, regardless of how the requests
// interleave.
func TestExchangeBuilderEnrollmentToken_ConcurrentRedemption(t *testing.T) {
	ctx := context.Background()

	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	_, caPrivKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	caCert, err := builder.CreateCA(caPrivKey)
	require.NoError(t, err)

	gqlClient, _ := newTestBuilderServer(t, graph, caCert, caPrivKey)
	token := createTestEnrollmentToken(t, gqlClient)

	// Each goroutine needs its own CSR/key; testify's require must only run
	// on the test's own goroutine, so failures here are reported via assert
	// (and the error channel) rather than require.
	newCSR := func() (string, error) {
		_, key, err := ed25519.GenerateKey(rand.Reader)
		if err != nil {
			return "", err
		}
		csrDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, key)
		if err != nil {
			return "", err
		}
		return string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrDER})), nil
	}

	const concurrency = 8
	var (
		wg        sync.WaitGroup
		successes atomic.Int32
		failures  atomic.Int32
	)
	for i := 0; i < concurrency; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			csr, err := newCSR()
			if !assert.NoError(t, err) {
				return
			}
			var resp struct {
				ExchangeBuilderEnrollmentToken struct{ BuilderID string }
			}
			err = gqlClient.Post(`mutation($token: String!, $input: CreateBuilderInput!, $csr: String!) {
				exchangeBuilderEnrollmentToken(token: $token, input: $input, csr: $csr) { builderID }
			}`, &resp, client.Var("token", token), client.Var("csr", csr), client.Var("input", map[string]any{"supportedTargets": []string{"PLATFORM_LINUX"}}))
			if err == nil {
				assert.NotEmpty(t, resp.ExchangeBuilderEnrollmentToken.BuilderID)
				successes.Add(1)
				return
			}
			if assert.ErrorContains(t, err, "invalid or used") {
				failures.Add(1)
			}
		}()
	}
	wg.Wait()
	assert.Equal(t, int32(concurrency), successes.Load()+failures.Load())

	assert.Equal(t, int32(1), successes.Load())
	count, err := graph.Builder.Query().Count(ctx)
	require.NoError(t, err)
	assert.Equal(t, 1, count)
}
