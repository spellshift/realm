package main

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/base64"
	"encoding/json"
	"encoding/pem"
	"math/big"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/crypto"
	"realm.pub/tavern/internal/enrollment"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/redirector"
	"realm.pub/tavern/internal/ent/redirectorenrollmenttoken"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/redirectors/managed"
	"realm.pub/tavern/internal/secrets"
)

type sharedRedirectorIdentity struct {
	mu       sync.Mutex
	identity *managed.Identity
}

func (s *sharedRedirectorIdentity) Load(context.Context) (*managed.Identity, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.identity == nil {
		return nil, os.ErrNotExist
	}
	return s.identity, nil
}

func (s *sharedRedirectorIdentity) Save(_ context.Context, identity *managed.Identity) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.identity = identity
	return nil
}

type threadSafeAuditRecorder struct {
	mu      sync.Mutex
	entries []logging.AuditEntry
}

func (r *threadSafeAuditRecorder) Enqueue(entry logging.AuditEntry) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.entries = append(r.entries, entry)
}

func (r *threadSafeAuditRecorder) Reset() {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.entries = nil
}

func (r *threadSafeAuditRecorder) Snapshot() []logging.AuditEntry {
	r.mu.Lock()
	defer r.mu.Unlock()
	out := make([]logging.AuditEntry, len(r.entries))
	copy(out, r.entries)
	return out
}

func newFullStackRedirectorServer(t *testing.T, graph *ent.Client, ca *x509.Certificate, caKey ed25519.PrivateKey) *httptest.Server {
	t.Helper()
	routes := tavernhttp.RouteMap{
		"/auth/redirector/enroll": tavernhttp.Endpoint{
			Handler:              tavernhttp.NewRedirectorEnrollHandler(graph, ca, caKey),
			AllowUnauthenticated: true,
			AllowUnactivated:     true,
		},
		"/auth/redirector/checkin": tavernhttp.Endpoint{
			Handler:              tavernhttp.NewRedirectorCheckinHandler(graph, ca),
			AllowUnauthenticated: true,
			AllowUnactivated:     true,
		},
		"/auth/redirector/renew": tavernhttp.Endpoint{
			Handler:              tavernhttp.NewRedirectorRenewHandler(graph, ca, caKey),
			AllowUnauthenticated: true,
			AllowUnactivated:     true,
		},
	}
	srv := tavernhttp.NewServer(routes, tavernhttp.WithAuthentication(graph))
	ts := httptest.NewServer(srv)
	t.Cleanup(ts.Close)
	return ts
}

func initSecretsAndCA(t *testing.T, secretsPath string) (*x509.Certificate, ed25519.PrivateKey) {
	t.Helper()
	sm, err := secrets.NewDebugFileSecrets(secretsPath)
	require.NoError(t, err)
	caKey, err := crypto.GetPrivKeyED25519(sm)
	require.NoError(t, err)
	ca, err := builder.CreateCA(caKey)
	require.NoError(t, err)
	return ca, caKey
}

func mintTestRedirectorCert(t *testing.T, ca *x509.Certificate, caKey ed25519.PrivateKey, template x509.Certificate) (*x509.Certificate, ed25519.PrivateKey, string, string) {
	t.Helper()
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	if template.SerialNumber == nil {
		serial, err := rand.Int(rand.Reader, new(big.Int).Lsh(big.NewInt(1), 128))
		require.NoError(t, err)
		template.SerialNumber = serial
	}
	certDER, err := x509.CreateCertificate(rand.Reader, &template, ca, pub, caKey)
	require.NoError(t, err)
	cert, err := x509.ParseCertificate(certDER)
	require.NoError(t, err)
	keyDER, err := x509.MarshalPKCS8PrivateKey(priv)
	require.NoError(t, err)
	certPEM := string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: certDER}))
	keyPEM := string(pem.EncodeToMemory(&pem.Block{Type: "PRIVATE KEY", Bytes: keyDER}))
	return cert, priv, certPEM, keyPEM
}

func sendRawCheckin(t *testing.T, serverURL, certHeader, timestamp, sigHeader string) int {
	t.Helper()
	req, err := http.NewRequest(http.MethodPost, serverURL+"/auth/redirector/checkin", nil)
	require.NoError(t, err)
	req.Header.Set("X-Realm-Redirector-Cert", certHeader)
	req.Header.Set("X-Realm-Redirector-Timestamp", timestamp)
	req.Header.Set("X-Realm-Redirector-Signature", sigHeader)
	resp, err := http.DefaultClient.Do(req)
	require.NoError(t, err)
	defer resp.Body.Close()
	return resp.StatusCode
}

