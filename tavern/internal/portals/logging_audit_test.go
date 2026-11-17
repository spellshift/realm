package portals_test

import (
	"bytes"
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"encoding/json"
	"net"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gocloud.dev/pubsub"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/c2"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/portals"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
	portalstream "realm.pub/tavern/portals/stream"
)

type syncBuffer struct {
	mu  sync.Mutex
	buf bytes.Buffer
}

func (b *syncBuffer) Write(p []byte) (int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.Write(p)
}

func (b *syncBuffer) String() string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.String()
}

func (b *syncBuffer) Reset() {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.buf.Reset()
}

type portalAuditRecorder struct {
	mu      sync.Mutex
	entries []logging.AuditEntry
}

func (r *portalAuditRecorder) Enqueue(entry logging.AuditEntry) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.entries = append(r.entries, entry)
}

func (r *portalAuditRecorder) snapshot() []logging.AuditEntry {
	r.mu.Lock()
	defer r.mu.Unlock()
	out := make([]logging.AuditEntry, len(r.entries))
	copy(out, r.entries)
	return out
}

func (r *portalAuditRecorder) byAction(action string) []logging.AuditEntry {
	var matched []logging.AuditEntry
	for _, entry := range r.snapshot() {
		if entry.Action == action {
			matched = append(matched, entry)
		}
	}
	return matched
}

func marshalPortalDetails(t *testing.T, raw map[string]any) map[string]any {
	t.Helper()
	b, err := json.Marshal(raw)
	require.NoError(t, err)
	var out map[string]any
	require.NoError(t, json.Unmarshal(b, &out))
	return out
}

func parseJSONLogRecords(t *testing.T, raw string) []map[string]any {
	t.Helper()
	var records []map[string]any
	for _, line := range strings.Split(strings.TrimSpace(raw), "\n") {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		var m map[string]any
		require.NoError(t, json.Unmarshal([]byte(line), &m), "invalid JSON log line: %s", line)
		records = append(records, m)
	}
	return records
}

type denyPortalAccessCtxKey struct{}

type wrappedServerStream struct {
	grpc.ServerStream
	ctx context.Context
}

func (w *wrappedServerStream) Context() context.Context {
	return w.ctx
}

type authPortalTestEnv struct {
	C2Client     c2pb.C2Client
	PortalClient portalpb.PortalClient
	EntClient    *ent.Client
	PrivKey      ed25519.PrivateKey
	Operator     *ent.User
	TaskID       int
	BeaconID     int
	HostID       int
	Close        func()
}

