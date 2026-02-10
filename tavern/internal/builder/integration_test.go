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
			builder.NewAuthInterceptor(caCert, graph),
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

	// 8. Test: Unauthenticated request should be rejected
	t.Run("unauthenticated request rejected", func(t *testing.T) {
		conn, err := grpc.NewClient("passthrough:///bufnet",
			grpc.WithContextDialer(bufDialer),
			grpc.WithTransportCredentials(insecure.NewCredentials()),
		)
		require.NoError(t, err)
		defer conn.Close()

		unauthClient := builderpb.NewBuilderClient(conn)
		_, err = unauthClient.Ping(ctx, &builderpb.PingRequest{})
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
		pingResp, err := authClient.Ping(ctx, &builderpb.PingRequest{})
		require.NoError(t, err)
		require.NotNil(t, pingResp)
	})
}
