package stream_test

import (
	"context"
	"fmt"
	"net/http/httptest"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gocloud.dev/pubsub"
	_ "gocloud.dev/pubsub/mempubsub"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/http/stream"

	_ "github.com/mattn/go-sqlite3"
)

func TestNewShellHandler(t *testing.T) {
	// Setup Ent Client
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	// Topic for messages going TO the websocket (server -> shell)
	outputTopicName := fmt.Sprintf("mem://websocket-output-%d", time.Now().UnixNano())
	outputTopic, err := pubsub.OpenTopic(ctx, outputTopicName)
	require.NoError(t, err)
	defer outputTopic.Shutdown(ctx)
	outputSub, err := pubsub.OpenSubscription(ctx, outputTopicName)
	require.NoError(t, err)
	defer outputSub.Shutdown(ctx)

	// Topic for messages coming FROM the websocket (shell -> server)
	inputTopicName := fmt.Sprintf("mem://websocket-input-%d", time.Now().UnixNano())
	inputTopic, err := pubsub.OpenTopic(ctx, inputTopicName)
	require.NoError(t, err)
	defer inputTopic.Shutdown(ctx)
	inputSub, err := pubsub.OpenSubscription(ctx, inputTopicName)
	require.NoError(t, err)
	defer inputSub.Shutdown(ctx)

	// Create a test user
	user, err := graph.User.Create().SetName("test-user").SetOauthID("test-oauth-id").SetPhotoURL("http://example.com/photo.jpg").Save(ctx)
	require.NoError(t, err)

	// Create a test host
	host, err := graph.Host.Create().SetIdentifier("test-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).Save(ctx)
	require.NoError(t, err)

	// Create a test beacon
	beacon, err := graph.Beacon.Create().SetHost(host).SetTransport(c2pb.Beacon_TRANSPORT_UNSPECIFIED).Save(ctx)
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
	mux := stream.NewMux(inputTopic, outputSub)
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
	err = outputTopic.Send(ctx, &pubsub.Message{
		Body:     testMessage,
		Metadata: map[string]string{"id": strconv.Itoa(shell.ID)},
	})
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
