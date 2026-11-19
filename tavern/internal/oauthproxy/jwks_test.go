package oauthproxy

import (
	"context"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
	"testing"
	"time"
)

func TestEncodeDecodeJWKS(t *testing.T) {
	pub, _ := mustKeyPair(t)

	data, err := EncodeJWKS(pub)
	if err != nil {
		t.Fatalf("EncodeJWKS returned error: %v", err)
	}

	got, err := decodeJWKS(data)
	if err != nil {
		t.Fatalf("decodeJWKS returned error: %v", err)
	}
	if !got.Equal(pub) {
		t.Error("decoded public key does not match original")
	}
}

func TestDecodeJWKSRejectsEmpty(t *testing.T) {
	if _, err := decodeJWKS([]byte(`{"keys":[]}`)); err == nil {
		t.Fatal("expected error decoding jwks with no keys, got nil")
	}
}

func TestKeySourcePublicKeyFetchesAndCaches(t *testing.T) {
	pub, _ := mustKeyPair(t)
	data, err := EncodeJWKS(pub)
	if err != nil {
		t.Fatalf("EncodeJWKS returned error: %v", err)
	}

	requests := 0
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		requests++
		w.Header().Set("Content-Type", "application/json")
		w.Write(data)
	}))
	defer srv.Close()

	ks := &KeySource{JWKSURL: srv.URL, CacheTTL: time.Hour}

	got, err := ks.PublicKey(context.Background())
	if err != nil {
		t.Fatalf("PublicKey returned error: %v", err)
	}
	if !got.Equal(pub) {
		t.Error("fetched public key does not match original")
	}

	// Second call within the cache TTL should not hit the server again.
	if _, err := ks.PublicKey(context.Background()); err != nil {
		t.Fatalf("PublicKey (cached) returned error: %v", err)
	}
	if requests != 1 {
		t.Errorf("expected 1 request to jwks endpoint, got %d", requests)
	}
}

func TestKeySourcePublicKeyServesStaleOnFetchError(t *testing.T) {
	pub, _ := mustKeyPair(t)
	data, err := EncodeJWKS(pub)
	if err != nil {
		t.Fatalf("EncodeJWKS returned error: %v", err)
	}

	fail := false
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if fail {
			w.WriteHeader(http.StatusInternalServerError)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		w.Write(data)
	}))
	defer srv.Close()

	ks := &KeySource{JWKSURL: srv.URL, CacheTTL: 0}
	// CacheTTL of 0 defaults internally to 5m, so force an immediate expiry for the test by
	// fetching once, then marking the cache as already expired.
	if _, err := ks.PublicKey(context.Background()); err != nil {
		t.Fatalf("PublicKey returned error: %v", err)
	}
	ks.cachedAt = time.Now().Add(-time.Hour)

	fail = true
	got, err := ks.PublicKey(context.Background())
	if err != nil {
		t.Fatalf("expected stale key to be served without error, got: %v", err)
	}
	if !got.Equal(pub) {
		t.Error("expected stale public key to be returned on fetch failure")
	}
}

func TestKeySourceDoesNotBlockCachedReadsDuringRefresh(t *testing.T) {
	pub, _ := mustKeyPair(t)
	data, err := EncodeJWKS(pub)
	if err != nil {
		t.Fatalf("EncodeJWKS: %v", err)
	}

	release := make(chan struct{})
	var slow atomic.Bool
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if slow.Load() {
			<-release
		}
		w.Write(data)
	}))
	defer srv.Close()

	ks := &KeySource{JWKSURL: srv.URL, CacheTTL: time.Nanosecond}
	if _, err := ks.PublicKey(context.Background()); err != nil {
		t.Fatalf("initial fetch: %v", err)
	}

	// Start a refresh that hangs on the server...
	slow.Store(true)
	done := make(chan struct{})
	go func() {
		ks.PublicKey(context.Background())
		close(done)
	}()
	time.Sleep(50 * time.Millisecond)

	// ...and make sure the lock isn't held while it waits.
	locked := make(chan struct{})
	go func() {
		ks.mu.Lock()
		ks.mu.Unlock()
		close(locked)
	}()
	select {
	case <-locked:
	case <-time.After(time.Second):
		t.Fatal("KeySource mutex held during network fetch")
	}
	close(release)
	<-done
}