func setupAuthPortalTestEnv(t *testing.T) *authPortalTestEnv {
	t.Helper()
	ctx := context.Background()

	entClient := enttest.OpenTempDB(t)
	entClient.Portal.Intercept(ent.InterceptFunc(func(next ent.Querier) ent.Querier {
		return ent.QuerierFunc(func(qCtx context.Context, q ent.Query) (ent.Value, error) {
			if deny, _ := qCtx.Value(denyPortalAccessCtxKey{}).(bool); deny {
				return nil, status.Error(codes.PermissionDenied, "privacy: permission denied")
			}
			return next.Query(qCtx, q)
		})
	}))

	portalMux := mux.New()

	topic, err := pubsub.OpenTopic(ctx, "mem://c2topic-audit")
	require.NoError(t, err)
	sub, err := pubsub.OpenSubscription(ctx, "mem://c2topic-audit")
	require.NoError(t, err)

	c2StreamMux := stream.NewMux(topic, sub)
	ctxMux, cancelMux := context.WithCancel(ctx)
	go c2StreamMux.Start(ctxMux)

	testPubKey, testPrivKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	operator, err := entClient.User.Create().
		SetOauthID("portal-operator-oauth").
		SetName("Alice Operator").
		SetPhotoURL("http://example.com/alice.jpg").
		SetSessionToken("alice-portal-session").
		SetIsActivated(true).
		Save(ctx)
	require.NoError(t, err)

	hostEnt, err := entClient.Host.Create().
		SetName("portal-audit-host").
		SetIdentifier("portal-host-1").
		SetPrimaryIP("10.0.0.5").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		Save(ctx)
	require.NoError(t, err)

	beaconEnt, err := entClient.Beacon.Create().
		SetIdentifier("portal-audit-beacon").
		SetName("beacon-alpha").
		SetHost(hostEnt).
		SetTransport(c2pb.Transport_TRANSPORT_GRPC).
		Save(ctx)
	require.NoError(t, err)

	tomeEnt, err := entClient.Tome.Create().
		SetName("portal-tome").
		SetDescription("portal tome").
		SetAuthor("alice").
		SetTactic("COMMAND_AND_CONTROL").
		SetEldritch("portal.create()").
		Save(ctx)
	require.NoError(t, err)

	questEnt, err := entClient.Quest.Create().
		SetName("portal-quest").
		SetCreator(operator).
		SetTome(tomeEnt).
		Save(ctx)
	require.NoError(t, err)

	taskEnt, err := entClient.Task.Create().
		SetBeacon(beaconEnt).
		SetQuest(questEnt).
		Save(ctx)
	require.NoError(t, err)

	c2Server := c2.New(entClient, c2StreamMux, portalMux, testPubKey, testPrivKey)
	portalServer := portals.New(entClient, portalMux)

	streamAuthInterceptor := func(srv any, ss grpc.ServerStream, info *grpc.StreamServerInfo, handler grpc.StreamHandler) error {
		sCtx := ss.Context()
		if md, ok := metadata.FromIncomingContext(sCtx); ok {
			if vals := md.Get("x-session-token"); len(vals) > 0 && vals[0] != "" {
				if authCtx, aErr := auth.ContextFromSessionToken(sCtx, entClient, vals[0]); aErr == nil {
					sCtx = authCtx
				}
			}
			if vals := md.Get("x-deny-portal"); len(vals) > 0 && vals[0] == "true" {
				sCtx = context.WithValue(sCtx, denyPortalAccessCtxKey{}, true)
			}
		}
		return handler(srv, &wrappedServerStream{ServerStream: ss, ctx: sCtx})
	}

	lis := bufconn.Listen(bufSize)
	s := grpc.NewServer(grpc.StreamInterceptor(streamAuthInterceptor))
	c2pb.RegisterC2Server(s, c2Server)
	portalpb.RegisterPortalServer(s, portalServer)
	go func() {
		_ = s.Serve(lis)
	}()

	dialer := func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}
	conn, err := grpc.DialContext(ctx, "bufnet", grpc.WithContextDialer(dialer), grpc.WithTransportCredentials(insecure.NewCredentials()))
	require.NoError(t, err)

	return &authPortalTestEnv{
		C2Client:     c2pb.NewC2Client(conn),
		PortalClient: portalpb.NewPortalClient(conn),
		EntClient:    entClient,
		PrivKey:      testPrivKey,
		Operator:     operator,
		TaskID:       taskEnt.ID,
		BeaconID:     beaconEnt.ID,
		HostID:       hostEnt.ID,
		Close: func() {
			_ = conn.Close()
			s.Stop()
			cancelMux()
			_ = topic.Shutdown(ctx)
			_ = sub.Shutdown(ctx)
			_ = entClient.Close()
		},
	}
}

