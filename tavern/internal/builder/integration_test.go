package builder_test

import (
	"context"
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

	// 2. Setup GraphQL server with authentication bypass
	git := tomes.NewGitImporter(graph)
	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": handler.NewDefaultServer(graphql.NewSchema(graph, git)),
		},
		tavernhttp.WithAuthenticationBypass(graph),
	)
	gqlClient := client.New(srv, client.Path("/graphql"))

	// 3. Register a builder via GraphQL mutation
	var registerResp struct {
		RegisterBuilder struct {
			Builder struct {
				ID string
			}
			MtlsCert string
			Config   string
		}
	}

	err := gqlClient.Post(`mutation registerNewBuilder($input: CreateBuilderInput!) {
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

	// 4. Parse the returned YAML config
	cfg, err := builder.ParseConfigBytes([]byte(registerResp.RegisterBuilder.Config))
	require.NoError(t, err)
	assert.Contains(t, cfg.SupportedTargets, "linux")
	assert.Contains(t, cfg.SupportedTargets, "macos")
	assert.NotEmpty(t, cfg.MTLS)
	assert.Equal(t, "https://tavern.example.com:443", cfg.Upstream)

	// 5. Verify builder exists in DB
	builders, err := graph.Builder.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, builders, 1)

	// 6. Setup builder gRPC server via bufconn
	lis := bufconn.Listen(1024 * 1024)
	grpcSrv := grpc.NewServer()
	builderSrv := builder.New(graph)
	builderpb.RegisterBuilderServer(grpcSrv, builderSrv)

	go func() {
		if err := grpcSrv.Serve(lis); err != nil {
			t.Logf("gRPC server exited: %v", err)
		}
	}()
	defer grpcSrv.Stop()

	// 7. Connect gRPC client via bufconn
	conn, err := grpc.DialContext(ctx, "bufnet",
		grpc.WithContextDialer(func(context.Context, string) (net.Conn, error) {
			return lis.Dial()
		}),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	require.NoError(t, err)
	defer conn.Close()

	// 8. Call Ping on the builder gRPC service
	builderClient := builderpb.NewBuilderClient(conn)
	pingResp, err := builderClient.Ping(ctx, &builderpb.PingRequest{})
	require.NoError(t, err)
	require.NotNil(t, pingResp)
}
