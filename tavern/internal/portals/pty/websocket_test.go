package pty_test

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/http/shell"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/portals/pty"
	"realm.pub/tavern/portals/portalpb"

	// Needed for ent to work
	_ "github.com/mattn/go-sqlite3"
	_ "realm.pub/tavern/internal/ent/runtime"
)

// ptyTestEnv bundles all objects needed for a PTY websocket e2e test.
type ptyTestEnv struct {
	EntClient *ent.Client
	Mux       *mux.Mux
	Server    *httptest.Server
	WSURL     string
	TaskEnt   *ent.Task
}

func (env *ptyTestEnv) Close() {
	env.Server.Close()
	env.EntClient.Close()
}

// setupPTYTestEnv creates an in-memory DB, portal mux, and HTTP test server
// running the PTY websocket handler. This is the minimal setup for an e2e test
// that exercises the output deduplication path.
func setupPTYTestEnv(t *testing.T) *ptyTestEnv {
	t.Helper()
	ctx := context.Background()

	entClient := enttest.OpenTempDB(t)
	portalMux := mux.New()

	// --- seed entities ---
	u, err := entClient.User.Create().
		SetOauthID("pty-test-user").
		SetName("PTY Test User").
		SetPhotoURL("http://example.com/photo.jpg").
		SetSessionToken("pty-test-token").
		Save(ctx)
	require.NoError(t, err)

	h, err := entClient.Host.Create().
		SetName("pty-host").
		SetIdentifier("pty-host-1").
		SetPrimaryIP("127.0.0.1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		Save(ctx)
	require.NoError(t, err)

	b, err := entClient.Beacon.Create().
		SetIdentifier("pty-beacon").
		SetName("pty-beacon").
		SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
		Save(ctx)
	require.NoError(t, err)

	tom, err := entClient.Tome.Create().
		SetName("pty-tome").
		SetDescription("pty tome").
		SetAuthor("pty-user").
		SetTactic("COMMAND_AND_CONTROL").
		SetEldritch("print('hello')").
		Save(ctx)
	require.NoError(t, err)

	q, err := entClient.Quest.Create().
		SetName("pty-quest").
		SetCreator(u).
		SetTome(tom).
		Save(ctx)
	require.NoError(t, err)

	task, err := entClient.Task.Create().
		SetBeacon(b).
		SetQuest(q).
		Save(ctx)
	require.NoError(t, err)

	// HTTP server with the PTY handler
	handler := pty.NewHandler(entClient, portalMux)
	server := httptest.NewServer(http.HandlerFunc(handler.ServeHTTP))
	wsURL := "ws" + strings.TrimPrefix(server.URL, "http")

	return &ptyTestEnv{
		EntClient: entClient,
		Mux:       portalMux,
		Server:    server,
		WSURL:     wsURL,
		TaskEnt:   task,
	}
}

// readOutputMessages reads output messages from the websocket until the
// deadline, returning all output strings received.
func readOutputMessages(t *testing.T, ws *websocket.Conn, deadline time.Time) []string {
	t.Helper()
	var outputs []string
	ws.SetReadDeadline(deadline)
	for {
		if time.Now().After(deadline) {
			break
		}
		_, data, err := ws.ReadMessage()
		if err != nil {
			break
		}
		var generic struct {
			Kind string `json:"kind"`
		}
		if err := json.Unmarshal(data, &generic); err != nil {
			continue
		}
		if generic.Kind == shell.WebsocketMessageKindOutput {
			var msg shell.WebsocketTaskOutputMessage
			if err := json.Unmarshal(data, &msg); err != nil {
				continue
			}
			outputs = append(outputs, msg.Output)
		}
	}
	return outputs
}

// TestPTYOutputDeduplication is an end-to-end test that verifies PTY output
// motes delivered through the portal mux are deduplicated correctly.
//
// Scenario: The mux's Publish() fast-path dispatches each mote to local
// subscribers immediately. The global pubsub receiveLoop (started by
// OpenPortal) then dispatches the same mote a second time. Without
// deduplication the websocket client would see doubled output. This test
// publishes several output motes and verifies each payload appears exactly once.
func TestPTYOutputDeduplication(t *testing.T) {
	env := setupPTYTestEnv(t)
	defer env.Close()
	ctx := context.Background()

	// 1. Create Portal (simulates agent registration)
	portalID, agentCleanup, err := env.Mux.CreatePortal(ctx, env.EntClient, env.TaskEnt.ID, 0)
	require.NoError(t, err)
	defer agentCleanup()

	// 2. Connect via WebSocket (user side)
	url := fmt.Sprintf("%s?portal_id=%d", env.WSURL, portalID)
	ws, _, err := websocket.DefaultDialer.Dial(url, nil)
	require.NoError(t, err)
	defer ws.Close()

	// Give the handler time to set up the session and subscribe
	time.Sleep(200 * time.Millisecond)

	// 3. Look up the ShellPivot to get the streamID
	pivots, err := env.EntClient.ShellPivot.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, pivots, 1, "expected exactly one ShellPivot")
	streamID := pivots[0].StreamID

	// 4. Simulate agent PTY output — three motes with sequential seq_ids
	// The agent pty.rs starts seq_id at 1 (increments before use).
	topicOut := env.Mux.TopicOut(portalID)
	payloads := []string{"hello", " world", "!\n"}
	for i, payload := range payloads {
		mote := &portalpb.Mote{
			StreamId: streamID,
			SeqId:    uint64(i + 1), // Agent starts at 1
			Payload: &portalpb.Mote_Bytes{
				Bytes: &portalpb.BytesPayload{
					Data: []byte(payload),
					Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_PTY,
				},
			},
		}
		err := env.Mux.Publish(ctx, topicOut, mote)
		require.NoError(t, err)
	}

	// 5. Collect output from the websocket
	// Allow enough time for both fast-path and slow-path deliveries.
	outputs := readOutputMessages(t, ws, time.Now().Add(3*time.Second))

	// 6. Verify: each payload should appear exactly once, in order
	combined := strings.Join(outputs, "")
	expected := strings.Join(payloads, "")
	assert.Equal(t, expected, combined,
		"PTY output should be deduplicated; got %d output messages: %v", len(outputs), outputs)
}

