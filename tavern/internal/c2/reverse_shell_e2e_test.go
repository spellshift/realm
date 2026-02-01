package c2_test

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"net"
	"net/http/httptest"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"
	"realm.pub/tavern/internal/c2"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/xpubsub"

	_ "github.com/mattn/go-sqlite3"
)

func TestReverseShell_E2E(t *testing.T) {
	// Setup Ent Client
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	// C2 Server Setup
	lis := bufconn.Listen(1024 * 1024)
	s := grpc.NewServer()

	// Pub/Sub Topics
	// The wsMux will be used by websockets to subscribe to shell output and publish new input.
	// The grpcMux will be used by gRPC to subscribe to shell input and publish new output.

	client, err := xpubsub.NewClient(ctx, "test-project", true)
	require.NoError(t, err)
	defer client.Close()

	inputTopic := "e2e-input"
	inputSubName := "e2e-input-sub"
	outputTopic := "e2e-output"
	outputSubName := "e2e-output-sub"

	require.NoError(t, client.EnsureTopic(ctx, inputTopic))
	require.NoError(t, client.EnsureSubscription(ctx, inputTopic, inputSubName, 0))
	require.NoError(t, client.EnsureTopic(ctx, outputTopic))
	require.NoError(t, client.EnsureSubscription(ctx, outputTopic, outputSubName, 0))

	pubInput := client.NewPublisher(inputTopic)
	defer pubInput.Close()
	subInput := client.NewSubscriber(inputSubName)
	defer subInput.Close()

	pubOutput := client.NewPublisher(outputTopic)
	defer pubOutput.Close()
	subOutput := client.NewSubscriber(outputSubName)
	defer subOutput.Close()

	// wsMux: reads from outputSub (shell output), writes to inputPub (shell input)
	// Wait, setup:
	// wsMux := stream.NewMux(pubInput, subOutput)
	// grpcMux := stream.NewMux(pubOutput, subInput)
	// pubInput = "e2e-input". subOutput = "e2e-output-sub" (of e2e-output topic)
	// pubOutput = "e2e-output". subInput = "e2e-input-sub" (of e2e-input topic)

	// Server (gRPC): writes to pubOutput (e2e-output). Reads from subInput (e2e-input-sub).
	// WS (Client): writes to pubInput (e2e-input). Reads from subOutput (e2e-output-sub).

	wsMux := stream.NewMux(pubInput, subOutput)
	grpcMux := stream.NewMux(pubOutput, subInput)

	portalMux, err := mux.New(ctx, mux.WithInMemoryDriver())
	require.NoError(t, err)

	// Generate test ED25519 key for JWT signing
	testPubKey, testPrivKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	go wsMux.Start(ctx)
	go grpcMux.Start(ctx)

	c2pb.RegisterC2Server(s, c2.New(graph, grpcMux, portalMux, testPubKey, testPrivKey))
	go func() {
		if err := s.Serve(lis); err != nil {
			t.Logf("Server exited with error: %v", err)
		}
	}()

	// gRPC Client Setup
	conn, err := grpc.DialContext(ctx, "bufnet", grpc.WithContextDialer(func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}), grpc.WithTransportCredentials(insecure.NewCredentials()))
	require.NoError(t, err)
	defer conn.Close()

	c2Client := c2pb.NewC2Client(conn)

	// Create test entities
	user, err := graph.User.Create().SetName("test-user").SetOauthID("test-oauth-id").SetPhotoURL("http://example.com/photo.jpg").Save(ctx)
	require.NoError(t, err)
	host, err := graph.Host.Create().SetIdentifier("test-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).Save(ctx)
	require.NoError(t, err)
	beacon, err := graph.Beacon.Create().SetHost(host).SetTransport(c2pb.Transport_TRANSPORT_UNSPECIFIED).Save(ctx)
	require.NoError(t, err)
	tome, err := graph.Tome.Create().SetName("test-tome").SetDescription("test-desc").SetAuthor("test-author").SetEldritch("test-eldritch").SetUploader(user).Save(ctx)
	require.NoError(t, err)
	quest, err := graph.Quest.Create().SetName("test-quest").SetTome(tome).SetCreator(user).Save(ctx)
	require.NoError(t, err)
	task, err := graph.Task.Create().SetQuest(quest).SetBeacon(beacon).Save(ctx)
	require.NoError(t, err)

	// WebSocket Server Setup
	handler := stream.NewShellHandler(graph, wsMux)
	httpServer := httptest.NewServer(handler)
	defer httpServer.Close()

	// gRPC Stream
	gRPCStream, err := c2Client.ReverseShell(ctx)
	require.NoError(t, err)

	// Register gRPC stream with task ID
	err = gRPCStream.Send(&c2pb.ReverseShellRequest{
		Context: &c2pb.TaskContext{TaskId: int64(task.ID)},
	})
	require.NoError(t, err)

	// Find the shell created by the gRPC service
	var shellID int
	require.Eventually(t, func() bool {
		shells, err := task.QueryShells().All(ctx)
		if err != nil || len(shells) == 0 {
			return false
		}
		shellID = shells[0].ID
		return true
	}, 5*time.Second, 100*time.Millisecond, "shell was not created in time")

	// WebSocket Client Setup
	wsURL := "ws" + strings.TrimPrefix(httpServer.URL, "http") + "?shell_id=" + strconv.Itoa(shellID)
	ws, _, err := websocket.DefaultDialer.Dial(wsURL, nil)
	require.NoError(t, err)
	defer ws.Close()

	// Test message from gRPC to WebSocket
	grpcMsg := []byte("hello from grpc")
	err = gRPCStream.Send(&c2pb.ReverseShellRequest{
		Kind: c2pb.ReverseShellMessageKind_REVERSE_SHELL_MESSAGE_KIND_DATA,
		Data: grpcMsg,
	})
	require.NoError(t, err)

	_, wsMsg, err := ws.ReadMessage()
	assert.NoError(t, err)
	assert.Equal(t, grpcMsg, wsMsg)

	// Test message from WebSocket to gRPC
	wsMsgToSend := []byte("hello from websocket")
	err = ws.WriteMessage(websocket.BinaryMessage, wsMsgToSend)
	require.NoError(t, err)

	grpcResp, err := gRPCStream.Recv()
	require.NoError(t, err)
	assert.Equal(t, wsMsgToSend, grpcResp.Data)
}
