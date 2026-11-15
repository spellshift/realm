package c2_test

import (
	"bytes"
	"context"
	"crypto/rand"
	"encoding/json"
	"errors"
	"io"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/timestamppb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/c2test"
	"realm.pub/tavern/internal/c2/epb"
	"realm.pub/tavern/internal/logging"
)

type syncLogBuffer struct {
	mu  sync.Mutex
	buf bytes.Buffer
}

func (b *syncLogBuffer) Write(p []byte) (int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.Write(p)
}

func (b *syncLogBuffer) String() string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.String()
}

type auditRecorder struct {
	mu      sync.Mutex
	entries []logging.AuditEntry
}

func (r *auditRecorder) Enqueue(entry logging.AuditEntry) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.entries = append(r.entries, entry)
}

func (r *auditRecorder) Snapshot() []logging.AuditEntry {
	r.mu.Lock()
	defer r.mu.Unlock()
	out := make([]logging.AuditEntry, len(r.entries))
	copy(out, r.entries)
	return out
}

func (r *auditRecorder) Reset() {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.entries = nil
}

func (r *auditRecorder) ByAction(action string) []logging.AuditEntry {
	var matched []logging.AuditEntry
	for _, e := range r.Snapshot() {
		if e.Action == action {
			matched = append(matched, e)
		}
	}
	return matched
}

func setupC2AuditCapture(t *testing.T) (*syncLogBuffer, *auditRecorder) {
	t.Helper()
	logBuf := &syncLogBuffer{}
	rec := &auditRecorder{}
	require.NoError(t, logging.Configure(logging.Options{
		Spec:      "trace",
		Format:    logging.FormatJSON,
		Output:    logBuf,
		Persister: rec,
	}))
	t.Cleanup(func() {
		logging.SetAuditPersister(nil)
		_ = logging.Configure(logging.Options{Spec: "dev"})
	})
	return logBuf, rec
}

func newClaimTasksReq(beaconID, hostID, hostName string) *c2pb.ClaimTasksRequest {
	return &c2pb.ClaimTasksRequest{
		Beacon: &c2pb.Beacon{
			Identifier: beaconID,
			Principal:  "root",
			Agent:      &c2pb.Agent{Identifier: "imix"},
			Host: &c2pb.Host{
				Identifier: hostID,
				Name:       hostName,
				Platform:   c2pb.Host_PLATFORM_LINUX,
				PrimaryIp:  "10.10.20.30",
			},
			AvailableTransports: &c2pb.AvailableTransports{
				Transports: []*c2pb.Transport{{
					Uri:      "grpc://127.0.0.1:8080",
					Interval: 60,
					Type:     c2pb.Transport_TRANSPORT_GRPC,
				}},
				ActiveIndex: 0,
			},
		},
	}
}