func TestPortalsSubsystemFilteringAndTrace(t *testing.T) {
	logBuf := &syncBuffer{}
	require.NoError(t, logging.Configure(logging.Options{
		Spec:   "portals=debug",
		Format: logging.FormatJSON,
		Output: logBuf,
	}))
	t.Cleanup(func() {
		logging.SetAuditPersister(nil)
		_ = logging.Configure(logging.Options{Spec: "dev"})
	})

	env := setupAuthPortalTestEnv(t)
	defer env.Close()

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Emit unrelated subsystem logs that must be filtered out under --log=portals=debug
	logging.For("graphql.query").Info("graphql request noise")
	logging.For("http.access").Info("http access noise")

	c2Stream, err := env.C2Client.CreatePortal(ctx)
	require.NoError(t, err)
	require.NoError(t, c2Stream.Send(&c2pb.CreatePortalRequest{
		Context: &c2pb.CreatePortalRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{
				TaskId: int64(env.TaskID),
				Jwt:    generateJWT(t, env.PrivKey, env.BeaconID),
			},
		},
	}))

	var portalID int
	require.Eventually(t, func() bool {
		all, qErr := env.EntClient.Portal.Query().All(ctx)
		if qErr == nil && len(all) == 1 {
			portalID = all[0].ID
			return true
		}
		return false
	}, 3*time.Second, 25*time.Millisecond)

	opCtx := metadata.AppendToOutgoingContext(ctx, "x-session-token", env.Operator.SessionToken)
	portalStream, err := env.PortalClient.OpenPortal(opCtx)
	require.NoError(t, err)
	require.NoError(t, portalStream.Send(&portalpb.OpenPortalRequest{PortalId: int64(portalID)}))

	userWriter := portalstream.NewOrderedWriter("user", func(m *portalpb.Mote) error {
		return portalStream.Send(&portalpb.OpenPortalRequest{Mote: m})
	})
	agentReader := portalstream.NewOrderedReader(func() (*portalpb.Mote, error) {
		resp, rErr := c2Stream.Recv()
		if rErr != nil {
			return nil, rErr
		}
		return resp.Mote, nil
	})

	require.NoError(t, userWriter.WriteBytes([]byte("debug-ping"), portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA))
	gotMote, err := agentReader.Read()
	require.NoError(t, err)
	require.Equal(t, []byte("debug-ping"), gotMote.GetBytes().Data)

	debugRecords := parseJSONLogRecords(t, logBuf.String())
	require.NotEmpty(t, debugRecords)

	seenSubsystems := make(map[string]bool)
	for _, rec := range debugRecords {
		sub, _ := rec["subsystem"].(string)
		seenSubsystems[sub] = true
		assert.NotEqual(t, "graphql.query", sub, "graphql noise must not appear with --log=portals=debug")
		assert.NotEqual(t, "http.access", sub, "http.access noise must not appear with --log=portals=debug")

		msg, _ := rec["msg"].(string)
		if msg == "" {
			msg, _ = rec["message"].(string)
		}
		assert.NotContains(t, msg, "publishing operator mote to portal input topic", "wire-level TRACE logs must not fire at DEBUG")
		assert.NotContains(t, msg, "sending mote to agent over gRPC", "wire-level TRACE logs must not fire at DEBUG")

		if sub == "portals.c2" || sub == "portals.grpc" {
			assert.Equal(t, float64(portalID), rec["portal_id"], "expected portal_id bound on %s record: %v", sub, rec)
		}
	}

	assert.True(t, seenSubsystems["portals.c2"], "expected portals.c2 debug logs")
	assert.True(t, seenSubsystems["portals.grpc"], "expected portals.grpc debug logs")
	assert.True(t, seenSubsystems["portals.mux"], "expected portals.mux debug logs")

	// Elevate to portals=trace and verify wire-level mote logs appear with portal_id
	logBuf.Reset()
	require.NoError(t, logging.Configure(logging.Options{
		Spec:   "portals=trace",
		Format: logging.FormatJSON,
		Output: logBuf,
	}))

	require.NoError(t, userWriter.WriteBytes([]byte("trace-ping"), portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA))
	gotTraceMote, err := agentReader.Read()
	require.NoError(t, err)
	require.Equal(t, []byte("trace-ping"), gotTraceMote.GetBytes().Data)

	traceRecords := parseJSONLogRecords(t, logBuf.String())
	sawTraceWireLog := false
	for _, rec := range traceRecords {
		rawRec, _ := json.Marshal(rec)
		if strings.Contains(string(rawRec), "publishing operator mote to portal input topic") ||
			strings.Contains(string(rawRec), "sending mote to agent over gRPC") {
			sawTraceWireLog = true
			assert.Equal(t, float64(portalID), rec["portal_id"])
		}
	}
	assert.True(t, sawTraceWireLog, "expected wire-level TRACE logs when --log=portals=trace")

	_ = portalStream.CloseSend()
	_ = c2Stream.CloseSend()
}

