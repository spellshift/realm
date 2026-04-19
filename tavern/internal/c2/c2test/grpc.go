package c2test

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"net"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
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

func New(t *testing.T) (c2pb.C2Client, *ent.Client, func(), string) {
	t.Helper()
	ctx := context.Background()

	// Ent Client
	graph := enttest.OpenTempDB(t)

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
	portalMux := mux.New()

	// Generate test ED25519 key for JWT signing
	testPubKey, testPrivKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	// Generate a signed JWT string for tests
	claims := jwt.MapClaims{
		"iat": time.Now().Unix(),
		"exp": time.Now().Add(1 * time.Hour).Unix(),
	}
	testToken := ""
	{
		token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
		s, err := token.SignedString(testPrivKey)
		require.NoError(t, err)
		testToken = s
	}

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
		// Stop the server first: this sets the quit flag and closes all
		// registered listeners, so Serve() returns nil instead of a raw
		// "closed" error from the listener.  Calling lis.Close() before
		// Stop() is a race — Serve() may observe the closed listener
		// before the quit flag is set and return an unexpected error.
		baseSrv.Stop()
		conn.Close()
		assert.NoError(t, graph.Close())
		if err := <-grpcErrCh; err != nil {
			t.Fatalf("failed to serve grpc: %v", err)
		}
	}, testToken
}
