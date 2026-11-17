package ssh_test

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net"
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
	"realm.pub/tavern/internal/c2/epb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/http/shell"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/portals/mux"
	portalssh "realm.pub/tavern/internal/portals/ssh"
	"realm.pub/tavern/portals/portalpb"
)

type sshSyncBuffer struct {
	mu  sync.Mutex
	buf bytes.Buffer
}

func (b *sshSyncBuffer) Write(p []byte) (int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.Write(p)
}

func (b *sshSyncBuffer) String() string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.String()
}

type sshAuditRecorder struct {
	mu      sync.Mutex
	entries []logging.AuditEntry
}

func (r *sshAuditRecorder) Enqueue(entry logging.AuditEntry) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.entries = append(r.entries, entry)
}

func (r *sshAuditRecorder) snapshot() []logging.AuditEntry {
	r.mu.Lock()
	defer r.mu.Unlock()
	out := make([]logging.AuditEntry, len(r.entries))
	copy(out, r.entries)
	return out
}

func (r *sshAuditRecorder) byAction(action string) []logging.AuditEntry {
	var matched []logging.AuditEntry
	for _, entry := range r.snapshot() {
		if entry.Action == action {
			matched = append(matched, entry)
		}
	}
	return matched
}

func marshalSSHDetails(t *testing.T, raw map[string]any) (string, map[string]any) {
	t.Helper()
	b, err := json.Marshal(raw)
	require.NoError(t, err)
	var out map[string]any
	require.NoError(t, json.Unmarshal(b, &out))
	return string(b), out
}