func findAuditEntry(entries []logging.AuditEntry, action, reason string) *logging.AuditEntry {
	for i := range entries {
		e := &entries[i]
		if e.Action != action {
			continue
		}
		if reason != "" && e.Details["reason"] != reason {
			continue
		}
		return e
	}
	return nil
}

func assertNoSecretMaterialInAudit(t *testing.T, entry *logging.AuditEntry, forbidden ...string) {
	t.Helper()
	rawDetails, err := json.Marshal(entry.Details)
	require.NoError(t, err)
	payload := string(rawDetails) + " " + entry.Summary
	require.NotContains(t, payload, "BEGIN CERTIFICATE")
	require.NotContains(t, payload, "BEGIN PRIVATE KEY")
	for _, secret := range forbidden {
		if secret != "" {
			require.NotContains(t, payload, secret)
		}
	}
}

func TestManagedRedirectorEnrollmentAndCheckins(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()
	require.NoError(t, seedRedirectorEnrollmentTokens(ctx, graph, []string{"one-use-token"}))
	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	ca, err := builder.CreateCA(caKey)
	require.NoError(t, err)
	mux := http.NewServeMux()
	mux.Handle("/auth/redirector/enroll", tavernhttp.NewRedirectorEnrollHandler(graph, ca, caKey))
	mux.Handle("/auth/redirector/checkin", tavernhttp.NewRedirectorCheckinHandler(graph, ca))
	server := httptest.NewServer(mux)
	defer server.Close()
	store := &sharedRedirectorIdentity{}
	cfg := managed.Config{Upstream: server.URL, Transport: "http1", Domains: []string{"example.com"}, Token: "one-use-token"}
	first, err := managed.EnsureIdentity(ctx, cfg, store)
	require.NoError(t, err)
	second, err := managed.EnsureIdentity(ctx, cfg, store)
	require.NoError(t, err)
	require.Equal(t, first.ID, second.ID)
	require.NoError(t, managed.Checkin(ctx, server.URL, first))
	require.NoError(t, managed.Checkin(ctx, server.URL, second))
	records, err := graph.Redirector.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, records, 1)
	require.NotNil(t, records[0].LastSeenAt)
	require.WithinDuration(t, time.Now(), *records[0].LastSeenAt, 5*time.Second)
	// Signed but stale timestamps and malformed proofs are rejected.
	req := httptest.NewRequest(http.MethodPost, "/auth/redirector/checkin", nil)
	certBlock, _ := pem.Decode([]byte(first.Certificate))
	require.NotNil(t, certBlock)
	req.Header.Set("X-Realm-Redirector-Cert", base64.StdEncoding.EncodeToString(certBlock.Bytes))
	req.Header.Set("X-Realm-Redirector-Timestamp", time.Now().UTC().Format(time.RFC3339Nano))
	req.Header.Set("X-Realm-Redirector-Signature", base64.StdEncoding.EncodeToString([]byte("invalid")))
	recorder := httptest.NewRecorder()
	mux.ServeHTTP(recorder, req)
	require.Equal(t, http.StatusUnauthorized, recorder.Code)
	keyBlock, _ := pem.Decode([]byte(first.PrivateKey))
	require.NotNil(t, keyBlock)
	keyAny, err := x509.ParsePKCS8PrivateKey(keyBlock.Bytes)
	require.NoError(t, err)
	stale := time.Now().Add(-10 * time.Minute).UTC().Format(time.RFC3339Nano)
	req.Header.Set("X-Realm-Redirector-Timestamp", stale)
	req.Header.Set("X-Realm-Redirector-Signature", base64.StdEncoding.EncodeToString(ed25519.Sign(keyAny.(ed25519.PrivateKey), []byte(stale))))
	recorder = httptest.NewRecorder()
	mux.ServeHTTP(recorder, req)
	require.Equal(t, http.StatusUnauthorized, recorder.Code)
	recordCount, err := graph.Redirector.Query().Count(ctx)
	require.NoError(t, err)
	require.Equal(t, 1, recordCount)
}

