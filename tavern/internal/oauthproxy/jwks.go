package oauthproxy

import (
	"context"
	"crypto/ed25519"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"sync"
	"time"
)

// jwksKeyID is the single key identifier the proxy currently publishes. A future key-rotation
// scheme could publish multiple entries and have assertions/state carry a "kid", but a single
// always-current key is sufficient for this deployment.
const jwksKeyID = "default"

// jwkKey is a minimal, intentionally non-standard key entry: both producer (oauthproxy) and
// consumer (Tavern) of this document are this same codebase, so a plain base64-encoded raw
// Ed25519 public key is simpler than conforming to RFC 7517's OKP encoding.
type jwkKey struct {
	Kid       string `json:"kid"`
	PublicKey string `json:"public_key"`
}

type jwkSet struct {
	Keys []jwkKey `json:"keys"`
}

// EncodeJWKS renders pubKey as the proxy's published key-set document, served from
// /.well-known/jwks.json.
func EncodeJWKS(pubKey ed25519.PublicKey) ([]byte, error) {
	data, err := json.Marshal(jwkSet{Keys: []jwkKey{{
		Kid:       jwksKeyID,
		PublicKey: base64.StdEncoding.EncodeToString(pubKey),
	}}})
	if err != nil {
		return nil, fmt.Errorf("oauthproxy: failed to encode jwks: %w", err)
	}
	return data, nil
}

// decodeJWKS parses a document produced by EncodeJWKS, returning its first key.
func decodeJWKS(data []byte) (ed25519.PublicKey, error) {
	var set jwkSet
	if err := json.Unmarshal(data, &set); err != nil {
		return nil, fmt.Errorf("oauthproxy: failed to parse jwks: %w", err)
	}
	if len(set.Keys) == 0 {
		return nil, fmt.Errorf("oauthproxy: jwks contained no keys")
	}

	raw, err := base64.StdEncoding.DecodeString(set.Keys[0].PublicKey)
	if err != nil {
		return nil, fmt.Errorf("oauthproxy: failed to decode jwks public key: %w", err)
	}
	if len(raw) != ed25519.PublicKeySize {
		return nil, fmt.Errorf("oauthproxy: invalid jwks public key size: got %d, expected %d", len(raw), ed25519.PublicKeySize)
	}

	return ed25519.PublicKey(raw), nil
}

// KeySource fetches and caches the oauth proxy's current public key from its JWKS endpoint, so a
// tenant Tavern instance can verify proxy-issued assertions without any manual key distribution
// or terraform-managed secret sharing between the two services.
type KeySource struct {
	JWKSURL    string
	HTTPClient *http.Client
	CacheTTL   time.Duration

	mu       sync.Mutex
	cached   ed25519.PublicKey
	cachedAt time.Time
}

// PublicKey returns the proxy's current public key, refreshing from the JWKS endpoint once the
// cached value has expired. A transient fetch failure after a key has already been cached once
// serves the stale key rather than breaking login entirely.
func (ks *KeySource) PublicKey(ctx context.Context) (ed25519.PublicKey, error) {
	ks.mu.Lock()
	defer ks.mu.Unlock()

	ttl := ks.CacheTTL
	if ttl <= 0 {
		ttl = 5 * time.Minute
	}
	if ks.cached != nil && time.Since(ks.cachedAt) < ttl {
		return ks.cached, nil
	}

	client := ks.HTTPClient
	if client == nil {
		client = http.DefaultClient
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, ks.JWKSURL, nil)
	if err != nil {
		return nil, fmt.Errorf("oauthproxy: failed to build jwks request: %w", err)
	}

	resp, err := client.Do(req)
	if err != nil {
		if ks.cached != nil {
			return ks.cached, nil
		}
		return nil, fmt.Errorf("oauthproxy: failed to fetch jwks: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		if ks.cached != nil {
			return ks.cached, nil
		}
		return nil, fmt.Errorf("oauthproxy: jwks endpoint returned status %d", resp.StatusCode)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("oauthproxy: failed to read jwks response: %w", err)
	}

	pubKey, err := decodeJWKS(body)
	if err != nil {
		if ks.cached != nil {
			return ks.cached, nil
		}
		return nil, err
	}

	ks.cached = pubKey
	ks.cachedAt = time.Now()
	return ks.cached, nil
}
