package builder_test

import (
	"bytes"
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"encoding/pem"
	"net"
	"path/filepath"
	"strconv"
	"sync"
	"testing"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"

	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/logging"
)

type syncBuilderLogBuf struct {
	mu  sync.Mutex
	buf bytes.Buffer
}

func (b *syncBuilderLogBuf) Write(p []byte) (int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.Write(p)
}

type builderAuditRecorder struct {
	mu      sync.Mutex
	entries []logging.AuditEntry
}

func (r *builderAuditRecorder) Enqueue(e logging.AuditEntry) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.entries = append(r.entries, e)
}

func (r *builderAuditRecorder) Reset() {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.entries = nil
}

func (r *builderAuditRecorder) ByAction(action string) []logging.AuditEntry {
	r.mu.Lock()
	defer r.mu.Unlock()
	var out []logging.AuditEntry
	for _, e := range r.entries {
		if e.Action == action {
			out = append(out, e)
		}
	}
	return out
}

func newTestBuilderCredentials(t *testing.T, ca *x509.Certificate, caKey ed25519.PrivateKey, id string) *builder.Config {
	t.Helper()
	_, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	csrDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, privKey)
	require.NoError(t, err)
	csrPEM := string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrDER}))

	certPEM, err := builder.SignBuilderCertificate(ca, caKey, id, csrPEM)
	require.NoError(t, err)
	keyDER, err := x509.MarshalPKCS8PrivateKey(privKey)
	require.NoError(t, err)
	keyPEM := string(pem.EncodeToMemory(&pem.Block{Type: "PRIVATE KEY", Bytes: keyDER}))

	idPath := filepath.Join(t.TempDir(), "builder.identity")
	require.NoError(t, builder.SaveIdentity(idPath, &builder.Identity{
		ID:          id,
		Certificate: string(certPEM),
		PrivateKey:  keyPEM,
	}))
	return &builder.Config{
		IdentityFile:     idPath,
		SupportedTargets: []string{"linux"},
		Upstream:         "http://127.0.0.1:8000",
		Executor:         "local",
		PollInterval:     5,
	}
}