func TestManagedRedirectorFullStackSecurityIntegration(t *testing.T) {
	ctx := context.Background()
	audits := &threadSafeAuditRecorder{}
	logging.SetAuditPersister(audits)
	t.Cleanup(func() { logging.SetAuditPersister(nil) })

	// 1. Initialize DebugFileSecrets + crypto.GetPrivKeyED25519 + builder.CreateCA and verify 0600 file perms.
	secretsDir := t.TempDir()
	secretsPath := filepath.Join(secretsDir, "tavern-secrets.yaml")
	ca, caKey := initSecretsAndCA(t, secretsPath)

	secretsInfo, err := os.Stat(secretsPath)
	require.NoError(t, err)
	require.Equal(t, os.FileMode(0600), secretsInfo.Mode().Perm())
	require.Zero(t, secretsInfo.Mode().Perm()&0077)

	// Verify DebugFileSecrets rejects symlinks and 0644 files when used with crypto.GetPrivKeyED25519.
	symlinkSecretsPath := filepath.Join(secretsDir, "secrets-symlink.yaml")
	require.NoError(t, os.Symlink(secretsPath, symlinkSecretsPath))
	symlinkSM, err := secrets.NewDebugFileSecrets(symlinkSecretsPath)
	require.NoError(t, err)
	_, err = crypto.GetPrivKeyED25519(symlinkSM)
	require.Error(t, err)
	_, err = symlinkSM.SetValue("test_key", []byte("test_value"))
	require.Error(t, err)

	broadSecretsPath := filepath.Join(secretsDir, "broad-secrets.yaml")
	require.NoError(t, os.WriteFile(broadSecretsPath, []byte("[]\n"), 0644))
	require.NoError(t, os.Chmod(broadSecretsPath, 0644))
	broadSM, err := secrets.NewDebugFileSecrets(broadSecretsPath)
	require.NoError(t, err)
	_, err = crypto.GetPrivKeyED25519(broadSM)
	require.ErrorContains(t, err, "insecure permissions")
	_, err = broadSM.SetValue("test_key", []byte("test_value"))
	require.ErrorContains(t, err, "insecure permissions")

	// 2. Mount endpoints through tavernhttp.NewServer with ent DB and AuditPersister.
	graph := enttest.OpenTempDB(t)
	defer graph.Close()
	server := newFullStackRedirectorServer(t, graph, ca, caKey)

	// Parse whitespace-padded token as server config does and seed into DB.
	rawToken := "  full-stack-enroll-token  "
	parsedTokens, err := parseRedirectorEnrollmentTokens(rawToken)
	require.NoError(t, err)
	require.NoError(t, seedRedirectorEnrollmentTokens(ctx, graph, parsedTokens))

	// 3. Enroll managed redirector using real fileStore (managed.OpenStore + managed.EnsureIdentity).
	identityPath := filepath.Join(t.TempDir(), "redirector-identity.json")
	cfg := managed.Config{
		Upstream:     server.URL,
		Transport:    "http1",
		Domains:      []string{"edge.example.com"},
		Token:        strings.TrimSpace(rawToken),
		IdentityFile: identityPath,
	}
	store, closeStore, err := managed.OpenStore(ctx, cfg)
	require.NoError(t, err)
	defer closeStore()

	enrolled, err := managed.EnsureIdentity(ctx, cfg, store)
	require.NoError(t, err)
	require.NotEmpty(t, enrolled.ID)

	idInfo, err := os.Stat(identityPath)
	require.NoError(t, err)
	require.Equal(t, os.FileMode(0600), idInfo.Mode().Perm())
	require.Zero(t, idInfo.Mode().Perm()&0077)

	enrolledAudit := findAuditEntry(audits.Snapshot(), "redirector.enrolled", "")
	require.NotNil(t, enrolledAudit)
	require.Equal(t, logging.OutcomeSuccess, enrolledAudit.Outcome)
	require.NotEmpty(t, enrolledAudit.TraceID)
	require.NotEmpty(t, enrolledAudit.RequestID)

	// Verify managed.Checkin updates last_seen_at.
	require.NoError(t, managed.Checkin(ctx, server.URL, enrolled))
	rec, err := graph.Redirector.Query().Where(redirector.Identifier(enrolled.ID)).Only(ctx)
	require.NoError(t, err)
	require.NotNil(t, rec.LastSeenAt)
	require.WithinDuration(t, time.Now(), *rec.LastSeenAt, 5*time.Second)

	// 4. Disable redirector in DB and verify check-in STILL succeeds (204) and advances last_seen_at.
	olderLastSeen := time.Now().Add(-10 * time.Minute)
	require.NoError(t, graph.Redirector.UpdateOneID(rec.ID).SetEnabled(false).SetLastSeenAt(olderLastSeen).Exec(ctx))
	require.NoError(t, managed.Checkin(ctx, server.URL, enrolled))
	recAfterDisable, err := graph.Redirector.Query().Where(redirector.Identifier(enrolled.ID)).Only(ctx)
	require.NoError(t, err)
	require.False(t, recAfterDisable.Enabled)
	require.NotNil(t, recAfterDisable.LastSeenAt)
	require.True(t, recAfterDisable.LastSeenAt.After(olderLastSeen.Add(5*time.Minute)), "expected disabled redirector check-in to advance last_seen_at")

	// 5. Rejected check-ins through tavernhttp.NewServer: verify 401 + redirector.checkin_denied audit + no secret leakage.
	now := time.Now()

	// Create and delete a redirector record to verify deleted redirector check-ins are rejected as unknown_redirector.
	deletedRec, err := graph.Redirector.Create().
		SetIdentifier("deleted-redirector-id").
		SetTransport(redirector.TransportHttp1).
		SetDomains([]string{"deleted.example.com"}).
		Save(ctx)
	require.NoError(t, err)
	require.NoError(t, graph.Redirector.DeleteOneID(deletedRec.ID).Exec(ctx))

	// Foreign CA
	_, foreignCAKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	foreignCA, err := builder.CreateCA(foreignCAKey)
	require.NoError(t, err)
	foreignCert, foreignKey, _, _ := mintTestRedirectorCert(t, foreignCA, foreignCAKey, x509.Certificate{
		Subject:               pkix.Name{CommonName: "redirector-" + enrolled.ID},
		NotBefore:             now.Add(-1 * time.Hour),
		NotAfter:              now.Add(24 * time.Hour),
		KeyUsage:              x509.KeyUsageDigitalSignature,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageClientAuth},
		BasicConstraintsValid: true,
	})

	// Builder cert signed by real CA
	_, builderKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	builderCSRDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, builderKey)
	require.NoError(t, err)
	builderCertPEM, err := builder.SignBuilderCertificate(ca, caKey, enrolled.ID, string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: builderCSRDER})))
	require.NoError(t, err)
	builderBlock, _ := pem.Decode(builderCertPEM)
	require.NotNil(t, builderBlock)

	// Expired cert signed by real CA
	expiredCert, expiredKey, _, _ := mintTestRedirectorCert(t, ca, caKey, x509.Certificate{
		Subject:               pkix.Name{CommonName: "redirector-" + enrolled.ID},
		NotBefore:             now.Add(-48 * time.Hour),
		NotAfter:              now.Add(-1 * time.Hour),
		KeyUsage:              x509.KeyUsageDigitalSignature,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageClientAuth},
		BasicConstraintsValid: true,
	})

	// Empty redirector ID CN="redirector-"
	emptyIDCert, emptyIDKey, _, _ := mintTestRedirectorCert(t, ca, caKey, x509.Certificate{
		Subject:               pkix.Name{CommonName: "redirector-"},
		NotBefore:             now.Add(-1 * time.Hour),
		NotAfter:              now.Add(24 * time.Hour),
		KeyUsage:              x509.KeyUsageDigitalSignature,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageClientAuth},
		BasicConstraintsValid: true,
	})

	// Missing KeyUsageDigitalSignature
	noDigitalSigCert, noDigitalSigKey, _, _ := mintTestRedirectorCert(t, ca, caKey, x509.Certificate{
		Subject:               pkix.Name{CommonName: "redirector-" + enrolled.ID},
		NotBefore:             now.Add(-1 * time.Hour),
		NotAfter:              now.Add(24 * time.Hour),
		KeyUsage:              x509.KeyUsageKeyEncipherment,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageClientAuth},
		BasicConstraintsValid: true,
	})

	// Valid cert for deleted/unknown redirector ID
	deletedCert, deletedKey, _, _ := mintTestRedirectorCert(t, ca, caKey, x509.Certificate{
		Subject:               pkix.Name{CommonName: "redirector-deleted-redirector-id"},
		NotBefore:             now.Add(-1 * time.Hour),
		NotAfter:              now.Add(24 * time.Hour),
		KeyUsage:              x509.KeyUsageDigitalSignature,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageClientAuth},
		BasicConstraintsValid: true,
	})

	type rejectionCase struct {
		name           string
		certHeader     string
		signKey        ed25519.PrivateKey
		expectedReason string
	}
	cases := []rejectionCase{
		{
			name:           "oversized cert header",
			certHeader:     strings.Repeat("A", 8193),
			signKey:        foreignKey,
			expectedReason: "missing_or_oversized_cert",
		},
		{
			name:           "untrusted foreign CA cert",
			certHeader:     base64.StdEncoding.EncodeToString(foreignCert.Raw),
			signKey:        foreignKey,
			expectedReason: "untrusted_ca",
		},
		{
			name:           "builder cert rejected",
			certHeader:     base64.StdEncoding.EncodeToString(builderBlock.Bytes),
			signKey:        builderKey,
			expectedReason: "invalid_cn",
		},
		{
			name:           "expired cert rejected",
			certHeader:     base64.StdEncoding.EncodeToString(expiredCert.Raw),
			signKey:        expiredKey,
			expectedReason: "cert_expired_or_not_yet_valid",
		},
		{
			name:           "empty redirector id CN rejected",
			certHeader:     base64.StdEncoding.EncodeToString(emptyIDCert.Raw),
			signKey:        emptyIDKey,
			expectedReason: "invalid_cn",
		},
		{
			name:           "missing digital signature key usage",
			certHeader:     base64.StdEncoding.EncodeToString(noDigitalSigCert.Raw),
			signKey:        noDigitalSigKey,
			expectedReason: "missing_digital_signature_ku",
		},
		{
			name:           "deleted unknown redirector id",
			certHeader:     base64.StdEncoding.EncodeToString(deletedCert.Raw),
			signKey:        deletedKey,
			expectedReason: "unknown_redirector",
		},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			audits.Reset()
			ts := time.Now().UTC().Format(time.RFC3339Nano)
			sigHeader := base64.StdEncoding.EncodeToString(ed25519.Sign(tc.signKey, []byte(ts)))

			statusCode := sendRawCheckin(t, server.URL, tc.certHeader, ts, sigHeader)
			require.Equal(t, http.StatusUnauthorized, statusCode)

			entry := findAuditEntry(audits.Snapshot(), "redirector.checkin_denied", tc.expectedReason)
			require.NotNil(t, entry, "expected redirector.checkin_denied audit with reason %q", tc.expectedReason)
			require.Equal(t, logging.OutcomeDenied, entry.Outcome)
			require.NotEmpty(t, entry.TraceID)
			require.NotEmpty(t, entry.RequestID)
			assertNoSecretMaterialInAudit(t, entry, tc.certHeader, sigHeader)
		})
	}
}

