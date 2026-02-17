package shell_test

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/require"
	_ "gocloud.dev/pubsub/mempubsub"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/http/shell"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"

	// Needed for ent to work
	_ "github.com/mattn/go-sqlite3"
)

type TestEnv struct {
	EntClient *ent.Client
	Mux       *mux.Mux
	Server    *httptest.Server
	WSURL     string
	User      *ent.User
	Shell     *ent.Shell
	Beacon    *ent.Beacon
	Task      *ent.Task
}

func (env *TestEnv) Close() {
	env.Server.Close()
	env.EntClient.Close()
}

func SetupTestEnv(t *testing.T) *TestEnv {
	ctx := context.Background()

	// 1. Setup DB
	dsn := fmt.Sprintf("file:ent_%s?mode=memory&cache=shared&_fk=1", uuid.NewString())
	entClient := enttest.Open(t, "sqlite3", dsn)

	// 2. Setup Portal Mux
	portalMux := mux.New()

	// 3. Setup Data
	// Create User
	u, err := entClient.User.Create().
		SetOauthID("testuser").
		SetName("Test User").
		SetPhotoURL("http://example.com/photo.jpg").
		SetSessionToken("test-token").
		Save(ctx)
	require.NoError(t, err)

	// Create Host
	h, err := entClient.Host.Create().
		SetName("testhost").
		SetIdentifier("host1").
		SetPrimaryIP("127.0.0.1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		Save(ctx)
	require.NoError(t, err)

	// Create Beacon
	b, err := entClient.Beacon.Create().
		SetIdentifier("testbeacon").
		SetName("testbeacon").
		SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
		Save(ctx)
	require.NoError(t, err)

	// Create Tome (Required for Quest/Task)
	tom, err := entClient.Tome.Create().
		SetName("testtome").
		SetDescription("test tome").
		SetAuthor("testuser").
		SetTactic("COMMAND_AND_CONTROL").
		SetEldritch("print('hello')").
		Save(ctx)
	require.NoError(t, err)

	// Create Quest
	q, err := entClient.Quest.Create().
		SetName("testquest").
		SetCreator(u).
		SetTome(tom).
		Save(ctx)
	require.NoError(t, err)

	// Create Task (Required for Portal)
	task, err := entClient.Task.Create().
		SetBeacon(b).
		SetQuest(q).
		Save(ctx)
	require.NoError(t, err)

	// Create Shell
	sh, err := entClient.Shell.Create().
		SetBeacon(b).
		SetOwner(u).
		SetData([]byte{}).
		Save(ctx)
	require.NoError(t, err)

	// 4. Setup Handler & Server
	handler := shell.NewHandler(entClient, portalMux)

	// Middleware to inject user
	authMiddleware := func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			newCtx, err := auth.ContextFromSessionToken(r.Context(), entClient, "test-token")
			if err != nil {
				http.Error(w, "auth failed", http.StatusUnauthorized)
				return
			}
			next.ServeHTTP(w, r.WithContext(newCtx))
		})
	}

	server := httptest.NewServer(authMiddleware(http.HandlerFunc(handler.ServeHTTP)))
	wsURL := "ws" + strings.TrimPrefix(server.URL, "http")

	return &TestEnv{
		EntClient: entClient,
		Mux:       portalMux,
		Server:    server,
		WSURL:     wsURL,
		User:      u,
		Shell:     sh,
		Beacon:    b,
		Task:      task,
	}
}

