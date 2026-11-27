package oauthproxy

import (
	"context"
	"encoding/base64"
	"encoding/json"
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

	// Verify standard RFC 7517 / RFC 8037 OKP fields.
	var doc struct {
		Keys []map[string]any `json:"keys"`
	}
	if err := json.Unmarshal(data, &doc); err != nil {
		t.Fatalf("failed to unmarshal jwks json: %v", err)
	}
	if len(doc.Keys) != 1 {
		t.Fatalf("expected 1 jwk, got %d", len(doc.Keys))
	}
	k := doc.Keys[0]
	if k["kty"] != "OKP" {
		t.Errorf("kty = %v, want OKP", k["kty"])
	}
	if k["crv"] != "Ed25519" {
		t.Errorf("crv = %v, want Ed25519", k["crv"])
	}
	if k["kid"] != jwksKeyID {
		t.Errorf("kid = %v, want %q", k["kid"], jwksKeyID)
	}
	if k["use"] != "sig" {
		t.Errorf("use = %v, want sig", k["use"])
	}
	if k["alg"] != "EdDSA" {
		t.Errorf("alg = %v, want EdDSA", k["alg"])
	}
	if k["x"] != base64.RawURLEncoding.EncodeToString(pub) {
		t.Errorf("x = %v, want %q", k["x"], base64.RawURLEncoding.EncodeToString(pub))
	}

	got, err := decodeJWKS(data)
	if err != nil {
		t.Fatalf("decodeJWKS returned error: %v", err)
	}
	if !got.Equal(pub) {
		t.Error("decoded public key does not match original")
	}
}

func TestDecodeJWKSRejectsInvalid(t *testing.T) {
	pub, _ := mustKeyPair(t)
	validX := base64.RawURLEncoding.EncodeToString(pub)

	cases := []struct {
		name string
		raw  string
	}{
		{"empty keys", `{"keys":[]}`},
		{"wrong kty", `{"keys":[{"kty":"RSA","crv":"Ed25519","kid":"default","x":"` + validX + `"}]}`},
		{"wrong crv", `{"keys":[{"kty":"OKP","crv":"Ed448","kid":"default","x":"` + validX + `"}]}`},
		{"wrong use", `{"keys":[{"kty":"OKP","crv":"Ed25519","kid":"default","use":"enc","x":"` + validX + `"}]}`},
		{"wrong alg", `{"keys":[{"kty":"OKP","crv":"Ed25519","kid":"default","alg":"RS256","x":"` + validX + `"}]}`},
		{"short x", `{"keys":[{"kty":"OKP","crv":"Ed25519","kid":"default","x":"AAAA"}]}`},
		{"invalid base64url", `{"keys":[{"kty":"OKP","crv":"Ed25519","kid":"default","x":"***"}]}`},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			if _, err := decodeJWKS([]byte(tc.raw)); err == nil {
				t.Fatalf("expected error decoding invalid jwks (%s), got nil", tc.name)
			}
		})
	}
}

func TestKeySourceRejectsRedirect(t *testing.T) {
	pub, _ := mustKeyPair(t)
	data, err := EncodeJWKS(pub)
	if err != nil {
		t.Fatalf("EncodeJWKS: %v", err)
	}

	target := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write(data)
	}))
	defer target.Close()

	redirector := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Redirect(w, r, target.URL, http.StatusFound)
	}))
	defer redirector.Close()

	ks := &KeySource{JWKSURL: redirector.URL}
	if _, err := ks.PublicKey(context.Background()); err == nil {
		t.Fatal("expected error when jwks endpoint redirects, got nil")
	}
}

func TestKeySourcePublicKeyRejectsExpiredStaleOnFetchError(t *testing.T) {
	pub, _ := mustKeyPair(t)
	data, err := EncodeJWKS(pub)
	if err != nil {
		t.Fatalf("EncodeJWKS: %v", err)
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

	ks := &KeySource{
		JWKSURL:  srv.URL,
		CacheTTL: 5 * time.Minute,
		MaxStale: 30 * time.Minute,
	}
	if _, err := ks.PublicKey(context.Background()); err != nil {
		t.Fatalf("initial PublicKey returned error: %v", err)
	}
	ks.cachedAt = time.Now().Add(-time.Hour)
	fail = true

	if _, err := ks.PublicKey(context.Background()); err == nil {
		t.Fatal("expected error when cached key exceeds MaxStale and refresh fails, got nil")
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