func TestSSHPivotLoggingAndAudit(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	entClient := enttest.OpenTempDB(t)
	defer entClient.Close()

	rec := &sshAuditRecorder{}
	logBuf := &sshSyncBuffer{}
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
		SetOauthID("ssh-audit-oauth").
		SetName("Bob SSH Operator").
		SetPhotoURL("http://example.com/bob.jpg").
		SetSessionToken("bob-ssh-session-token").
		SetIsActivated(true).
		Save(ctx)
	require.NoError(t, err)

	hostEnt, err := entClient.Host.Create().
		SetName("ssh-audit-host").
		SetIdentifier("ssh-audit-host-1").
		SetPrimaryIP("10.10.0.12").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		Save(ctx)
	require.NoError(t, err)

	beaconEnt, err := entClient.Beacon.Create().
		SetIdentifier("ssh-audit-beacon").
		SetName("ssh-audit-beacon").
		SetHost(hostEnt).
		SetTransport(c2pb.Transport_TRANSPORT_GRPC).
		Save(ctx)
	require.NoError(t, err)

	tomeEnt, err := entClient.Tome.Create().
		SetName("ssh-audit-tome").
		SetDescription("ssh audit tome").
		SetAuthor("bob").
		SetTactic("LATERAL_MOVEMENT").
		SetEldritch("portal.create()").
		Save(ctx)
	require.NoError(t, err)

	questEnt, err := entClient.Quest.Create().
		SetName("ssh-audit-quest").
		SetCreator(operator).
		SetTome(tomeEnt).
		Save(ctx)
	require.NoError(t, err)

	taskEnt, err := entClient.Task.Create().
		SetBeacon(beaconEnt).
		SetQuest(questEnt).
		Save(ctx)
	require.NoError(t, err)

	shellEnt, err := entClient.Shell.Create().
		SetBeacon(beaconEnt).
		SetOwner(operator).
		SetData([]byte{}).
		Save(ctx)
	require.NoError(t, err)

	inlineSecret := "TopSecretInlinePass99!"
	credSecret := "SuperSecretCredPass42!"
	wrongSecret := "WrongSecretPass77!"

	credEnt, err := entClient.HostCredential.Create().
		SetHost(hostEnt).
		SetTask(taskEnt).
		SetPrincipal("operator").
		SetSecret(credSecret).
		SetKind(epb.Credential_KIND_PASSWORD).
		Save(ctx)
	require.NoError(t, err)

	sshAddr, cleanupSSH := startTestSSHServer(t, "operator", inlineSecret)
	defer cleanupSSH()

	handler := portalssh.NewHandler(entClient, portalMux)
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

	inCh, cleanupIn := portalMux.Subscribe(portalMux.TopicIn(portalID))
	defer cleanupIn()

	var (
		tcpConnsMu sync.Mutex
		tcpConns   = make(map[string]net.Conn)
		sshMux     *portalssh.Mux
	)
	sendToAgent := func(mote *portalpb.Mote) error {
		tcpPayload := mote.GetTcp()
		if tcpPayload == nil {
			return nil
		}
		streamID := mote.StreamId
		tcpConnsMu.Lock()
		conn, exists := tcpConns[streamID]
		if !exists {
			targetAddr := net.JoinHostPort(tcpPayload.DstAddr, strconv.Itoa(int(tcpPayload.DstPort)))
			var dialErr error
			conn, dialErr = net.DialTimeout("tcp", targetAddr, 5*time.Second)
			if dialErr != nil {
				tcpConnsMu.Unlock()
				_ = sshMux.RouteAgentMote(&portalpb.Mote{
					StreamId: streamID,
					Payload: &portalpb.Mote_Bytes{
						Bytes: &portalpb.BytesPayload{Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE},
					},
				})
				return nil
			}
			tcpConns[streamID] = conn
			go func(c net.Conn, sid string) {
				defer func() {
					_ = c.Close()
					tcpConnsMu.Lock()
					delete(tcpConns, sid)
					tcpConnsMu.Unlock()
					_ = sshMux.RouteAgentMote(&portalpb.Mote{
						StreamId: sid,
						Payload: &portalpb.Mote_Bytes{
							Bytes: &portalpb.BytesPayload{Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE},
						},
					})
				}()
				buf := make([]byte, 4096)
				var seq uint64
				for {
					n, rErr := c.Read(buf)
					if n > 0 {
						chunk := make([]byte, n)
						copy(chunk, buf[:n])
						_ = sshMux.RouteAgentMote(&portalpb.Mote{
							StreamId: sid,
							SeqId:    seq,
							Payload:  &portalpb.Mote_Tcp{Tcp: &portalpb.TCPPayload{Data: chunk}},
						})
						seq++
					}
					if rErr != nil {
						return
					}
				}
			}(conn, streamID)
		}
		tcpConnsMu.Unlock()
		if len(tcpPayload.Data) > 0 {
			_, wErr := conn.Write(tcpPayload.Data)
			return wErr
		}
		return nil
	}

	sshMux = portalssh.New(portalID, portalMux, sendToAgent)
	defer sshMux.Close()

	go func() {
		for {
			select {
			case <-ctx.Done():
				return
			case mote, ok := <-inCh:
				if !ok || mote == nil {
					return
				}
				if mote.GetSsh() != nil {
					sshMux.HandleMote(ctx, mote)
				}
			}
		}
	}()

	// 1. Open SSH pivot with inline secret + credential_id
	wsURL := fmt.Sprintf("%s?portal_id=%d&shell_id=%d&target=%s&credential_id=%d",
		wsBaseURL, portalID, shellEnt.ID, fmt.Sprintf("operator:%s@%s", inlineSecret, sshAddr), credEnt.ID)
	ws, _, err := websocket.DefaultDialer.Dial(wsURL, nil)
	require.NoError(t, err)
	defer ws.Close()

	deadline := time.Now().Add(5 * time.Second)
	ws.SetReadDeadline(deadline)
	var gotWelcome bool
	for time.Now().Before(deadline) {
		_, data, rErr := ws.ReadMessage()
		require.NoError(t, rErr)
		var outMsg shell.WebsocketTaskOutputMessage
		if json.Unmarshal(data, &outMsg) == nil && outMsg.Kind == shell.WebsocketMessageKindOutput && strings.Contains(outMsg.Output, "Welcome to Test SSH") {
			gotWelcome = true
			break
		}
	}
	require.True(t, gotWelcome)

	pivots, err := entClient.ShellPivot.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, pivots, 1)
	pivotID := pivots[0].ID

	// 2. Reconnect to active SSH pivot via pivot_id
	wsReconn, _, err := websocket.DefaultDialer.Dial(fmt.Sprintf("%s?pivot_id=%d", wsBaseURL, pivotID), nil)
	require.NoError(t, err)
	wsReconn.SetReadDeadline(time.Now().Add(3 * time.Second))
	_, _, _ = wsReconn.ReadMessage()
	_ = wsReconn.Close()

	// 3. Close SSH pivot via remote exit
	require.NoError(t, ws.WriteJSON(shell.WebsocketTaskInputMessage{
		Kind:  shell.WebsocketMessageKindInput,
		Input: "exit\n",
	}))
	require.Eventually(t, func() bool {
		p, qErr := entClient.ShellPivot.Get(ctx, pivotID)
		return qErr == nil && !p.ClosedAt.IsZero()
	}, 5*time.Second, 25*time.Millisecond)

	// 4. Attempt second SSH pivot with wrong password -> triggers portal.ssh_pivot_failed
	badWSURL := fmt.Sprintf("%s?portal_id=%d&shell_id=%d&target=%s",
		wsBaseURL, portalID, shellEnt.ID, fmt.Sprintf("operator:%s@%s", wrongSecret, sshAddr))
	badWS, _, err := websocket.DefaultDialer.Dial(badWSURL, nil)
	require.NoError(t, err)
	defer badWS.Close()

	badDeadline := time.Now().Add(5 * time.Second)
	badWS.SetReadDeadline(badDeadline)
	for time.Now().Before(badDeadline) {
		_, data, rErr := badWS.ReadMessage()
		if rErr != nil {
			break
		}
		var errMsg shell.WebsocketErrorMessage
		if json.Unmarshal(data, &errMsg) == nil && errMsg.Kind == shell.WebsocketMessageKindError {
			break
		}
	}

	require.Eventually(t, func() bool {
		return len(rec.byAction("portal.ssh_pivot_failed")) == 1
	}, 5*time.Second, 25*time.Millisecond)

	audits := rec.snapshot()
	byAction := make(map[string][]logging.AuditEntry)
	for _, a := range audits {
		byAction[a.Action] = append(byAction[a.Action], a)
		detailsJSON, _ := marshalSSHDetails(t, a.Details)
		for _, secret := range []string{inlineSecret, credSecret, wrongSecret, "BEGIN OPENSSH PRIVATE KEY"} {
			assert.NotContains(t, detailsJSON, secret, "audit details leaked SSH secret for action %s", a.Action)
			assert.NotContains(t, a.Summary, secret, "audit summary leaked SSH secret for action %s", a.Action)
			assert.NotContains(t, a.TargetName, secret, "audit target_name leaked SSH secret for action %s", a.Action)
		}
	}

	// Verify structured logs also never contain any SSH secrets and carry portals.ssh + pivot_id
	rawLogs := logBuf.String()
	for _, secret := range []string{inlineSecret, credSecret, wrongSecret, "BEGIN OPENSSH PRIVATE KEY"} {
		assert.NotContains(t, rawLogs, secret, "structured console logs leaked SSH secret")
	}
	assert.Contains(t, rawLogs, `"subsystem":"portals.ssh"`)
	assert.Contains(t, rawLogs, fmt.Sprintf(`"pivot_id":%d`, pivotID))

	// Verify portal.ssh_pivot_opened
	require.Len(t, byAction["portal.ssh_pivot_opened"], 1)
	opened := byAction["portal.ssh_pivot_opened"][0]
	assert.Equal(t, logging.OutcomeSuccess, opened.Outcome)
	assert.Equal(t, logging.ActorUser, opened.Actor.Kind)
	assert.Equal(t, strconv.Itoa(operator.ID), opened.Actor.ID)
	assert.Equal(t, "Bob SSH Operator", opened.Actor.Name)
	assert.Equal(t, "pivot", opened.TargetType)
	assert.Equal(t, strconv.Itoa(pivotID), opened.TargetID)
	assert.Equal(t, sshAddr, opened.TargetName)
	_, openedDetails := marshalSSHDetails(t, opened.Details)
	assert.Equal(t, float64(pivotID), openedDetails["pivot_id"])
	assert.Equal(t, float64(portalID), openedDetails["portal_id"])
	assert.Equal(t, float64(hostEnt.ID), openedDetails["host_id"])
	assert.Equal(t, sshAddr, openedDetails["target"])
	assert.Equal(t, "operator", openedDetails["ssh_user"])
	assert.Equal(t, float64(credEnt.ID), openedDetails["credential_id"])

	// Verify portal.ssh_pivot_reconnected
	require.Len(t, byAction["portal.ssh_pivot_reconnected"], 1)
	assert.Equal(t, logging.ActorUser, byAction["portal.ssh_pivot_reconnected"][0].Actor.Kind)

	// Verify portal.ssh_pivot_closed (only for the 1st connected pivot, NOT the 2nd failed pivot!)
	require.Len(t, byAction["portal.ssh_pivot_closed"], 1)
	assert.Equal(t, strconv.Itoa(pivotID), byAction["portal.ssh_pivot_closed"][0].TargetID)

	// Verify portal.ssh_pivot_failed
	require.Len(t, byAction["portal.ssh_pivot_failed"], 1)
	failed := byAction["portal.ssh_pivot_failed"][0]
	assert.Equal(t, logging.OutcomeFailure, failed.Outcome)
	assert.Equal(t, logging.ActorUser, failed.Actor.Kind)
	assert.Equal(t, sshAddr, failed.TargetName)
}
