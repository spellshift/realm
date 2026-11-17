package shell_test

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"strconv"
	"sync"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"realm.pub/tavern/internal/http/shell"
	"realm.pub/tavern/internal/logging"
)

type shellSyncBuffer struct {
	mu  sync.Mutex
	buf bytes.Buffer
}

func (b *shellSyncBuffer) Write(p []byte) (int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.Write(p)
}

func (b *shellSyncBuffer) String() string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.String()
}

type shellAuditRecorder struct {
	mu      sync.Mutex
	entries []logging.AuditEntry
}

func (r *shellAuditRecorder) Enqueue(entry logging.AuditEntry) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.entries = append(r.entries, entry)
}

func (r *shellAuditRecorder) snapshot() []logging.AuditEntry {
	r.mu.Lock()
	defer r.mu.Unlock()
	out := make([]logging.AuditEntry, len(r.entries))
	copy(out, r.entries)
	return out
}

func (r *shellAuditRecorder) byAction(action string) []logging.AuditEntry {
	var matched []logging.AuditEntry
	for _, entry := range r.snapshot() {
		if entry.Action == action {
			matched = append(matched, entry)
		}
	}
	return matched
}

func marshalShellDetails(t *testing.T, raw map[string]any) map[string]any {
	t.Helper()
	b, err := json.Marshal(raw)
	require.NoError(t, err)
	var out map[string]any
	require.NoError(t, json.Unmarshal(b, &out))
	return out
}

