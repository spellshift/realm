package c2test

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"errors"
	"net"
	"testing"
	"time"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"
	"realm.pub/tavern/internal/c2"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/xpubsub"
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
		pubOutput = "shell_output"
		subInput  = "shell_input"
		subName   = "shell_input_sub"
	)

	// Create xpubsub Client
	client, err := xpubsub.NewClient(ctx, "test-project", true)
	require.NoError(t, err)

	// Ensure Topics and Subscription
	require.NoError(t, client.EnsureTopic(ctx, pubOutput))
	require.NoError(t, client.EnsureTopic(ctx, subInput))
	require.NoError(t, client.EnsureSubscription(ctx, subInput, subName, 24*time.Hour))

	grpcOutPub := client.NewPublisher(pubOutput)
	grpcInSub := client.NewSubscriber(subName)

	grpcShellMux := stream.NewMux(grpcOutPub, grpcInSub)

	// portalMux
	portalMux, err := mux.New(ctx, mux.WithInMemoryDriver())
	require.NoError(t, err)

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
		assert.NoError(t, client.Close())
		assert.NoError(t, portalMux.Close())
		if err := <-grpcErrCh; err != nil && !errors.Is(err, grpc.ErrServerStopped) {
			t.Fatalf("failed to serve grpc: %v", err)
		}
	}
}
