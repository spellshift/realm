package pty_test

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/portals/pty"
)

type ptySyncBuffer struct {
	mu  sync.Mutex
	buf bytes.Buffer
}

func (b *ptySyncBuffer) Write(p []byte) (int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.Write(p)
}

func (b *ptySyncBuffer) String() string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.String()
}

type ptyAuditRecorder struct {
	mu      sync.Mutex
	entries []logging.AuditEntry
}

func (r *ptyAuditRecorder) Enqueue(entry logging.AuditEntry) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.entries = append(r.entries, entry)
}

func (r *ptyAuditRecorder) byAction() map[string]logging.AuditEntry {
	r.mu.Lock()
	defer r.mu.Unlock()
	out := make(map[string]logging.AuditEntry, len(r.entries))
	for _, entry := range r.entries {
		out[entry.Action] = entry
	}
	return out
}

func marshalPTYDetails(t *testing.T, raw map[string]any) map[string]any {
	t.Helper()
	b, err := json.Marshal(raw)
	require.NoError(t, err)
	var out map[string]any
	require.NoError(t, json.Unmarshal(b, &out))
	return out
}

func TestPTYPivotLoggingAndAudit(t *testing.T) {
	ctx := context.Background()

	entClient := enttest.OpenTempDB(t)
	defer entClient.Close()

	rec := &ptyAuditRecorder{}
	logBuf := &ptySyncBuffer{}
	require.NoError(t, logging.Configure(logging.Options{
		Spec:      "portals=debug,audit=info",
		Format:    logging.FormatJSON,
		Output:    logBuf,
		Persister: rec,
	}))
	t.Cleanup(func() {
		logging.SetAuditPersister(nil)
		_ = logging.Configure(logging.Options{Spec: "dev"})
	})

	portalMux := mux.New()

	operator, err := entClient.User.Create().
		SetOauthID("pty-audit-oauth").
		SetName("Carol PTY Operator").
		SetPhotoURL("http://example.com/carol.jpg").
		SetSessionToken("carol-pty-session-token").
		SetIsActivated(true).
		Save(ctx)
	require.NoError(t, err)

	hostEnt, err := entClient.Host.Create().
		SetName("pty-audit-host").
		SetIdentifier("pty-audit-host-1").
		SetPrimaryIP("10.20.0.5").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		Save(ctx)
	require.NoError(t, err)

	beaconEnt, err := entClient.Beacon.Create().
		SetIdentifier("pty-audit-beacon").
		SetName("pty-audit-beacon").
		SetHost(hostEnt).
		SetTransport(c2pb.Transport_TRANSPORT_GRPC).
		Save(ctx)
	require.NoError(t, err)

	tomeEnt, err := entClient.Tome.Create().
		SetName("pty-audit-tome").
		SetDescription("pty audit tome").
		SetAuthor("carol").
		SetTactic("EXECUTION").
		SetEldritch("portal.create()").
		Save(ctx)
	require.NoError(t, err)

	questEnt, err := entClient.Quest.Create().
		SetName("pty-audit-quest").
		SetCreator(operator).
		SetTome(tomeEnt).
		Save(ctx)
	require.NoError(t, err)

	taskEnt, err := entClient.Task.Create().
		SetBeacon(beaconEnt).
		SetQuest(questEnt).
		Save(ctx)
	require.NoError(t, err)

	handler := pty.NewHandler(entClient, portalMux)
	authMiddleware := func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			authCtx, aErr := auth.ContextFromSessionToken(r.Context(), entClient, operator.SessionToken)
			if aErr == nil {
				r = r.WithContext(authCtx)
			}
			next.ServeHTTP(w, r)
		})
	}
	server := httptest.NewServer(authMiddleware(http.HandlerFunc(handler.ServeHTTP)))
	defer server.Close()
	wsBaseURL := "ws" + strings.TrimPrefix(server.URL, "http")

	portalID, agentCleanup, err := portalMux.CreatePortal(ctx, entClient, taskEnt.ID, 0)
	require.NoError(t, err)
	defer agentCleanup()

	inCh, inCleanup := portalMux.Subscribe(portalMux.TopicIn(portalID))
	defer inCleanup()

	// 1. Open PTY pivot
	ws, _, err := websocket.DefaultDialer.Dial(fmt.Sprintf("%s?portal_id=%d&rows=24&cols=80", wsBaseURL, portalID), nil)
	require.NoError(t, err)
	defer ws.Close()

	select {
	case <-inCh:
	case <-time.After(3 * time.Second):
		t.Fatal("timed out waiting for PTY init mote")
	}

	pivots, err := entClient.ShellPivot.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, pivots, 1)
	pivotID := pivots[0].ID

	// 2. Reconnect to active PTY pivot
	wsReconn, _, err := websocket.DefaultDialer.Dial(fmt.Sprintf("%s?pivot_id=%d", wsBaseURL, pivotID), nil)
	require.NoError(t, err)
	wsReconn.SetReadDeadline(time.Now().Add(2 * time.Second))
	_, _, _ = wsReconn.ReadMessage()
	_ = wsReconn.Close()

	// 3. Send close control message over primary websocket
	require.NoError(t, ws.WriteJSON(pty.WebsocketControlMessage{Kind: "close"}))
	require.Eventually(t, func() bool {
		p, qErr := entClient.ShellPivot.Get(ctx, pivotID)
		return qErr == nil && !p.ClosedAt.IsZero()
	}, 3*time.Second, 25*time.Millisecond)

	rawLogs := logBuf.String()
	assert.Contains(t, rawLogs, `"subsystem":"portals.pty"`)
	assert.Contains(t, rawLogs, fmt.Sprintf(`"pivot_id":%d`, pivotID))
	assert.Contains(t, rawLogs, fmt.Sprintf(`"portal_id":%d`, portalID))

	byAction := rec.byAction()
	for _, action := range []string{"portal.pty_pivot_opened", "portal.pty_pivot_reconnected", "portal.pty_pivot_closed"} {
		entry, ok := byAction[action]
		require.True(t, ok, "expected %s audit log", action)
		assert.Equal(t, logging.OutcomeSuccess, entry.Outcome)
		assert.Equal(t, logging.ActorUser, entry.Actor.Kind)
		assert.Equal(t, strconv.Itoa(operator.ID), entry.Actor.ID)
		assert.Equal(t, "Carol PTY Operator", entry.Actor.Name)
		assert.Equal(t, "pivot", entry.TargetType)
		assert.Equal(t, strconv.Itoa(pivotID), entry.TargetID)

		details := marshalPTYDetails(t, entry.Details)
		assert.Equal(t, float64(pivotID), details["pivot_id"])
		assert.Equal(t, float64(portalID), details["portal_id"])
		assert.Equal(t, float64(beaconEnt.ID), details["beacon_id"])
		assert.Equal(t, float64(hostEnt.ID), details["host_id"])
	}
}
