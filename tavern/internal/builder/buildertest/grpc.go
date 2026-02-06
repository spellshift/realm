package buildertest

import (
	"context"
	"errors"
	"net"
	"testing"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
)

// New creates a new Builder gRPC test server and returns a client, ent client, and cleanup function.
func New(t *testing.T) (builderpb.BuilderClient, *ent.Client, func()) {
	t.Helper()

	// TestDB Config
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)

	// Ent Client
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())

	// gRPC Server
	lis := bufconn.Listen(1024 * 1024 * 10)
	baseSrv := grpc.NewServer()
	builderpb.RegisterBuilderServer(baseSrv, builder.New(graph))

	grpcErrCh := make(chan error, 1)
	go func() {
		grpcErrCh <- baseSrv.Serve(lis)
	}()

	conn, err := grpc.DialContext(
		context.Background(),
		"",
		grpc.WithContextDialer(func(context.Context, string) (net.Conn, error) {
			return lis.Dial()
		}),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	require.NoError(t, err)

	return builderpb.NewBuilderClient(conn), graph, func() {
		assert.NoError(t, lis.Close())
		baseSrv.Stop()
		assert.NoError(t, graph.Close())
		if err := <-grpcErrCh; err != nil && !errors.Is(err, grpc.ErrServerStopped) {
			t.Fatalf("failed to serve grpc: %v", err)
		}
	}
}
