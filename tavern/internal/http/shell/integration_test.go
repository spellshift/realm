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
	dsn := fmt.Sprintf("file:ent_%s?mode=memory&cache=shared&_fk=1&_busy_timeout=5000", uuid.NewString())
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
	portalID, agentCleanup, err := env.Mux.CreatePortal(ctx, env.EntClient, env.Task.ID, 0)
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
	require.Equal(t, fmt.Sprintf("[+] %s\n%s", inputCmd, outputData), outMsg.Output)
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
	require.Equal(t, fmt.Sprintf("[+] %s\nroot", inputCmd), outMsg.Output)
	require.Equal(t, task.ID, outMsg.ShellTaskID)
}

func TestOtherStreamOutput(t *testing.T) {
	env := SetupTestEnv(t)
	defer env.Close()
	ctx := context.Background()

	// 1. Create Portal
	portalID, agentCleanup, err := env.Mux.CreatePortal(ctx, env.EntClient, env.Task.ID, 0)
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

	longInput := strings.Repeat("A", 100)
	otherTask, err := env.EntClient.ShellTask.Create().
		SetShell(env.Shell).
		SetInput(longInput).
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

	truncatedInput := longInput[:64] + "..."
	expectedFormat := fmt.Sprintf("\x1b[38;5;104m[@%s]\x1b[0m\x1b[38;5;35m[+]\x1b[0m %s\n", "Other User", truncatedInput)
	require.True(t, strings.HasPrefix(otherMsg.Output, expectedFormat), "Output should start with expected format with truncation")
	require.Contains(t, otherMsg.Output, "rebooting...")
}