// TestPTYOutputOrdering verifies that when motes arrive in order from a
// single source, the websocket delivers them in the same order.
func TestPTYOutputOrdering(t *testing.T) {
	env := setupPTYTestEnv(t)
	defer env.Close()
	ctx := context.Background()

	portalID, agentCleanup, err := env.Mux.CreatePortal(ctx, env.EntClient, env.TaskEnt.ID, 0)
	require.NoError(t, err)
	defer agentCleanup()

	url := fmt.Sprintf("%s?portal_id=%d", env.WSURL, portalID)
	ws, _, err := websocket.DefaultDialer.Dial(url, nil)
	require.NoError(t, err)
	defer ws.Close()

	time.Sleep(200 * time.Millisecond)

	pivots, err := env.EntClient.ShellPivot.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, pivots, 1)
	streamID := pivots[0].StreamID

	topicOut := env.Mux.TopicOut(portalID)

	// Send 10 sequential motes
	const numMotes = 10
	for i := 1; i <= numMotes; i++ {
		mote := &portalpb.Mote{
			StreamId: streamID,
			SeqId:    uint64(i),
			Payload: &portalpb.Mote_Bytes{
				Bytes: &portalpb.BytesPayload{
					Data: []byte(fmt.Sprintf("line-%d\n", i)),
					Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_PTY,
				},
			},
		}
		err := env.Mux.Publish(ctx, topicOut, mote)
		require.NoError(t, err)
		// Small delay between publishes to simulate real PTY output
		time.Sleep(10 * time.Millisecond)
	}

	// Collect output
	outputs := readOutputMessages(t, ws, time.Now().Add(3*time.Second))
	combined := strings.Join(outputs, "")

	// Build expected string
	var expectedBuilder strings.Builder
	for i := 1; i <= numMotes; i++ {
		expectedBuilder.WriteString(fmt.Sprintf("line-%d\n", i))
	}

	assert.Equal(t, expectedBuilder.String(), combined,
		"PTY output must be in order and each line should appear exactly once")
}
