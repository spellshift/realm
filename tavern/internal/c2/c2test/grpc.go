package c2test

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"errors"
	"net"
	"testing"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gocloud.dev/pubsub"
	_ "gocloud.dev/pubsub/mempubsub"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"
	"realm.pub/tavern/internal/c2"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/portals/mux"
)

func New(t *testing.T) (c2pb.C2Client, *ent.Client, func()) {
	t.Helper()
	ctx := context.Background()

	// TestDB Config
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)

	// Ent Client
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())

	// gRPC Mux
	var (
		pubOutput = "mem://shell_output"
		subInput  = "mem://shell_input"
	)
	grpcOutTopic, err := pubsub.OpenTopic(ctx, pubOutput)
	require.NoError(t, err)
	_, err = pubsub.OpenTopic(ctx, subInput)
	require.NoError(t, err)
	grpcInSub, err := pubsub.OpenSubscription(ctx, subInput)
	require.NoError(t, err)
	grpcShellMux := stream.NewMux(grpcOutTopic, grpcInSub)
	portalMux := mux.New(mux.WithInMemoryDriver())

	// Generate test ED25519 key for JWT signing
	testPubKey, testPrivKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	// gRPC Server
	lis := bufconn.Listen(1024 * 1024 * 10)
	baseSrv := grpc.NewServer()
	c2pb.RegisterC2Server(baseSrv, c2.New(graph, grpcShellMux, portalMux, testPubKey, testPrivKey))

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

	return c2pb.NewC2Client(conn), graph, func() {
		assert.NoError(t, lis.Close())
		baseSrv.Stop()
		assert.NoError(t, graph.Close())
		if err := <-grpcErrCh; err != nil && !errors.Is(err, grpc.ErrServerStopped) {
			t.Fatalf("failed to serve grpc: %v", err)
		}
	}
}