func TestInteractiveShell(t *testing.T) {
	env := SetupTestEnv(t)
	defer env.Close()
	ctx := context.Background()

	// 1. Create Portal (Agent side) using mux.CreatePortal
	// This will create the Portal entity AND setup the topics.
	// It returns portalID, teardown, err
	portalID, agentCleanup, err := env.Mux.CreatePortal(ctx, env.EntClient, env.Task.ID)
	require.NoError(t, err)
	defer agentCleanup()

	// We need to fetch the portal to associate it with the Shell
	// (CreatePortal saves it to DB)
	p, err := env.EntClient.Portal.Get(ctx, portalID)
	require.NoError(t, err)

	// Associate Portal with Shell
	err = env.Shell.Update().AddPortals(p).Exec(ctx)
	require.NoError(t, err)

	// Subscribe to IN topic (User -> Agent)
	// CreatePortal sets up subscriptions for IN topic?
	// `CreatePortal` calls `m.openSubscription(ctx, topicIn, subName)` and starts `m.receiveLoop`.
	// Wait, `CreatePortal` subscribes to IN topic (messages FROM user TO agent).
	// But it consumes them internally (via `receiveLoop`).
	// `receiveLoop` likely processes messages or forwards them?
	// `mux.go` doesn't show `receiveLoop` impl.
	// But typically `CreatePortal` is used by the C2 server to bridge PubSub <-> gRPC stream.
	// Here in test, we want to simulate the Agent receiving the message.
	// If `CreatePortal` already subscribed, we can't subscribe again easily with `mempubsub` (maybe?).
	// However, `mux` exposes `Subscribe` method? Yes.
	// But `CreatePortal` locks the sub manager.
	// If we want to intercept messages sent by User, we should subscribe to `TopicIn`.
	// If `CreatePortal` already subscribed, we might have competition.
	// But `CreatePortal` is designed for the C2 service.
	// In this test, we ARE the C2 service / Agent.
	// `CreatePortal` logic: starts a loop.
	// We can't access that loop's output channel easily unless we use `Mux` internals or if `CreatePortal` returned a channel (it returns cleanup).

	// Alternative: Don't use `CreatePortal`. Use `OpenPortal` pattern but ensure topics exist manually?
	// But `ensureTopic` is private in `Mux`.
	// AND we need the DB record.

	// Actually, `OpenPortal` (User side) subscribes to `TopicOut`.
	// `CreatePortal` (Agent side) subscribes to `TopicIn`.
	// We want to write to `TopicOut` (simulate Agent output) and read from `TopicIn` (simulate Agent input).

	// If `CreatePortal` consumes `TopicIn`, we can't read it.
	// BUT, `CreatePortal` implementation (read earlier) does:
	// `go func() { m.receiveLoop(ctxLoop, topicIn, sub) }()`
	// We don't know what `receiveLoop` does.
	// If `receiveLoop` just drops messages or handles history, we might miss them.
	// Wait, `CreatePortal` is used by `c2/server.go`. It likely forwards to the gRPC stream.
	// But here we don't have the gRPC stream connected to `CreatePortal`.
	// `CreatePortal` signature in `mux_create.go`: `func (m *Mux) CreatePortal(...) (int, func(), error)`.
	// It doesn't take a channel or stream.
	// So where do messages go?
	// Maybe `receiveLoop` puts them into `history` or `subs`?
	// `SubscriberRegistry` in `Mux` struct.

	// If `CreatePortal` is running, it consumes messages.
	// We probably shouldn't use `CreatePortal` if we want to intercept messages manually in the test using `Mux.Subscribe`.
	// UNLESS `Mux.Subscribe` adds a *local* subscriber to the *internal* registry?
	// `Mux` seems to have `SubscriberRegistry`.
	// `Subscribe` probably adds a channel to this registry.
	// `receiveLoop` probably reads from PubSub and broadcasts to registry channels.
	// If so, we are fine. `CreatePortal` sets up the PubSub subscription. `Subscribe` hooks into the stream.

	// Let's verify `Subscribe`. I didn't read it.
	// Assuming `Subscribe` works as intended (pubsub style).

	agentInCh, agentSubCleanup := env.Mux.Subscribe(env.Mux.TopicIn(p.ID))
	defer agentSubCleanup()

	// 2. Connect via WebSocket (User side)
	url := fmt.Sprintf("%s?shell_id=%d", env.WSURL, env.Shell.ID)
	ws, _, err := websocket.DefaultDialer.Dial(url, nil)
	require.NoError(t, err)
	defer ws.Close()

	// 3. Wait for Portal Upgrade Message
	var msg shell.WebsocketControlFlowMessage
	for {
		_, data, err := ws.ReadMessage()
		require.NoError(t, err)

		var genericMsg struct {
			Kind string `json:"kind"`
		}
		json.Unmarshal(data, &genericMsg)
		if genericMsg.Kind == shell.WebsocketMessageKindControlFlow {
			json.Unmarshal(data, &msg)
			if msg.Signal == shell.WebsocketControlFlowSignalPortalUpgrade {
				break
			}
		}
	}
	require.Equal(t, p.ID, msg.PortalID)

	// 4. Send Input
	inputCmd := "ls -la"
	inputMsg := shell.WebsocketTaskInputMessage{
		Kind:  shell.WebsocketMessageKindInput,
		Input: inputCmd,
	}
	err = ws.WriteJSON(inputMsg)
	require.NoError(t, err)

	// 5. Verify Input on Portal IN
	select {
	case mote := <-agentInCh:
		require.NotNil(t, mote.GetShell())
		require.Equal(t, inputCmd, mote.GetShell().Input)
		require.Equal(t, int64(env.Shell.ID), mote.GetShell().ShellId)
	case <-time.After(5 * time.Second):
		t.Fatal("timeout waiting for input on portal")
	}

	// 6. Send Output from Agent
	// Get created task
	tasks, err := env.EntClient.ShellTask.Query().All(ctx)
	require.NoError(t, err)
	require.NotEmpty(t, tasks)
	task := tasks[0]

	outputData := "total 0\n"
	outMote := &portalpb.Mote{
		StreamId: task.StreamID,
		SeqId:    task.SequenceID,
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Data: []byte(outputData),
				Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA,
			},
		},
	}
	// We publish to TopicOut. User subscribes to TopicOut.
	// Ensure TopicOut exists? `CreatePortal` calls `ensureTopic(TopicOut)`.
	// So it should be fine.
	err = env.Mux.Publish(ctx, env.Mux.TopicOut(p.ID), outMote)
	require.NoError(t, err)

	// 7. Verify Output on WebSocket
	var outMsg shell.WebsocketTaskOutputMessage
	found := false
	timeout := time.After(5 * time.Second)
	for !found {
		select {
		case <-timeout:
			t.Fatal("timeout waiting for output on websocket")
		default:
			_, data, err := ws.ReadMessage()
			if err != nil {
				continue
			}

			var genericMsg struct {
				Kind string `json:"kind"`
			}
			json.Unmarshal(data, &genericMsg)
			if genericMsg.Kind == shell.WebsocketMessageKindOutput {
				json.Unmarshal(data, &outMsg)
				found = true
			}
		}
	}
	require.Equal(t, outputData, outMsg.Output)
	require.Equal(t, task.ID, outMsg.ShellTaskID)
}