func TestAuditClaimTasksMilestones(t *testing.T) {
	logBuf, rec := setupC2AuditCapture(t)
	client, graph, closeFn, _ := c2test.New(t)
	defer closeFn()

	ctx := metadata.NewOutgoingContext(context.Background(), metadata.Pairs(
		"traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
		"x-request-id", "req-claim-1",
		"x-redirected-for", "198.51.100.77",
	))

	// 1. Brand-new host + brand-new beacon with NO tasks:
	// Emits c2.host_registered and c2.beacon_registered, NEVER c2.tasks_claimed.
	resp, err := client.ClaimTasks(ctx, newClaimTasksReq("beacon-alpha", "host-alpha", "web-01"))
	require.NoError(t, err)
	require.Empty(t, resp.Tasks)

	hostRegs := rec.ByAction("c2.host_registered")
	require.Len(t, hostRegs, 1)
	assert.Equal(t, "audit.c2", hostRegs[0].Subsystem)
	assert.Equal(t, logging.OutcomeSuccess, hostRegs[0].Outcome)
	assert.Equal(t, logging.ActorBeacon, hostRegs[0].Actor.Kind)
	assert.Equal(t, "c2", hostRegs[0].Actor.AuthMethod)
	assert.Equal(t, "0af7651916cd43dd8448eb211c80319c", hostRegs[0].TraceID)
	assert.Equal(t, "198.51.100.77", hostRegs[0].ClientIP)

	beaconRegs := rec.ByAction("c2.beacon_registered")
	require.Len(t, beaconRegs, 1)
	assert.Equal(t, "audit.c2", beaconRegs[0].Subsystem)
	assert.Equal(t, logging.OutcomeSuccess, beaconRegs[0].Outcome)
	assert.Equal(t, "root", beaconRegs[0].Details["principal"])

	assert.Empty(t, rec.ByAction("c2.tasks_claimed"), "empty ClaimTasks poll must not emit c2.tasks_claimed")

	// 2. Subsequent empty poll from the same beacon + host: ZERO audit events.
	rec.Reset()
	resp, err = client.ClaimTasks(ctx, newClaimTasksReq("beacon-alpha", "host-alpha", "web-01"))
	require.NoError(t, err)
	require.Empty(t, resp.Tasks)
	assert.Empty(t, rec.Snapshot(), "routine empty beacon poll must emit zero audit records")

	// 3. Poll when a task is assigned: emits c2.tasks_claimed and never logs the minted JWT.
	rec.Reset()
	assigned := c2test.NewRandomAssignedTask(context.Background(), graph, "beacon-alpha")
	resp, err = client.ClaimTasks(ctx, newClaimTasksReq("beacon-alpha", "host-alpha", "web-01"))
	require.NoError(t, err)
	require.Len(t, resp.Tasks, 1)
	require.NotEmpty(t, resp.Tasks[0].Jwt)

	claimed := rec.ByAction("c2.tasks_claimed")
	require.Len(t, claimed, 1)
	assert.Equal(t, logging.OutcomeSuccess, claimed[0].Outcome)
	assert.Equal(t, logging.ActorBeacon, claimed[0].Actor.Kind)
	assert.Contains(t, claimed[0].Details, "task_ids")
	_ = assigned
	assert.NotContains(t, logBuf.String(), resp.Tasks[0].Jwt, "minted task JWT must never be logged")

	// 4. ScheduledTask automation creates a Quest with ActorScheduler and scrubs secret parameters.
	rec.Reset()
	const schedSecret = "SUPER_SECRET_SCHED_TOKEN_9988"
	tome := graph.Tome.Create().
		SetName("Automated Recon Tome").
		SetEldritch(`print("recon")`).
		SetDescription("Automated recon").
		SetAuthor("test").
		SetParamDefs(`[{"name":"api_key","label":"API Key","type":"string"},{"name":"target","label":"Target","type":"string"}]`).
		SaveX(context.Background())
	schedTask := graph.ScheduledTask.Create().
		SetName("Auto Recon").
		SetDescription("Automated recon").
		SetTome(tome).
		SetRunOnNewBeaconCallback(true).
		SetParameters(`{"api_key":"` + schedSecret + `","target":"10.9.8.7"}`).
		SaveX(context.Background())

	// Second beacon on the existing host triggers beacon_registered + quest.created + c2.tasks_claimed (not host_registered).
	resp, err = client.ClaimTasks(ctx, newClaimTasksReq("beacon-beta", "host-alpha", "web-01"))
	require.NoError(t, err)
	require.Len(t, resp.Tasks, 1)

	assert.Empty(t, rec.ByAction("c2.host_registered"), "existing host must not re-emit c2.host_registered")
	assert.Len(t, rec.ByAction("c2.beacon_registered"), 1)
	assert.Len(t, rec.ByAction("c2.tasks_claimed"), 1)

	questAudits := rec.ByAction("quest.created")
	require.Len(t, questAudits, 1)
	assert.Equal(t, "audit.quest", questAudits[0].Subsystem)
	assert.Equal(t, logging.ActorScheduler, questAudits[0].Actor.Kind)
	assert.Equal(t, "Auto Recon", questAudits[0].Actor.Name)
	assert.Equal(t, "scheduled_task", questAudits[0].Actor.AuthMethod)
	assert.EqualValues(t, schedTask.ID, questAudits[0].Details["scheduled_task_id"])
	assert.Equal(t, tome.Name, questAudits[0].Details["tome_name"])

	auditJSON, err := json.Marshal(rec.Snapshot())
	require.NoError(t, err)
	assert.NotContains(t, string(auditJSON), schedSecret, "scheduled task secret parameter must be scrubbed")
	assert.Contains(t, string(auditJSON), "[REDACTED]")
	assert.NotContains(t, logBuf.String(), schedSecret)
}