func TestOtherStreamOutput_Polling(t *testing.T) {
	env := SetupTestEnv(t)
	defer env.Close()
	ctx := context.Background()

	// 1. Connect User 1 (Non-interactive, no portal created)
	url := fmt.Sprintf("%s?shell_id=%d", env.WSURL, env.Shell.ID)
	ws1, _, err := websocket.DefaultDialer.Dial(url, nil)
	require.NoError(t, err)
	defer ws1.Close()

	// 2. Create Task from Other Stream (simulate it was created by another user, and has output in DB)
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
		SetOutput("rebooting...").
		Save(ctx)
	require.NoError(t, err)

	// 3. Verify User 1 receives "Other Stream" message via Polling
	var otherMsg shell.WebsocketTaskOutputFromOtherStreamMessage
	found := false
	timeout := time.After(5 * time.Second)

	for !found {
		select {
		case <-timeout:
			t.Fatal("timeout waiting for other stream message via polling")
		default:
			_, data, err := ws1.ReadMessage()
			if err != nil {
				continue
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
	expectedFormat := fmt.Sprintf("\x1b[38;5;104m[@%s]\x1b[0m\x1b[38;5;35m[+]\x1b[0m %s\n", "Other User", "sudo reboot")
	require.True(t, strings.HasPrefix(otherMsg.Output, expectedFormat), "Output should start with expected format")
	require.Contains(t, otherMsg.Output, "rebooting...")
}

func TestTruncation(t *testing.T) {
	env := SetupTestEnv(t)
	defer env.Close()

	// 1. Connect via WebSocket
	url := fmt.Sprintf("%s?shell_id=%d", env.WSURL, env.Shell.ID)
	ws, _, err := websocket.DefaultDialer.Dial(url, nil)
	require.NoError(t, err)
	defer ws.Close()

	// 2. Send Long Input
	longInput := strings.Repeat("A", 100)
	inputMsg := shell.WebsocketTaskInputMessage{
		Kind:  shell.WebsocketMessageKindInput,
		Input: longInput,
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

	_, err = task.Update().SetOutput("out").Save(context.Background())
	require.NoError(t, err)

	// 5. Expect Output on WebSocket
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
	require.Equal(t, task.ID, outMsg.ShellTaskID)

	expectedPrefix := "[+] " + longInput[:64] + "...\n"
	require.Contains(t, outMsg.Output, expectedPrefix)
}

func TestLargeShellTask(t *testing.T) {
	env := SetupTestEnv(t)
	defer env.Close()
	ctx := context.Background()

	// Create 70KB string (exceeds standard TEXT 64KB limit)
	largeData := strings.Repeat("A", 1024*70)

	// Create ShellTask
	st, err := env.EntClient.ShellTask.Create().
		SetShell(env.Shell).
		SetInput(largeData).
		SetOutput(largeData).
		SetError(largeData).
		SetCreator(env.User).
		SetStreamID("test-stream").
		SetSequenceID(1).
		Save(ctx)
	require.NoError(t, err)

	// Retrieve ShellTask
	retrieved, err := env.EntClient.ShellTask.Get(ctx, st.ID)
	require.NoError(t, err)

	// Verify Data
	require.Equal(t, largeData, retrieved.Input)
	require.Equal(t, largeData, retrieved.Output)
	require.Equal(t, largeData, retrieved.Error)
}

func TestShellHistory_Interactive(t *testing.T) {
	env := SetupTestEnv(t)
	defer env.Close()
	ctx := context.Background()

	// 1. Create Portal (Agent side)
	portalID, agentCleanup, err := env.Mux.CreatePortal(ctx, env.EntClient, env.Task.ID, 0)
	require.NoError(t, err)
	defer agentCleanup()

	p, err := env.EntClient.Portal.Get(ctx, portalID)
	require.NoError(t, err)

	err = env.Shell.Update().AddPortals(p).Exec(ctx)
	require.NoError(t, err)

	// Subscribe to verify connection
	// agentInCh, agentSubCleanup := env.Mux.Subscribe(env.Mux.TopicIn(p.ID))
	// defer agentSubCleanup()

	// 2. Create ShellTask history *before* WebSocket connects
	_, err = env.EntClient.ShellTask.Create().
		SetShell(env.Shell).
		SetInput("history_command").
		SetOutput("history_output").
		SetCreator(env.User).
		SetStreamID("history-stream").
		SetSequenceID(1).
		Save(ctx)
	require.NoError(t, err)

	// 3. Connect via WebSocket (User side)
	url := fmt.Sprintf("%s?shell_id=%d", env.WSURL, env.Shell.ID)
	ws, _, err := websocket.DefaultDialer.Dial(url, nil)
	require.NoError(t, err)
	defer ws.Close()

	// 4. Verify History is Received IMMEDIATELY
	// We expect the history to be sent upon connection, even if interactive mode is active.
	var outMsg shell.WebsocketTaskOutputMessage
	found := false
	timeout := time.After(2 * time.Second) // Should be fast

	for !found {
		select {
		case <-timeout:
			t.Fatal("timeout waiting for history output")
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
				if strings.Contains(outMsg.Output, "history_output") {
					found = true
				}
			} else if genericMsg.Kind == shell.WebsocketMessageKindOutputFromOtherStream {
				var otherMsg shell.WebsocketTaskOutputFromOtherStreamMessage
				json.Unmarshal(data, &otherMsg)
				if strings.Contains(otherMsg.Output, "history_output") {
					found = true
					// Assign to outMsg for final assertion if needed
					outMsg.Output = otherMsg.Output
				}
			}
		}
	}
	require.Contains(t, outMsg.Output, "history_output")
}

func TestShellActiveUsers(t *testing.T) {
	env := SetupTestEnv(t)
	defer env.Close()
	ctx := context.Background()

	// 1. Initial State: No Active Users
	users, err := env.Shell.QueryActiveUsers().All(ctx)
	require.NoError(t, err)
	require.Empty(t, users)

	// 2. Connect via WebSocket
	url := fmt.Sprintf("%s?shell_id=%d", env.WSURL, env.Shell.ID)
	ws, _, err := websocket.DefaultDialer.Dial(url, nil)
	require.NoError(t, err)
	defer ws.Close()

	// 3. Verify Active User Added
	// Give a small grace period for the goroutine to run
	require.Eventually(t, func() bool {
		users, err := env.Shell.QueryActiveUsers().All(ctx)
		require.NoError(t, err)
		if len(users) != 1 {
			return false
		}
		return users[0].ID == env.User.ID
	}, 5*time.Second, 100*time.Millisecond)

	// 4. Disconnect
	ws.Close()

	// 5. Verify Active User Removed
	require.Eventually(t, func() bool {
		users, err := env.Shell.QueryActiveUsers().All(ctx)
		require.NoError(t, err)
		return len(users) == 0
	}, 5*time.Second, 100*time.Millisecond)
}
