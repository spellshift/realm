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

	"realm.pub/tavern/internal/logging"
)

var jwksLog = logging.For("oauthproxy.jwks")

// jwksKeyID is the single key identifier the proxy currently publishes. A future key-rotation
// scheme could publish multiple entries and have assertions/state carry a "kid", but a single
// always-current key is sufficient for this deployment.
const jwksKeyID = "default"

// jwkKey is an RFC 7517 / RFC 8037 Octet Key Pair (OKP) JWK entry for an Ed25519 public key.
type jwkKey struct {
	Kty string `json:"kty"`
	Crv string `json:"crv"`
	Kid string `json:"kid"`
	Use string `json:"use,omitempty"`
	Alg string `json:"alg,omitempty"`
	X   string `json:"x"`
}

type jwkSet struct {
	Keys []jwkKey `json:"keys"`
}

// EncodeJWKS renders pubKey as a standard RFC 7517 / RFC 8037 OKP JWKS document, served from
// /.well-known/jwks.json.
func EncodeJWKS(pubKey ed25519.PublicKey) ([]byte, error) {
	if len(pubKey) != ed25519.PublicKeySize {
		return nil, fmt.Errorf("oauthproxy: invalid public key size: got %d, expected %d", len(pubKey), ed25519.PublicKeySize)
	}

	data, err := json.Marshal(jwkSet{Keys: []jwkKey{{
		Kty: "OKP",
		Crv: "Ed25519",
		Kid: jwksKeyID,
		Use: "sig",
		Alg: "EdDSA",
		X:   base64.RawURLEncoding.EncodeToString(pubKey),
	}}})
	if err != nil {
		return nil, fmt.Errorf("oauthproxy: failed to encode jwks: %w", err)
	}
	return data, nil
}

// decodeJWKS parses an RFC 7517 / RFC 8037 OKP JWKS document produced by EncodeJWKS, returning
// its first Ed25519 public key.
func decodeJWKS(data []byte) (ed25519.PublicKey, error) {
	var set jwkSet
	if err := json.Unmarshal(data, &set); err != nil {
		return nil, fmt.Errorf("oauthproxy: failed to parse jwks: %w", err)
	}
	if len(set.Keys) == 0 {
		return nil, fmt.Errorf("oauthproxy: jwks contained no keys")
	}

	k := set.Keys[0]
	if k.Kty != "OKP" || k.Crv != "Ed25519" {
		return nil, fmt.Errorf("oauthproxy: unsupported jwks key type %q/%q (expected OKP/Ed25519)", k.Kty, k.Crv)
	}
	if k.Use != "" && k.Use != "sig" {
		return nil, fmt.Errorf("oauthproxy: unsupported jwks key use %q", k.Use)
	}
	if k.Alg != "" && k.Alg != "EdDSA" {
		return nil, fmt.Errorf("oauthproxy: unsupported jwks key alg %q", k.Alg)
	}

	raw, err := base64.RawURLEncoding.DecodeString(k.X)
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
	MaxStale   time.Duration

	mu       sync.Mutex
	cached   ed25519.PublicKey
	cachedAt time.Time
}

const (
	defaultCacheTTL = 5 * time.Minute
	defaultMaxStale = 24 * time.Hour
)

// defaultHTTPClient is used when KeySource.HTTPClient is nil. Unlike http.DefaultClient it has a
// timeout (so an unresponsive proxy can't hang logins indefinitely) and refuses HTTP redirects
// (so a redirect from JWKSURL cannot downgrade to plaintext HTTP or target an internal address).
var defaultHTTPClient = &http.Client{
	Timeout: 10 * time.Second,
	CheckRedirect: func(*http.Request, []*http.Request) error {
		return fmt.Errorf("oauthproxy: redirects are not allowed when fetching jwks")
	},
}

// maxJWKSBytes bounds how much of a JWKS response is read.
const maxJWKSBytes = 64 << 10

// PublicKey returns the proxy's current public key, refreshing from the JWKS endpoint once the
// cached value has expired. A transient fetch failure after a key has already been cached once
// serves the stale key (up to MaxStale, defaulting to 24h) and logs a warning rather than
// breaking login immediately. The network fetch happens without holding the lock, so a slow
// proxy never blocks callers that can be served from the cache.
func (ks *KeySource) PublicKey(ctx context.Context) (ed25519.PublicKey, error) {
	ttl := ks.CacheTTL
	if ttl <= 0 {
		ttl = defaultCacheTTL
	}
	maxStale := ks.MaxStale
	if maxStale <= 0 {
		maxStale = defaultMaxStale
	}
	if maxStale < ttl {
		maxStale = ttl
	}

	ks.mu.Lock()
	stale := ks.cached
	cachedAt := ks.cachedAt
	if stale != nil && time.Since(cachedAt) < ttl {
		ks.mu.Unlock()
		return stale, nil
	}
	ks.mu.Unlock()

	pubKey, err := ks.fetch(ctx)
	if err != nil {
		if stale != nil && time.Since(cachedAt) < maxStale {
			jwksLog.WarnContext(ctx, "failed to refresh jwks; serving cached public key",
				"jwks_url", ks.JWKSURL,
				"cache_age", time.Since(cachedAt).Round(time.Second).String(),
				"error", err,
			)
			return stale, nil
		}
		return nil, err
	}

	ks.mu.Lock()
	ks.cached = pubKey
	ks.cachedAt = time.Now()
	ks.mu.Unlock()
	return pubKey, nil
}

// fetch retrieves and decodes the proxy's public key from its JWKS endpoint.
func (ks *KeySource) fetch(ctx context.Context) (ed25519.PublicKey, error) {
	client := ks.HTTPClient
	if client == nil {
		client = defaultHTTPClient
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, ks.JWKSURL, nil)
	if err != nil {
		return nil, fmt.Errorf("oauthproxy: failed to build jwks request: %w", err)
	}

	resp, err := client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("oauthproxy: failed to fetch jwks: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("oauthproxy: jwks endpoint returned status %d", resp.StatusCode)
	}

	body, err := io.ReadAll(io.LimitReader(resp.Body, maxJWKSBytes))
	if err != nil {
		return nil, fmt.Errorf("oauthproxy: failed to read jwks response: %w", err)
	}

	return decodeJWKS(body)
}