func TestManagedRedirectorConcurrentMultiReplicaEnrollment(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	ca, caKey := initSecretsAndCA(t, filepath.Join(t.TempDir(), "secrets.yaml"))
	server := newFullStackRedirectorServer(t, graph, ca, caKey)

	const singleUseToken = "multi-replica-one-use-token"
	require.NoError(t, seedRedirectorEnrollmentTokens(ctx, graph, []string{singleUseToken}))

	sharedIdentityPath := filepath.Join(t.TempDir(), "shared-redirector-identity.json")
	cfg := managed.Config{
		Upstream:     server.URL,
		Transport:    "http1",
		Domains:      []string{"replica.example.com"},
		Token:        singleUseToken,
		IdentityFile: sharedIdentityPath,
	}

	const replicas = 3
	runCtx, cancel := context.WithTimeout(ctx, 15*time.Second)
	defer cancel()

	start := make(chan struct{})
	type replicaResult struct {
		identity *managed.Identity
		err      error
	}
	results := make([]replicaResult, replicas)
	var wg sync.WaitGroup
	for i := range replicas {
		wg.Add(1)
		go func(idx int) {
			defer wg.Done()
			store, closeStore, err := managed.OpenStore(runCtx, cfg)
			if err != nil {
				results[idx] = replicaResult{err: err}
				return
			}
			defer closeStore()
			<-start
			id, err := managed.EnsureIdentity(runCtx, cfg, store)
			results[idx] = replicaResult{identity: id, err: err}
		}(i)
	}

	close(start)
	wg.Wait()

	var expectedID string
	for i, res := range results {
		require.NoErrorf(t, res.err, "replica %d failed EnsureIdentity", i)
		require.NotNil(t, res.identity)
		if expectedID == "" {
			expectedID = res.identity.ID
		}
		require.Equal(t, expectedID, res.identity.ID)
		require.Equal(t, results[0].identity.Certificate, res.identity.Certificate)
		require.Equal(t, results[0].identity.PrivateKey, res.identity.PrivateKey)
		require.NoErrorf(t, managed.Checkin(ctx, server.URL, res.identity), "replica %d check-in failed", i)
	}

	redirectorsInDB, err := graph.Redirector.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, redirectorsInDB, 1)
	require.Equal(t, expectedID, redirectorsInDB[0].Identifier)
	require.NotNil(t, redirectorsInDB[0].LastSeenAt)

	tokenRow, err := graph.RedirectorEnrollmentToken.Query().
		Where(redirectorenrollmenttoken.TokenHash(enrollment.Hash(singleUseToken))).
		Only(ctx)
	require.NoError(t, err)
	require.True(t, tokenRow.Used)

	info, err := os.Stat(sharedIdentityPath)
	require.NoError(t, err)
	require.Equal(t, os.FileMode(0600), info.Mode().Perm())
}