func TestAuditReportCredentialRedactsSecret(t *testing.T) {
	logBuf, rec := setupC2AuditCapture(t)
	client, graph, closeFn, mintJWT := c2test.New(t)
	defer closeFn()

	ctx := context.Background()
	beacon := c2test.NewRandomBeacon(ctx, graph)
	task := c2test.NewRandomAssignedTask(ctx, graph, beacon.Identifier)
	token := mintJWT(beacon.ID)

	const credSecret = "UltraSecretCleartextPassword!2026#$"
	_, err := client.ReportCredential(ctx, &c2pb.ReportCredentialRequest{
		Context: &c2pb.ReportCredentialRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{
				TaskId: int64(task.ID),
				Jwt:    token,
			},
		},
		Credential: &epb.Credential{
			Principal: `CORP\domain_admin`,
			Secret:    credSecret,
			Kind:      epb.Credential_KIND_PASSWORD,
		},
	})
	require.NoError(t, err)

	credAudits := rec.ByAction("c2.credential_reported")
	require.Len(t, credAudits, 1)
	assert.Equal(t, "audit.c2", credAudits[0].Subsystem)
	assert.Equal(t, logging.OutcomeSuccess, credAudits[0].Outcome)
	assert.Equal(t, logging.ActorBeacon, credAudits[0].Actor.Kind)
	assert.Equal(t, "task_jwt", credAudits[0].Actor.AuthMethod)
	assert.Equal(t, `CORP\domain_admin`, credAudits[0].Details["principal"])
	assert.NotContains(t, credAudits[0].Details, "secret")

	// Also verify invalid JWT emits c2.jwt_denied and never logs the bad token or credential secret.
	const badJWT = "eyJhbGciOiJFZERTQSJ9.invalidpayload.invalidsignature"
	_, err = client.ReportCredential(ctx, &c2pb.ReportCredentialRequest{
		Context: &c2pb.ReportCredentialRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{
				TaskId: int64(task.ID),
				Jwt:    badJWT,
			},
		},
		Credential: &epb.Credential{
			Principal: "admin",
			Secret:    credSecret,
			Kind:      epb.Credential_KIND_PASSWORD,
		},
	})
	require.Equal(t, codes.PermissionDenied, status.Code(err))
	assert.Len(t, rec.ByAction("c2.jwt_denied"), 1)

	auditJSON, err := json.Marshal(rec.Snapshot())
	require.NoError(t, err)
	for _, sensitive := range []string{credSecret, token, badJWT} {
		assert.NotContains(t, logBuf.String(), sensitive)
		assert.NotContains(t, string(auditJSON), sensitive)
	}
}