func TestPortalLifecycleAuditLogs(t *testing.T) {
	env := setupAuthPortalTestEnv(t)
	defer env.Close()

	rec := &portalAuditRecorder{}
	logBuf := &syncBuffer{}
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

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// 1. Agent creates portal
	c2Stream, err := env.C2Client.CreatePortal(ctx)
	require.NoError(t, err)
	require.NoError(t, c2Stream.Send(&c2pb.CreatePortalRequest{
		Context: &c2pb.CreatePortalRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{
				TaskId: int64(env.TaskID),
				Jwt:    generateJWT(t, env.PrivKey, env.BeaconID),
			},
		},
	}))

	var portalID int
	require.Eventually(t, func() bool {
		all, qErr := env.EntClient.Portal.Query().All(ctx)
		if qErr == nil && len(all) == 1 {
			portalID = all[0].ID
			return true
		}
		return false
	}, 3*time.Second, 25*time.Millisecond)

	// 2. Unauthorized operator denied OpenPortal
	deniedCtx := metadata.AppendToOutgoingContext(ctx, "x-deny-portal", "true")
	deniedStream, err := env.PortalClient.OpenPortal(deniedCtx)
	require.NoError(t, err)
	require.NoError(t, deniedStream.Send(&portalpb.OpenPortalRequest{PortalId: int64(portalID)}))
	_, deniedErr := deniedStream.Recv()
	require.Error(t, deniedErr)
	assert.Equal(t, codes.PermissionDenied, status.Code(deniedErr))

	// 3. Authenticated operator opens portal, exchanges data, and disconnects
	opCtx, opCancel := context.WithCancel(metadata.AppendToOutgoingContext(ctx, "x-session-token", env.Operator.SessionToken))
	portalStream, err := env.PortalClient.OpenPortal(opCtx)
	require.NoError(t, err)
	require.NoError(t, portalStream.Send(&portalpb.OpenPortalRequest{PortalId: int64(portalID)}))
	require.NoError(t, portalStream.Send(&portalpb.OpenPortalRequest{
		Mote: &portalpb.Mote{
			Payload: &portalpb.Mote_Bytes{
				Bytes: &portalpb.BytesPayload{
					Data: []byte("hello"),
					Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA,
				},
			},
		},
	}))

	resp, err := c2Stream.Recv()
	require.NoError(t, err)
	require.Equal(t, []byte("hello"), resp.Mote.GetBytes().Data)

	// Operator disconnects
	_ = portalStream.CloseSend()
	opCancel()

	// Agent closes portal
	require.NoError(t, c2Stream.CloseSend())
	require.Eventually(t, func() bool {
		p, gErr := env.EntClient.Portal.Get(ctx, portalID)
		return gErr == nil && !p.ClosedAt.IsZero()
	}, 3*time.Second, 25*time.Millisecond)

	require.Eventually(t, func() bool {
		return len(rec.byAction("portal.disconnected")) >= 1 && len(rec.byAction("portal.closed")) >= 1
	}, 3*time.Second, 25*time.Millisecond)

	audits := rec.snapshot()
	byActionOutcome := make(map[string]logging.AuditEntry, len(audits))
	for _, a := range audits {
		key := a.Action + ":" + string(a.Outcome)
		byActionOutcome[key] = a
	}

	// Verify portal.created (beacon attribution)
	created, ok := byActionOutcome["portal.created:"+string(logging.OutcomeSuccess)]
	require.True(t, ok, "expected portal.created audit entry, got %v", audits)
	assert.Equal(t, "audit.portal", created.Subsystem)
	assert.Equal(t, logging.ActorBeacon, created.Actor.Kind)
	assert.Equal(t, strconv.Itoa(env.BeaconID), created.Actor.ID)
	assert.Equal(t, "beacon-alpha", created.Actor.Name)
	assert.Equal(t, "portal", created.TargetType)
	assert.Equal(t, strconv.Itoa(portalID), created.TargetID)
	createdDetails := marshalPortalDetails(t, created.Details)
	assert.Equal(t, float64(portalID), createdDetails["portal_id"])
	assert.Equal(t, float64(env.TaskID), createdDetails["task_id"])
	assert.Equal(t, float64(env.BeaconID), createdDetails["beacon_id"])
	assert.Equal(t, float64(env.HostID), createdDetails["host_id"])

	// Verify portal.opened denied
	denied, ok := byActionOutcome["portal.opened:"+string(logging.OutcomeDenied)]
	require.True(t, ok, "expected denied portal.opened audit entry")
	assert.Equal(t, "portal", denied.TargetType)
	assert.Equal(t, strconv.Itoa(portalID), denied.TargetID)

	// Verify portal.opened (operator user attribution!)
	opened, ok := byActionOutcome["portal.opened:"+string(logging.OutcomeSuccess)]
	require.True(t, ok, "expected successful portal.opened audit entry")
	assert.Equal(t, logging.ActorUser, opened.Actor.Kind)
	assert.Equal(t, strconv.Itoa(env.Operator.ID), opened.Actor.ID)
	assert.Equal(t, "Alice Operator", opened.Actor.Name)
	assert.Equal(t, "portal", opened.TargetType)
	assert.Equal(t, strconv.Itoa(portalID), opened.TargetID)
	openedDetails := marshalPortalDetails(t, opened.Details)
	assert.Equal(t, float64(portalID), openedDetails["portal_id"])
	assert.Equal(t, float64(env.BeaconID), openedDetails["beacon_id"])
	assert.Equal(t, float64(env.HostID), openedDetails["host_id"])

	// Verify portal.disconnected (operator user attribution)
	disconnectedEntries := rec.byAction("portal.disconnected")
	require.NotEmpty(t, disconnectedEntries)
	assert.Equal(t, logging.ActorUser, disconnectedEntries[0].Actor.Kind)
	assert.Equal(t, strconv.Itoa(env.Operator.ID), disconnectedEntries[0].Actor.ID)
	assert.Equal(t, "Alice Operator", disconnectedEntries[0].Actor.Name)

	// Verify portal.closed (beacon attribution)
	closed, ok := byActionOutcome["portal.closed:"+string(logging.OutcomeSuccess)]
	require.True(t, ok, "expected portal.closed audit entry")
	assert.Equal(t, logging.ActorBeacon, closed.Actor.Kind)
	assert.Equal(t, strconv.Itoa(env.BeaconID), closed.Actor.ID)
	assert.Equal(t, "portal", closed.TargetType)
	assert.Equal(t, strconv.Itoa(portalID), closed.TargetID)

	// Verify audit events also landed in JSON structured log output
	rawLogs := logBuf.String()
	assert.Contains(t, rawLogs, `"audit_action":"portal.created"`)
	assert.Contains(t, rawLogs, `"audit_action":"portal.opened"`)
	assert.Contains(t, rawLogs, `"audit_action":"portal.closed"`)
}