func TestManagedRedirectorRenewalAndCrossReplicaReload(t *testing.T) {
	ctx := context.Background()
	audits := &threadSafeAuditRecorder{}
	logging.SetAuditPersister(audits)
	t.Cleanup(func() { logging.SetAuditPersister(nil) })

	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	ca, caKey := initSecretsAndCA(t, filepath.Join(t.TempDir(), "secrets.yaml"))
	server := newFullStackRedirectorServer(t, graph, ca, caKey)

	const enrollToken = "renewal-enroll-token"
	require.NoError(t, seedRedirectorEnrollmentTokens(ctx, graph, []string{enrollToken}))

	identityPath := filepath.Join(t.TempDir(), "redirector-identity.json")
	cfg := managed.Config{
		Upstream:     server.URL,
		Transport:    "http1",
		Domains:      []string{"renew.example.com"},
		Token:        enrollToken,
		IdentityFile: identityPath,
	}
	store, closeStore, err := managed.OpenStore(ctx, cfg)
	require.NoError(t, err)
	defer closeStore()

	enrolled, err := managed.EnsureIdentity(ctx, cfg, store)
	require.NoError(t, err)
	require.NoError(t, managed.Checkin(ctx, server.URL, enrolled))

	// 1. Explicit managed.Renew through tavernhttp.NewServer.
	audits.Reset()
	renewed, err := managed.Renew(ctx, server.URL, enrolled, store)
	require.NoError(t, err)
	require.Equal(t, enrolled.ID, renewed.ID)
	require.NotEqual(t, enrolled.Certificate, renewed.Certificate)
	require.NotEqual(t, enrolled.PrivateKey, renewed.PrivateKey)

	loadedAfterRenew, err := store.Load(ctx)
	require.NoError(t, err)
	require.Equal(t, renewed.ID, loadedAfterRenew.ID)
	require.Equal(t, renewed.Certificate, loadedAfterRenew.Certificate)
	require.Equal(t, renewed.PrivateKey, loadedAfterRenew.PrivateKey)

	info, err := os.Stat(identityPath)
	require.NoError(t, err)
	require.Equal(t, os.FileMode(0600), info.Mode().Perm())

	renewAudit := findAuditEntry(audits.Snapshot(), "redirector.renewed", "")
	require.NotNil(t, renewAudit)
	require.Equal(t, logging.OutcomeSuccess, renewAudit.Outcome)
	require.Equal(t, logging.ActorRedirector, renewAudit.Actor.Kind)
	require.Equal(t, enrolled.ID, renewAudit.Actor.Name)
	require.Equal(t, "mtls", renewAudit.Actor.AuthMethod)
	require.Equal(t, "redirector", renewAudit.TargetType)
	assertNoSecretMaterialInAudit(t, renewAudit, renewed.Certificate, renewed.PrivateKey)

	require.NoError(t, managed.Checkin(ctx, server.URL, renewed))

	// 2. Automatic near-expiry renewal via managed.RunCheckinsWithStore.
	now := time.Now()
	_, _, nearCertPEM, nearKeyPEM := mintTestRedirectorCert(t, ca, caKey, x509.Certificate{
		Subject:               pkix.Name{CommonName: "redirector-" + enrolled.ID},
		NotBefore:             now.Add(-1 * time.Hour),
		NotAfter:              now.Add(10 * 24 * time.Hour),
		KeyUsage:              x509.KeyUsageDigitalSignature,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageClientAuth},
		BasicConstraintsValid: true,
	})
	nearExpiry := &managed.Identity{
		ID:          enrolled.ID,
		Certificate: nearCertPEM,
		PrivateKey:  nearKeyPEM,
	}
	require.True(t, managed.NeedsRenewal(nearExpiry, now))

	replacer, ok := store.(managed.Replacer)
	require.True(t, ok, "fileStore must implement managed.Replacer")
	require.NoError(t, replacer.Replace(ctx, nearExpiry))

	rec, err := graph.Redirector.Query().Where(redirector.Identifier(enrolled.ID)).Only(ctx)
	require.NoError(t, err)
	oldLastSeen := time.Now().Add(-10 * time.Minute)
	require.NoError(t, graph.Redirector.UpdateOneID(rec.ID).SetLastSeenAt(oldLastSeen).Exec(ctx))

	audits.Reset()
	runCtx, cancelRun := context.WithCancel(ctx)
	runDone := make(chan struct{})
	go func() {
		defer close(runDone)
		managed.RunCheckinsWithStore(runCtx, server.URL, nearExpiry, store)
	}()

	var autoRenewed *managed.Identity
	require.Eventually(t, func() bool {
		loaded, loadErr := store.Load(ctx)
		if loadErr != nil || loaded.Certificate == nearExpiry.Certificate {
			return false
		}
		updatedRec, queryErr := graph.Redirector.Query().Where(redirector.Identifier(enrolled.ID)).Only(ctx)
		if queryErr != nil || updatedRec.LastSeenAt == nil || !updatedRec.LastSeenAt.After(oldLastSeen.Add(5*time.Minute)) {
			return false
		}
		autoRenewed = loaded
		return true
	}, 5*time.Second, 25*time.Millisecond)

	cancelRun()
	select {
	case <-runDone:
	case <-time.After(5 * time.Second):
		t.Fatal("RunCheckinsWithStore did not exit after context cancellation")
	}

	require.NotNil(t, autoRenewed)
	require.Equal(t, enrolled.ID, autoRenewed.ID)
	require.False(t, managed.NeedsRenewal(autoRenewed, time.Now()))
	block, _ := pem.Decode([]byte(autoRenewed.Certificate))
	require.NotNil(t, block)
	parsedRenewedCert, err := x509.ParseCertificate(block.Bytes)
	require.NoError(t, err)
	require.Greater(t, time.Until(parsedRenewedCert.NotAfter), 360*24*time.Hour)
	require.NotNil(t, findAuditEntry(audits.Snapshot(), "redirector.renewed", ""))

	info, err = os.Stat(identityPath)
	require.NoError(t, err)
	require.Equal(t, os.FileMode(0600), info.Mode().Perm())

	// 3. Cross-replica reload via RunCheckinsWithStore: a second replica holding a stale/untrusted cert
	// gets 401 on check-in, reloads the valid renewed identity from the shared fileStore, and succeeds.
	_, oldCAKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	oldCA, err := builder.CreateCA(oldCAKey)
	require.NoError(t, err)
	_, _, staleCertPEM, staleKeyPEM := mintTestRedirectorCert(t, oldCA, oldCAKey, x509.Certificate{
		Subject:               pkix.Name{CommonName: "redirector-" + enrolled.ID},
		NotBefore:             now.Add(-1 * time.Hour),
		NotAfter:              now.Add(180 * 24 * time.Hour),
		KeyUsage:              x509.KeyUsageDigitalSignature,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageClientAuth},
		BasicConstraintsValid: true,
	})
	staleReplicaIdentity := &managed.Identity{
		ID:          enrolled.ID,
		Certificate: staleCertPEM,
		PrivateKey:  staleKeyPEM,
	}
	require.False(t, managed.NeedsRenewal(staleReplicaIdentity, now))

	oldLastSeen = time.Now().Add(-15 * time.Minute)
	require.NoError(t, graph.Redirector.UpdateOneID(rec.ID).SetLastSeenAt(oldLastSeen).Exec(ctx))
	audits.Reset()

	reloadCtx, cancelReload := context.WithCancel(ctx)
	reloadDone := make(chan struct{})
	go func() {
		defer close(reloadDone)
		managed.RunCheckinsWithStore(reloadCtx, server.URL, staleReplicaIdentity, store)
	}()

	require.Eventually(t, func() bool {
		updatedRec, queryErr := graph.Redirector.Query().Where(redirector.Identifier(enrolled.ID)).Only(ctx)
		return queryErr == nil && updatedRec.LastSeenAt != nil && updatedRec.LastSeenAt.After(oldLastSeen.Add(5*time.Minute))
	}, 5*time.Second, 25*time.Millisecond)

	cancelReload()
	select {
	case <-reloadDone:
	case <-time.After(5 * time.Second):
		t.Fatal("RunCheckinsWithStore cross-replica reload goroutine did not exit after context cancellation")
	}

	require.NotNil(t, findAuditEntry(audits.Snapshot(), "redirector.checkin_denied", "untrusted_ca"))
}