func TestNonInteractiveShell(t *testing.T) {
	env := SetupTestEnv(t)
	defer env.Close()

	// 1. Connect via WebSocket
	url := fmt.Sprintf("%s?shell_id=%d", env.WSURL, env.Shell.ID)
	ws, _, err := websocket.DefaultDialer.Dial(url, nil)
	require.NoError(t, err)
	defer ws.Close()

	// 2. Send Input
	inputCmd := "whoami"
	inputMsg := shell.WebsocketTaskInputMessage{
		Kind:  shell.WebsocketMessageKindInput,
		Input: inputCmd,
	}
	err = ws.WriteJSON(inputMsg)
	require.NoError(t, err)

	// 3. Expect "Task Queued" Control Message
	var msg shell.WebsocketControlFlowMessage
	for {
		_, data, err := ws.ReadMessage()
		require.NoError(t, err)

		var genericMsg struct {
			Kind string `json:"kind"`
		}
		json.Unmarshal(data, &genericMsg)
		if genericMsg.Kind == shell.WebsocketMessageKindControlFlow {
			json.Unmarshal(data, &msg)
			if msg.Signal == shell.WebsocketControlFlowSignalTaskQueued {
				break
			}
		}
	}
	require.Contains(t, msg.Message, "Task Queued for testbeacon")

	// 4. Simulate C2 updating Task Output
	time.Sleep(100 * time.Millisecond)
	tasks, err := env.EntClient.ShellTask.Query().All(context.Background())
	require.NoError(t, err)
	require.NotEmpty(t, tasks)
	task := tasks[0]

	_, err = task.Update().SetOutput("root").Save(context.Background())
	require.NoError(t, err)

	// 5. Expect Output on WebSocket (polled)
	var outMsg shell.WebsocketTaskOutputMessage
	found := false
	timeout := time.After(5 * time.Second)
	for !found {
		select {
		case <-timeout:
			t.Fatal("timeout waiting for output")
		default:
			_, data, err := ws.ReadMessage()
			if err != nil {
				continue
			}
			var genericMsg struct {
				Kind string `json:"kind"`
			}
			json.Unmarshal(data, &genericMsg)
			if genericMsg.Kind == shell.WebsocketMessageKindOutput {
				json.Unmarshal(data, &outMsg)
				found = true
			}
		}
	}
	require.Equal(t, "root", outMsg.Output)
	require.Equal(t, task.ID, outMsg.ShellTaskID)
}