func TestAuditReportOutputFileAndFetchAssetMilestones(t *testing.T) {
	_, rec := setupC2AuditCapture(t)
	client, graph, closeFn, mintJWT := c2test.New(t)
	defer closeFn()

	ctx := context.Background()
	beacon := c2test.NewRandomBeacon(ctx, graph)
	taskOK := c2test.NewRandomAssignedTask(ctx, graph, beacon.Identifier)
	taskFail := c2test.NewRandomAssignedTask(ctx, graph, beacon.Identifier)
	token := mintJWT(beacon.ID)

	// 1. ReportOutput: streaming chunk without ExecFinishedAt emits NO audit; final chunk emits c2.task_completed.
	now := timestamppb.Now()
	_, err := client.ReportOutput(ctx, &c2pb.ReportOutputRequest{
		Message: &c2pb.ReportOutputRequest_TaskOutput{
			TaskOutput: &c2pb.ReportTaskOutputMessage{
				Context: &c2pb.TaskContext{TaskId: int64(taskOK.ID), Jwt: token},
				Output: &c2pb.TaskOutput{
					Id:            int64(taskOK.ID),
					Output:        "chunk-1...",
					ExecStartedAt: now,
				},
			},
		},
	})
	require.NoError(t, err)
	assert.Empty(t, rec.ByAction("c2.task_completed"), "streaming output chunk must not emit completion audit")
	assert.Empty(t, rec.ByAction("c2.task_failed"))

	_, err = client.ReportOutput(ctx, &c2pb.ReportOutputRequest{
		Message: &c2pb.ReportOutputRequest_TaskOutput{
			TaskOutput: &c2pb.ReportTaskOutputMessage{
				Context: &c2pb.TaskContext{TaskId: int64(taskOK.ID), Jwt: token},
				Output: &c2pb.TaskOutput{
					Id:             int64(taskOK.ID),
					Output:         "done",
					ExecFinishedAt: timestamppb.New(time.Now().UTC()),
				},
			},
		},
	})
	require.NoError(t, err)
	completed := rec.ByAction("c2.task_completed")
	require.Len(t, completed, 1)
	assert.Equal(t, logging.OutcomeSuccess, completed[0].Outcome)

	// Task reporting an error on completion emits c2.task_failed with OutcomeFailure.
	_, err = client.ReportOutput(ctx, &c2pb.ReportOutputRequest{
		Message: &c2pb.ReportOutputRequest_TaskOutput{
			TaskOutput: &c2pb.ReportTaskOutputMessage{
				Context: &c2pb.TaskContext{TaskId: int64(taskFail.ID), Jwt: token},
				Output: &c2pb.TaskOutput{
					Id:             int64(taskFail.ID),
					Error:          &c2pb.TaskError{Msg: "eldritch runtime error"},
					ExecStartedAt:  now,
					ExecFinishedAt: timestamppb.New(time.Now().UTC()),
				},
			},
		},
	})
	require.NoError(t, err)
	failed := rec.ByAction("c2.task_failed")
	require.Len(t, failed, 1)
	assert.Equal(t, logging.OutcomeFailure, failed[0].Outcome)

	// 2. ReportFile: multi-chunk upload emits a single c2.file_reported milestone.
	rec.Reset()
	fileStream, err := client.ReportFile(ctx)
	require.NoError(t, err)
	require.NoError(t, fileStream.Send(&c2pb.ReportFileRequest{
		Context: &c2pb.ReportFileRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(taskOK.ID), Jwt: token},
		},
		Chunk: &epb.File{
			Metadata: &epb.FileMetadata{
				Path:        "/etc/hosts",
				Owner:       "root",
				Group:       "root",
				Permissions: "0644",
			},
			Chunk: []byte("127.0.0.1 localhost\n"),
		},
	}))
	require.NoError(t, fileStream.Send(&c2pb.ReportFileRequest{
		Chunk: &epb.File{
			Chunk: []byte("10.0.0.5 internal\n"),
		},
	}))
	_, err = fileStream.CloseAndRecv()
	require.NoError(t, err)

	filesReported := rec.ByAction("c2.file_reported")
	require.Len(t, filesReported, 1)
	assert.Equal(t, "/etc/hosts", filesReported[0].Details["path"])
	assert.Equal(t, c2pb.ReportFileKind_REPORT_FILE_KIND_UNSPECIFIED.String(), filesReported[0].Details["kind"])

	// 3. FetchAsset: multi-chunk asset download emits c2.asset_fetched once per RPC (not per chunk).
	rec.Reset()
	payload := make([]byte, 2*1024*1024+512) // > 2MB -> multiple 1MB chunks
	_, err = rand.Read(payload)
	require.NoError(t, err)
	asset := graph.Asset.Create().
		SetName("multi-chunk-payload.bin").
		SetContent(payload).
		SaveX(ctx)

	fetchStream, err := client.FetchAsset(ctx, &c2pb.FetchAssetRequest{
		Name: asset.Name,
		Context: &c2pb.FetchAssetRequest_TaskContext{
			TaskContext: &c2pb.TaskContext{TaskId: int64(taskOK.ID), Jwt: token},
		},
	})
	require.NoError(t, err)

	chunks := 0
	for {
		_, err := fetchStream.Recv()
		if errors.Is(err, io.EOF) {
			break
		}
		require.NoError(t, err)
		chunks++
	}
	require.Greater(t, chunks, 2, "expected multi-chunk stream")

	fetched := rec.ByAction("c2.asset_fetched")
	require.Len(t, fetched, 1, "c2.asset_fetched must be emitted once per FetchAsset call, not per chunk")
	assert.Equal(t, asset.Name, fetched[0].Details["asset_name"])
	assert.Equal(t, asset.Hash, fetched[0].Details["asset_hash"])
	assert.EqualValues(t, asset.Size, fetched[0].Details["asset_size"])
}