func TestManagedRedirectorFileStoreHardeningWithEnsureIdentity(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	ca, caKey := initSecretsAndCA(t, filepath.Join(t.TempDir(), "secrets.yaml"))
	server := newFullStackRedirectorServer(t, graph, ca, caKey)

	require.NoError(t, seedRedirectorEnrollmentTokens(ctx, graph, []string{"initial-token", "unused-token"}))

	// First enroll a valid identity on disk so we have a real target file for symlink & permission tests.
	validPath := filepath.Join(t.TempDir(), "valid-identity.json")
	validCfg := managed.Config{
		Upstream:     server.URL,
		Transport:    "http1",
		Domains:      []string{"hardening.example.com"},
		Token:        "initial-token",
		IdentityFile: validPath,
	}
	validStore, closeValidStore, err := managed.OpenStore(ctx, validCfg)
	require.NoError(t, err)
	defer closeValidStore()

	enrolled, err := managed.EnsureIdentity(ctx, validCfg, validStore)
	require.NoError(t, err)
	require.NotEmpty(t, enrolled.ID)

	// 1. Symlink identity path is rejected when loading via managed.OpenStore + managed.EnsureIdentity
	// and must not trigger re-enrollment.
	t.Run("symlink identity path rejected", func(t *testing.T) {
		linkPath := filepath.Join(t.TempDir(), "identity-symlink.json")
		require.NoError(t, os.Symlink(validPath, linkPath))

		linkCfg := managed.Config{
			Upstream:     server.URL,
			Transport:    "http1",
			Domains:      []string{"hardening.example.com"},
			Token:        "unused-token",
			IdentityFile: linkPath,
		}
		store, closeStore, err := managed.OpenStore(ctx, linkCfg)
		require.NoError(t, err)
		defer closeStore()

		_, err = managed.EnsureIdentity(ctx, linkCfg, store)
		require.Error(t, err)

		count, err := graph.Redirector.Query().Count(ctx)
		require.NoError(t, err)
		require.Equal(t, 1, count)
		unusedRow, err := graph.RedirectorEnrollmentToken.Query().
			Where(redirectorenrollmenttoken.TokenHash(enrollment.Hash("unused-token"))).
			Only(ctx)
		require.NoError(t, err)
		require.False(t, unusedRow.Used)
	})

	// 2. Broad-permission (0644) identity file is rejected by managed.EnsureIdentity
	// and must not trigger re-enrollment.
	t.Run("broad permission identity file rejected", func(t *testing.T) {
		rawIdentity, err := os.ReadFile(validPath)
		require.NoError(t, err)
		broadPath := filepath.Join(t.TempDir(), "broad-identity.json")
		require.NoError(t, os.WriteFile(broadPath, rawIdentity, 0644))
		require.NoError(t, os.Chmod(broadPath, 0644))

		broadCfg := managed.Config{
			Upstream:     server.URL,
			Transport:    "http1",
			Domains:      []string{"hardening.example.com"},
			Token:        "unused-token",
			IdentityFile: broadPath,
		}
		store, closeStore, err := managed.OpenStore(ctx, broadCfg)
		require.NoError(t, err)
		defer closeStore()

		_, err = managed.EnsureIdentity(ctx, broadCfg, store)
		require.ErrorContains(t, err, "permissions are too broad")

		count, err := graph.Redirector.Query().Count(ctx)
		require.NoError(t, err)
		require.Equal(t, 1, count)
		unusedRow, err := graph.RedirectorEnrollmentToken.Query().
			Where(redirectorenrollmenttoken.TokenHash(enrollment.Hash("unused-token"))).
			Only(ctx)
		require.NoError(t, err)
		require.False(t, unusedRow.Used)
	})

	// 3. Supplying both IdentityFile and IdentitySecret fails in managed.OpenStore.
	t.Run("both identity file and identity secret rejected", func(t *testing.T) {
		_, _, err := managed.OpenStore(ctx, managed.Config{
			IdentityFile:   validPath,
			IdentitySecret: "projects/test-project/secrets/redirector-identity",
		})
		require.ErrorContains(t, err, "not both")
	})
}