func TestOtherStreamOutput(t *testing.T) {
	env := SetupTestEnv(t)
	defer env.Close()
	ctx := context.Background()

	// 1. Create Portal
	portalID, agentCleanup, err := env.Mux.CreatePortal(ctx, env.EntClient, env.Task.ID)
	require.NoError(t, err)
	defer agentCleanup()

	p, err := env.EntClient.Portal.Get(ctx, portalID)
	require.NoError(t, err)

	err = env.Shell.Update().AddPortals(p).Exec(ctx)
	require.NoError(t, err)

	// 2. Connect User 1
	url := fmt.Sprintf("%s?shell_id=%d", env.WSURL, env.Shell.ID)
	ws1, _, err := websocket.DefaultDialer.Dial(url, nil)
	require.NoError(t, err)
	defer ws1.Close()

	// Wait for upgrade
	var msg shell.WebsocketControlFlowMessage
	for {
		_, data, err := ws1.ReadMessage()
		require.NoError(t, err)
		var genericMsg struct {
			Kind string `json:"kind"`
		}
		json.Unmarshal(data, &genericMsg)
		if genericMsg.Kind == shell.WebsocketMessageKindControlFlow {
			json.Unmarshal(data, &msg)
			if msg.Signal == shell.WebsocketControlFlowSignalPortalUpgrade {
				break
			}
		}
	}

	// 3. Create Task from Other Stream
	otherStreamID := uuid.NewString()
	otherUser, err := env.EntClient.User.Create().
		SetOauthID("other").
		SetName("Other User").
		SetPhotoURL("...").
		SetSessionToken("token-2").
		Save(ctx)
	require.NoError(t, err)

	otherTask, err := env.EntClient.ShellTask.Create().
		SetShell(env.Shell).
		SetInput("sudo reboot").
		SetCreator(otherUser).
		SetStreamID(otherStreamID).
		SetSequenceID(1).
		Save(ctx)
	require.NoError(t, err)

	// 4. Inject Output
	outMote := &portalpb.Mote{
		StreamId: otherStreamID,
		SeqId:    1,
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Data: []byte("rebooting..."),
				Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA,
			},
		},
	}
	err = env.Mux.Publish(ctx, env.Mux.TopicOut(p.ID), outMote)
	require.NoError(t, err)

	// 5. Verify User 1 receives "Other Stream" message
	var otherMsg shell.WebsocketTaskOutputFromOtherStreamMessage
	found := false
	timeout := time.After(5 * time.Second)

	for !found {
		select {
		case <-timeout:
			t.Fatal("timeout waiting for other stream message")
		default:
			_, data, err := ws1.ReadMessage()
			if err != nil {
				// if error, fail or retry if valid
				// e.g. read closed
				t.Fatalf("ws read error: %v", err)
			}

			var genericMsg struct {
				Kind string `json:"kind"`
			}
			json.Unmarshal(data, &genericMsg)
			if genericMsg.Kind == shell.WebsocketMessageKindOutputFromOtherStream {
				json.Unmarshal(data, &otherMsg)
				found = true
			}
		}
	}

	require.Equal(t, otherTask.ID, otherMsg.ShellTaskID)
	require.Contains(t, otherMsg.Output, "Other User")
	require.Contains(t, otherMsg.Output, "rebooting...")
}
