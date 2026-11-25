package managed

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"encoding/json"
	"encoding/pem"
	"fmt"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder"
)

func TestFileStoreDoesNotOverwriteIdentity(t *testing.T) {
	ctx := context.Background()
	store := fileStore{path: filepath.Join(t.TempDir(), "identity.json")}
	identity := &Identity{ID: "first"}
	require.NoError(t, store.Save(ctx, identity))
	require.ErrorIs(t, store.Save(ctx, &Identity{ID: "second"}), os.ErrExist)
	loaded, err := store.Load(ctx)
	require.NoError(t, err)
	require.Equal(t, "first", loaded.ID)
	require.NoError(t, os.Chmod(store.path, 0644))
	_, err = store.Load(ctx)
	require.ErrorContains(t, err, "permissions are too broad")
}

type barrierStore struct {
	Store
	ready   chan struct{}
	release chan struct{}
}

func (s barrierStore) Save(ctx context.Context, identity *Identity) error {
	s.ready <- struct{}{}
	<-s.release
	return s.Store.Save(ctx, identity)
}

func TestConcurrentFileEnrollmentLoadsPublishedWinner(t *testing.T) {
	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	ca, err := builder.CreateCA(caKey)
	require.NoError(t, err)
	var enrollments atomic.Int32
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var request struct {
			CSR string `json:"csr"`
		}
		if err := json.NewDecoder(r.Body).Decode(&request); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		id := fmt.Sprintf("race-%d", enrollments.Add(1))
		cert, err := builder.SignRedirectorCertificate(ca, caKey, id, request.CSR)
		if err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]string{"redirector_id": id, "mtls_cert": string(cert)})
	}))
	defer server.Close()
	path := filepath.Join(t.TempDir(), "shared.json")
	ready := make(chan struct{}, 2)
	release := make(chan struct{})
	store := barrierStore{Store: fileStore{path: path}, ready: ready, release: release}
	cfg := Config{Upstream: server.URL, Transport: "http1", Domains: []string{"c2.example.com"}, Token: "test-token"}
	results := make(chan *Identity, 2)
	errors := make(chan error, 2)
	var workers sync.WaitGroup
	for i := 0; i < 2; i++ {
		workers.Add(1)
		go func() {
			defer workers.Done()
			identity, err := EnsureIdentity(context.Background(), cfg, store)
			results <- identity
			errors <- err
		}()
	}
	for i := 0; i < 2; i++ {
		select {
		case <-ready:
		case <-time.After(5 * time.Second):
			t.Fatal("both enrollments did not reach Save")
		}
	}
	close(release)
	workers.Wait()
	first, second := <-results, <-results
	require.NoError(t, <-errors)
	require.NoError(t, <-errors)
	require.NotNil(t, first)
	require.Equal(t, first.ID, second.ID)
	loaded, err := (fileStore{path: path}).Load(context.Background())
	require.NoError(t, err)
	require.Equal(t, loaded.ID, first.ID)
	require.EqualValues(t, 2, enrollments.Load())
}

func TestManagementURL(t *testing.T) {
	for _, tc := range []struct {
		explicit, upstream, want string
		valid                    bool
	}{
		{"", "https://tavern.example.com/", "https://tavern.example.com", true},
		{"https://tavern.example.com", "tavern.example.com:443", "https://tavern.example.com", true},
		{"", "tavern.example.com:443", "", false},
		{"", "localhost:8000", "", false},
		{"ftp://tavern.example.com", "localhost:8000", "", false},
		{"https://tavern.example.com/path", "localhost:8000", "", false},
	} {
		got, err := ManagementURL(tc.explicit, tc.upstream)
		if tc.valid {
			require.NoError(t, err)
			require.Equal(t, tc.want, got)
		} else {
			require.ErrorContains(t, err, "--management-url")
		}
	}
}

type publishedStore struct {
	mu       sync.Mutex
	identity *Identity
}

func (s *publishedStore) Load(context.Context) (*Identity, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.identity == nil {
		return nil, os.ErrNotExist
	}
	return s.identity, nil
}

func (s *publishedStore) Save(context.Context, *Identity) error { return fmt.Errorf("unexpected save") }

func TestRejectedTokenWaitsForReplicaIdentityWithoutRetryingToken(t *testing.T) {
	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	ca, err := builder.CreateCA(caKey)
	require.NoError(t, err)
	_, key, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	csrDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, key)
	require.NoError(t, err)
	cert, err := builder.SignRedirectorCertificate(ca, caKey, "shared", string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrDER})))
	require.NoError(t, err)
	keyDER, err := x509.MarshalPKCS8PrivateKey(key)
	require.NoError(t, err)
	winner := &Identity{ID: "shared", Certificate: string(cert), PrivateKey: string(pem.EncodeToMemory(&pem.Block{Type: "PRIVATE KEY", Bytes: keyDER}))}
	var requests atomic.Int32
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { requests.Add(1); w.WriteHeader(http.StatusUnauthorized) }))
	defer server.Close()
	store := &publishedStore{}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	result := make(chan *Identity, 1)
	failures := make(chan error, 1)
	go func() {
		identity, err := EnsureIdentity(ctx, Config{Upstream: server.URL, Token: "used", Transport: "http1", Domains: []string{"c2.example.com"}}, store)
		result <- identity
		failures <- err
	}()
	deadline := time.After(2 * time.Second)
	for requests.Load() == 0 {
		select {
		case <-deadline:
			t.Fatal("enrollment was not attempted")
		case <-time.After(time.Millisecond):
		}
	}
	store.mu.Lock()
	store.identity = winner
	store.mu.Unlock()
	require.NoError(t, <-failures)
	require.Equal(t, winner.ID, (<-result).ID)
	require.EqualValues(t, 1, requests.Load())
}

func TestRejectedTokenWithoutPublishedIdentityTimesOut(t *testing.T) {
	store := &publishedStore{}
	start := time.Now()
	identity, err := waitForIdentity(context.Background(), store, 20*time.Millisecond)
	require.Nil(t, identity)
	require.ErrorIs(t, err, context.DeadlineExceeded)
	require.ErrorContains(t, err, "shared identity did not appear")
	require.Less(t, time.Since(start), time.Second)
}