func TestShellLoggingAndAudit(t *testing.T) {
	env := SetupTestEnv(t)
	defer env.Close()
	ctx := context.Background()

	rec := &shellAuditRecorder{}
	logBuf := &shellSyncBuffer{}
	require.NoError(t, logging.Configure(logging.Options{
		Spec:      "shells=debug,audit=info",
		Format:    logging.FormatJSON,
		Output:    logBuf,
		Persister: rec,
	}))
	t.Cleanup(func() {
		logging.SetAuditPersister(nil)
		_ = logging.Configure(logging.Options{Spec: "dev"})
	})

	hostEnt, err := env.Beacon.QueryHost().Only(ctx)
	require.NoError(t, err)

	// 1. Connect WebSocket without an open portal (queued mode)
	url := fmt.Sprintf("%s?shell_id=%d", env.WSURL, env.Shell.ID)
	ws, _, err := websocket.DefaultDialer.Dial(url, nil)
	require.NoError(t, err)

	// Submit queued command
	require.NoError(t, ws.WriteJSON(shell.WebsocketTaskInputMessage{
		Kind:  shell.WebsocketMessageKindInput,
		Input: "whoami",
	}))

	// Wait for Task Queued signal
	ws.SetReadDeadline(time.Now().Add(5 * time.Second))
	for {
		_, data, rErr := ws.ReadMessage()
		require.NoError(t, rErr)
		var generic struct {
			Kind string `json:"kind"`
		}
		_ = json.Unmarshal(data, &generic)
		if generic.Kind == shell.WebsocketMessageKindControlFlow {
			var ctrl shell.WebsocketControlFlowMessage
			_ = json.Unmarshal(data, &ctrl)
			if ctrl.Signal == shell.WebsocketControlFlowSignalTaskQueued {
				break
			}
		}
	}

	// 2. Create portal so session upgrades to interactive mode
	portalID, agentCleanup, err := env.Mux.CreatePortal(ctx, env.EntClient, env.Task.ID, 0)
	require.NoError(t, err)
	defer agentCleanup()

	agentInCh, agentSubCleanup := env.Mux.Subscribe(env.Mux.TopicIn(portalID))
	defer agentSubCleanup()

	ws.SetReadDeadline(time.Now().Add(10 * time.Second))
	for {
		_, data, rErr := ws.ReadMessage()
		require.NoError(t, rErr)
		var generic struct {
			Kind string `json:"kind"`
		}
		_ = json.Unmarshal(data, &generic)
		if generic.Kind == shell.WebsocketMessageKindControlFlow {
			var ctrl shell.WebsocketControlFlowMessage
			_ = json.Unmarshal(data, &ctrl)
			if ctrl.Signal == shell.WebsocketControlFlowSignalPortalUpgrade {
				require.Equal(t, portalID, ctrl.PortalID)
				break
			}
		}
	}

	// 3. Submit interactive command
	require.NoError(t, ws.WriteJSON(shell.WebsocketTaskInputMessage{
		Kind:  shell.WebsocketMessageKindInput,
		Input: "uname -a",
	}))

	select {
	case mote := <-agentInCh:
		require.NotNil(t, mote.GetShell())
		assert.Equal(t, "uname -a", mote.GetShell().Input)
	case <-time.After(5 * time.Second):
		t.Fatal("timed out waiting for interactive shell mote on portal")
	}

	// 4. Close WebSocket and wait for shell.ws_disconnected audit entry
	_ = ws.Close()

	require.Eventually(t, func() bool {
		return len(rec.byAction("shell.ws_disconnected")) == 1
	}, 5*time.Second, 25*time.Millisecond)

	// Verify per-message "publishing to portal" chatter was demoted to TRACE (not emitted at DEBUG/INFO)
	rawLogs := logBuf.String()
	assert.Contains(t, rawLogs, `"subsystem":"shells.ws"`)
	assert.Contains(t, rawLogs, fmt.Sprintf(`"shell_id":%d`, env.Shell.ID))
	assert.Contains(t, rawLogs, `"subsystem":"audit.shell"`)
	assert.NotContains(t, rawLogs, "publishing to portal", "high-frequency 'publishing to portal' must be TRACE only")

	// Verify shell.ws_connected & shell.ws_disconnected
	connectedEntries := rec.byAction("shell.ws_connected")
	disconnectedEntries := rec.byAction("shell.ws_disconnected")
	require.Len(t, connectedEntries, 1)
	require.Len(t, disconnectedEntries, 1)
	for _, entry := range []logging.AuditEntry{connectedEntries[0], disconnectedEntries[0]} {
		assert.Equal(t, logging.OutcomeSuccess, entry.Outcome)
		assert.Equal(t, logging.ActorUser, entry.Actor.Kind)
		assert.Equal(t, strconv.Itoa(env.User.ID), entry.Actor.ID)
		assert.Equal(t, env.User.Name, entry.Actor.Name)
		assert.Equal(t, "shell", entry.TargetType)
		assert.Equal(t, strconv.Itoa(env.Shell.ID), entry.TargetID)
		details := marshalShellDetails(t, entry.Details)
		assert.Equal(t, float64(env.Shell.ID), details["shell_id"])
		assert.Equal(t, float64(env.Beacon.ID), details["beacon_id"])
		assert.Equal(t, float64(hostEnt.ID), details["host_id"])
	}

	// Verify shell.task_created for both queued and interactive commands
	taskAudits := rec.byAction("shell.task_created")
	require.Len(t, taskAudits, 2)

	queuedDetails := marshalShellDetails(t, taskAudits[0].Details)
	interactiveDetails := marshalShellDetails(t, taskAudits[1].Details)

	assert.Equal(t, logging.ActorUser, taskAudits[0].Actor.Kind)
	assert.Equal(t, strconv.Itoa(env.User.ID), taskAudits[0].Actor.ID)
	assert.Equal(t, "shell_task", taskAudits[0].TargetType)
	assert.NotEmpty(t, taskAudits[0].TargetID)
	assert.Equal(t, "queued", queuedDetails["mode"])
	assert.Equal(t, "whoami", queuedDetails["command"])
	assert.Equal(t, float64(env.Shell.ID), queuedDetails["shell_id"])
	assert.Equal(t, float64(env.Beacon.ID), queuedDetails["beacon_id"])
	assert.Equal(t, float64(hostEnt.ID), queuedDetails["host_id"])

	assert.Equal(t, logging.ActorUser, taskAudits[1].Actor.Kind)
	assert.Equal(t, "shell_task", taskAudits[1].TargetType)
	assert.Equal(t, "interactive", interactiveDetails["mode"])
	assert.Equal(t, "uname -a", interactiveDetails["command"])
	assert.Equal(t, float64(portalID), interactiveDetails["portal_id"])
	assert.Equal(t, float64(env.Shell.ID), interactiveDetails["shell_id"])
	assert.Equal(t, float64(env.Beacon.ID), interactiveDetails["beacon_id"])
	assert.Equal(t, float64(hostEnt.ID), interactiveDetails["host_id"])
}
