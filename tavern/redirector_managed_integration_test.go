package main

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"encoding/base64"
	"encoding/pem"
	"net/http"
	"net/http/httptest"
	"os"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/internal/redirectors"
	"realm.pub/tavern/internal/redirectors/managed"
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
	// The deployment state is persisted in Tavern and returned to every replica,
	// including one that reloads its identity after a restart.
	require.NoError(t, graph.Redirector.UpdateOneID(records[0].ID).SetEnabled(false).Exec(ctx))
	enabled, err := managed.CheckinEnabled(ctx, server.URL, first)
	require.NoError(t, err)
	require.False(t, enabled)
	restarted, err := store.Load(ctx)
	require.NoError(t, err)
	enabled, err = managed.CheckinEnabled(ctx, server.URL, restarted)
	require.NoError(t, err)
	require.False(t, enabled)
	restartGate := redirectors.NewGate()
	restartGate.SetEnabled(false)
	require.NoError(t, managed.WaitForInitialCheckin(ctx, server.URL, restarted, restartGate))
	require.False(t, restartGate.Enabled(), "restart must retain Tavern's disabled state")
	// Toggle after the heartbeat update has completed. The response must read
	// enabled after this point, rather than reuse a pre-update snapshot.
	var toggled atomic.Bool
	graph.Redirector.Use(func(next ent.Mutator) ent.Mutator {
		return ent.MutateFunc(func(ctx context.Context, mutation ent.Mutation) (ent.Value, error) {
			value, err := next.Mutate(ctx, mutation)
			if err != nil {
				return value, err
			}
			if _, heartbeat := mutation.Field("last_seen_at"); heartbeat && toggled.CompareAndSwap(false, true) {
				if err := graph.Redirector.UpdateOneID(records[0].ID).SetEnabled(true).Exec(ctx); err != nil {
					return nil, err
				}
			}
			return value, nil
		})
	})
	enabled, err = managed.CheckinEnabled(ctx, server.URL, first)
	require.NoError(t, err)
	require.True(t, toggled.Load())
	require.True(t, enabled)
	require.NoError(t, graph.Redirector.UpdateOneID(records[0].ID).SetEnabled(true).Exec(ctx))
	enabled, err = managed.CheckinEnabled(ctx, server.URL, second)
	require.NoError(t, err)
	require.True(t, enabled)
	gate := redirectors.NewGate()
	require.NoError(t, graph.Redirector.UpdateOneID(records[0].ID).SetEnabled(false).Exec(ctx))
	require.NoError(t, managed.RefreshGate(ctx, server.URL, first, gate))
	require.False(t, gate.Enabled())
	server.Close()
	require.Error(t, managed.RefreshGate(ctx, server.URL, first, gate))
	require.False(t, gate.Enabled(), "failed check-in must preserve last known state")
	startupCtx, cancel := context.WithTimeout(ctx, 50*time.Millisecond)
	defer cancel()
	err = managed.WaitForInitialCheckin(startupCtx, server.URL, first, gate)
	require.ErrorIs(t, err, context.DeadlineExceeded)
	require.False(t, gate.Enabled(), "restart during outage must not open ingress")
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
