package stream_test

import (
	"context"
	"net/http/httptest"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/xpubsub"

	_ "github.com/mattn/go-sqlite3"
)

func TestNewShellHandler(t *testing.T) {
	// Setup Ent Client
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	client, err := xpubsub.NewClient(ctx, "test-project", true)
	require.NoError(t, err)
	defer client.Close()

	// Topic for messages going TO the websocket (server -> shell)
	outputTopic := "websocket-output"
	outputSubName := "websocket-output-sub"
	require.NoError(t, client.EnsureTopic(ctx, outputTopic))
	require.NoError(t, client.EnsureSubscription(ctx, outputTopic, outputSubName, 0))

	outputPub := client.NewPublisher(outputTopic)
	defer outputPub.Close()
	outputSub := client.NewSubscriber(outputSubName)
	defer outputSub.Close()

	// Topic for messages coming FROM the websocket (shell -> server)
	inputTopic := "websocket-input"
	inputSubName := "websocket-input-sub"
	require.NoError(t, client.EnsureTopic(ctx, inputTopic))
	require.NoError(t, client.EnsureSubscription(ctx, inputTopic, inputSubName, 0))

	inputPub := client.NewPublisher(inputTopic)
	defer inputPub.Close()
	inputSub := client.NewSubscriber(inputSubName)
	defer inputSub.Close()

	// Create a test user
	user, err := graph.User.Create().SetName("test-user").SetOauthID("test-oauth-id").SetPhotoURL("http://example.com/photo.jpg").Save(ctx)
	require.NoError(t, err)

	// Create a test host
	host, err := graph.Host.Create().SetIdentifier("test-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).Save(ctx)
	require.NoError(t, err)

	// Create a test beacon
	beacon, err := graph.Beacon.Create().SetHost(host).SetTransport(c2pb.Transport_TRANSPORT_UNSPECIFIED).Save(ctx)
	require.NoError(t, err)

	// Create a test tome
	tome, err := graph.Tome.Create().
		SetName("test-tome").
		SetDescription("test-description").
		SetAuthor("test-author").
		SetEldritch("test-eldritch").
		SetUploader(user).
		Save(ctx)
	require.NoError(t, err)

	// Create a test quest
	quest, err := graph.Quest.Create().SetName("test-quest").SetTome(tome).SetCreator(user).Save(ctx)
	require.NoError(t, err)

	// Create a test task
	task, err := graph.Task.Create().SetQuest(quest).SetBeacon(beacon).Save(ctx)
	require.NoError(t, err)

	// Create Mux with the correct topics
	// Mux reads from input (pub), writes to output (sub)?
	// Wait, Mux in websocket test:
	// NewMux(inputTopic, outputSub)
	// inputTopic is *pubsub.Topic (publisher)
	// outputSub is *pubsub.Subscription (subscriber)
	// The Mux.send sends to `inputTopic`.
	// The Mux.Register reads from `outputSub`.

	// Server -> Shell: Published to outputTopic?
	// Websocket writes to `inputTopic` (via mux.send?) No.
	// `mux.send` sends to `mux.pub`.
	// `NewShellHandler` uses `mux`.
	// When WS receives message, it sends to WS client.
	// When WS reads message from client, it calls `mux.send`.

	// So WS reads from `mux.sub` (which should be messages intended for Shell).
	// WS writes to `mux.pub` (which are messages FROM Shell).

	// The test setup:
	// outputTopic/Sub is for Server -> Shell?
	// inputTopic/Sub is for Shell -> Server?

	// In test:
	// outputTopic.Send -> received by WS client.
	// So WS should listen to outputSub.
	// So mux.sub should be outputSub.

	// WS WriteMessage -> received on inputSub.
	// So mux.pub should be inputTopic.

	// NewMux(pub, sub)
	// NewMux(inputPub, outputSub)

	mux := stream.NewMux(inputPub, outputSub)
	go mux.Start(ctx)

	// Create a test shell
	shell, err := graph.Shell.Create().SetData([]byte("test-data")).SetOwner(user).SetTask(task).SetBeacon(beacon).Save(ctx)
	require.NoError(t, err)

	// Create a test server
	handler := stream.NewShellHandler(graph, mux)
	server := httptest.NewServer(handler)
	defer server.Close()

	// Create a websocket client
	wsURL := "ws" + strings.TrimPrefix(server.URL, "http") + "?shell_id=" + strconv.Itoa(shell.ID)
	ws, _, err := websocket.DefaultDialer.Dial(wsURL, nil)
	require.NoError(t, err)
	defer ws.Close()

	// Test writing to the websocket (server -> shell)
	testMessage := []byte("hello from server")
	// Test publishes to outputPub (Server sends to Shell)
	err = outputPub.Publish(ctx, testMessage, map[string]string{"id": strconv.Itoa(shell.ID)})
	require.NoError(t, err)

	_, p, err := ws.ReadMessage()
	assert.NoError(t, err)

	assert.Equal(t, testMessage, p)

	// Test reading from the websocket (shell -> server)
	// Client sends raw bytes
	readMessage := []byte("hello from shell")
	err = ws.WriteMessage(websocket.TextMessage, readMessage)
	require.NoError(t, err)

	// Now, we expect the message on the input subscription
	msg, err := inputSub.Receive(ctx)
	require.NoError(t, err, "timed out waiting for message from websocket")

	// The body sent to pubsub should be the raw bytes
	assert.Equal(t, readMessage, msg.Body)
	assert.Equal(t, "data", msg.Metadata[stream.MetadataMsgKind])
	msg.Ack()
}