func TestBuilderMTLSAndTaskAudits(t *testing.T) {
	logBuf := &syncBuilderLogBuf{}
	rec := &builderAuditRecorder{}
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

	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	caCert, err := builder.CreateCA(caKey)
	require.NoError(t, err)

	// Register Builder in Ent DB
	dbBuilder := graph.Builder.Create().
		SetIdentifier("builder-audit-01").
		SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_LINUX}).
		SetPollInterval(5).
		SaveX(ctx)

	lis := bufconn.Listen(1024 * 1024)
	grpcSrv := grpc.NewServer(
		grpc.ChainUnaryInterceptor(builder.NewMTLSAuthInterceptor(caCert, graph)),
		grpc.ChainStreamInterceptor(builder.NewMTLSStreamAuthInterceptor(caCert, graph)),
	)
	builderpb.RegisterBuilderServer(grpcSrv, builder.New(graph, "dGVzdC1wdWJrZXk="))
	go func() {
		_ = grpcSrv.Serve(lis)
	}()
	defer grpcSrv.Stop()

	dialer := func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}

	// 1. Unauthenticated / untrusted CA requests emit builder.mtls_denied under audit.auth
	t.Run("MTLSAuthDeniedEmitsAudit", func(t *testing.T) {
		rec.Reset()
		unauthConn, err := grpc.NewClient("passthrough:///bufnet",
			grpc.WithContextDialer(dialer),
			grpc.WithTransportCredentials(insecure.NewCredentials()),
		)
		require.NoError(t, err)
		defer unauthConn.Close()

		unauthClient := builderpb.NewBuilderClient(unauthConn)
		_, err = unauthClient.ClaimBuildTasks(ctx, &builderpb.ClaimBuildTasksRequest{})
		require.Equal(t, codes.Unauthenticated, status.Code(err))

		// Also test untrusted CA cert on a streaming RPC
		_, rogueCAKey, err := ed25519.GenerateKey(rand.Reader)
		require.NoError(t, err)
		rogueCA, err := builder.CreateCA(rogueCAKey)
		require.NoError(t, err)
		rogueCfg := newTestBuilderCredentials(t, rogueCA, rogueCAKey, dbBuilder.Identifier)
		rogueCreds, err := builder.NewCredentialsFromConfig(rogueCfg)
		require.NoError(t, err)

		rogueConn, err := grpc.NewClient("passthrough:///bufnet",
			grpc.WithContextDialer(dialer),
			grpc.WithTransportCredentials(insecure.NewCredentials()),
			grpc.WithPerRPCCredentials(rogueCreds),
		)
		require.NoError(t, err)
		defer rogueConn.Close()

		stream, err := builderpb.NewBuilderClient(rogueConn).StreamBuildTaskOutput(ctx)
		require.NoError(t, err)
		_, err = stream.CloseAndRecv()
		require.Equal(t, codes.Unauthenticated, status.Code(err))

		denied := rec.ByAction("builder.mtls_denied")
		require.Len(t, denied, 2)
		assert.Equal(t, "audit.auth", denied[0].Subsystem)
		assert.Equal(t, logging.OutcomeDenied, denied[0].Outcome)
		assert.Equal(t, "missing_credentials", denied[0].Details["reason"])
		assert.Equal(t, "untrusted_ca", denied[1].Details["reason"])
	})

	// 2. Authenticated builder attaches ActorBuilder and emits build_task.completed, build_task.failed, and build_task.artifact_uploaded
	t.Run("BuildLifecycleAuditsAndActorAttribution", func(t *testing.T) {
		rec.Reset()
		cfg := newTestBuilderCredentials(t, caCert, caKey, dbBuilder.Identifier)
		creds, err := builder.NewCredentialsFromConfig(cfg)
		require.NoError(t, err)

		conn, err := grpc.NewClient("passthrough:///bufnet",
			grpc.WithContextDialer(dialer),
			grpc.WithTransportCredentials(insecure.NewCredentials()),
			grpc.WithPerRPCCredentials(creds),
		)
		require.NoError(t, err)
		defer conn.Close()

		client := builderpb.NewBuilderClient(conn)

		profile := graph.BuildProfile.Create().
			SetName("linux-prod").
			SetBuildImage("rust:1.75").
			SetDescription("Production Linux profile").
			SaveX(ctx)
		snap := builder.NewProfileSnapshot(profile)

		btSuccess := graph.BuildTask.Create().
			SetTargetOs(c2pb.Host_PLATFORM_LINUX).
			SetTargetFormat(builderpb.TargetFormat_TARGET_FORMAT_BIN).
			SetBuildScript("cargo build --release").
			SetProfileID(profile.ID).
			SetProfileAtCreation(snap).
			SetBuilderID(dbBuilder.ID).
			SaveX(ctx)

		btFail := graph.BuildTask.Create().
			SetTargetOs(c2pb.Host_PLATFORM_LINUX).
			SetTargetFormat(builderpb.TargetFormat_TARGET_FORMAT_BIN).
			SetBuildScript("cargo build --broken").
			SetProfileID(profile.ID).
			SetProfileAtCreation(snap).
			SetBuilderID(dbBuilder.ID).
			SaveX(ctx)

		claimResp, err := client.ClaimBuildTasks(ctx, &builderpb.ClaimBuildTasksRequest{})
		require.NoError(t, err)
		require.Len(t, claimResp.Tasks, 2)

		// Stream completion for btSuccess
		outStream, err := client.StreamBuildTaskOutput(ctx)
		require.NoError(t, err)
		require.NoError(t, outStream.Send(&builderpb.StreamBuildTaskOutputRequest{
			TaskId: int64(btSuccess.ID),
			Output: "Compiling imix v0.1.0",
		}))
		assert.Empty(t, rec.ByAction("build_task.completed"), "non-final output chunk must not emit completion audit")
		require.NoError(t, outStream.Send(&builderpb.StreamBuildTaskOutputRequest{
			TaskId:   int64(btSuccess.ID),
			Finished: true,
			ExitCode: 0,
		}))
		_, err = outStream.CloseAndRecv()
		require.NoError(t, err)

		completed := rec.ByAction("build_task.completed")
		require.Len(t, completed, 1)
		assert.Equal(t, "audit.builder", completed[0].Subsystem)
		assert.Equal(t, logging.OutcomeSuccess, completed[0].Outcome)
		assert.Equal(t, logging.ActorBuilder, completed[0].Actor.Kind)
		assert.Equal(t, strconv.Itoa(dbBuilder.ID), completed[0].Actor.ID)
		assert.Equal(t, dbBuilder.Identifier, completed[0].Actor.Name)
		assert.Equal(t, "mtls", completed[0].Actor.AuthMethod)
		assert.EqualValues(t, btSuccess.ID, completed[0].Details["build_task_id"])
		assert.EqualValues(t, dbBuilder.ID, completed[0].Details["builder_id"])

		// Upload artifact for btSuccess
		uploadStream, err := client.UploadBuildArtifact(ctx)
		require.NoError(t, err)
		artifactBytes := []byte("\x7fELF-fake-imix-binary-payload")
		require.NoError(t, uploadStream.Send(&builderpb.UploadBuildArtifactRequest{
			TaskId:       int64(btSuccess.ID),
			ArtifactName: "imix",
			Chunk:        artifactBytes[:10],
		}))
		require.NoError(t, uploadStream.Send(&builderpb.UploadBuildArtifactRequest{
			Chunk: artifactBytes[10:],
		}))
		uploadResp, err := uploadStream.CloseAndRecv()
		require.NoError(t, err)
		require.NotZero(t, uploadResp.AssetId)

		uploaded := rec.ByAction("build_task.artifact_uploaded")
		require.Len(t, uploaded, 1)
		assert.Equal(t, "audit.builder", uploaded[0].Subsystem)
		assert.Equal(t, logging.OutcomeSuccess, uploaded[0].Outcome)
		assert.Equal(t, logging.ActorBuilder, uploaded[0].Actor.Kind)
		assert.EqualValues(t, btSuccess.ID, uploaded[0].Details["build_task_id"])
		assert.EqualValues(t, uploadResp.AssetId, uploaded[0].Details["asset_id"])
		assert.EqualValues(t, len(artifactBytes), uploaded[0].Details["asset_size"])
		assert.NotEmpty(t, uploaded[0].Details["asset_hash"])

		// Stream failure for btFail
		failStream, err := client.StreamBuildTaskOutput(ctx)
		require.NoError(t, err)
		require.NoError(t, failStream.Send(&builderpb.StreamBuildTaskOutputRequest{
			TaskId:   int64(btFail.ID),
			Error:    "linker error",
			Finished: true,
			ExitCode: 101,
		}))
		_, err = failStream.CloseAndRecv()
		require.NoError(t, err)

		failed := rec.ByAction("build_task.failed")
		require.Len(t, failed, 1)
		assert.Equal(t, "audit.builder", failed[0].Subsystem)
		assert.Equal(t, logging.OutcomeFailure, failed[0].Outcome)
		assert.Equal(t, logging.ActorBuilder, failed[0].Actor.Kind)
		assert.EqualValues(t, btFail.ID, failed[0].Details["build_task_id"])
		assert.EqualValues(t, 101, failed[0].Details["exit_code"])
	})
}
