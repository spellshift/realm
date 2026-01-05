package keyservice

import (
	"crypto/ecdh"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/sha512"
	"crypto/x509"
	"fmt"
	"log/slog"
	"os"

	"filippo.io/edwards25519"
	"realm.pub/tavern/internal/secrets"
)

// KeyService manages the server's master ed25519 key and derives keys for different purposes
type KeyService struct {
	ed25519PrivateKey ed25519.PrivateKey
	ed25519PublicKey  ed25519.PublicKey
	x25519PrivateKey  *ecdh.PrivateKey
	x25519PublicKey   *ecdh.PublicKey
}

// NewKeyService creates or loads the server's master ed25519 key and derives the x25519 key
func NewKeyService() (*KeyService, error) {
	secretsManager, err := newSecretsManager()
	if err != nil || secretsManager == nil {
		return nil, fmt.Errorf("failed to configure secret manager: %w", err)
	}

	// Check if we already have a master key
	privateKeyBytes, err := secretsManager.GetValue("tavern_master_ed25519_key")
	var ed25519Priv ed25519.PrivateKey
	var ed25519Pub ed25519.PublicKey

	if err != nil {
		// Generate a new ed25519 master key if it doesn't exist
		pub, priv, err := ed25519.GenerateKey(rand.Reader)
		if err != nil {
			return nil, fmt.Errorf("failed to generate ed25519 keypair: %w", err)
		}

		// Marshal and store the private key
		privateKeyBytes, err := x509.MarshalPKCS8PrivateKey(priv)
		if err != nil {
			return nil, fmt.Errorf("unable to marshal private key: %w", err)
		}

		_, err = secretsManager.SetValue("tavern_master_ed25519_key", privateKeyBytes)
		if err != nil {
			return nil, fmt.Errorf("unable to set 'tavern_master_ed25519_key' using secrets manager: %w", err)
		}

		ed25519Priv = priv
		ed25519Pub = pub

		slog.Info("Generated new ed25519 master keypair")
	} else {
		// Parse existing private key
		tmp, err := x509.ParsePKCS8PrivateKey(privateKeyBytes)
		if err != nil {
			return nil, fmt.Errorf("unable to parse private key: %w", err)
		}

		var ok bool
		ed25519Priv, ok = tmp.(ed25519.PrivateKey)
		if !ok {
			return nil, fmt.Errorf("expected ed25519.PrivateKey, got %T", tmp)
		}

		ed25519Pub = ed25519Priv.Public().(ed25519.PublicKey)

		slog.Info("Loaded existing ed25519 master keypair")
	}

	// Derive x25519 keys from ed25519 keys
	x25519Priv, x25519Pub, err := deriveX25519FromEd25519(ed25519Priv, ed25519Pub)
	if err != nil {
		return nil, fmt.Errorf("failed to derive x25519 keys: %w", err)
	}

	return &KeyService{
		ed25519PrivateKey: ed25519Priv,
		ed25519PublicKey:  ed25519Pub,
		x25519PrivateKey:  x25519Priv,
		x25519PublicKey:   x25519Pub,
	}, nil
}

// GetEd25519PrivateKey returns the ed25519 private key for JWT signing
func (ks *KeyService) GetEd25519PrivateKey() ed25519.PrivateKey {
	return ks.ed25519PrivateKey
}

// GetEd25519PublicKey returns the ed25519 public key for JWT verification
func (ks *KeyService) GetEd25519PublicKey() ed25519.PublicKey {
	return ks.ed25519PublicKey
}

// GetX25519PrivateKey returns the derived x25519 private key for ECDH
func (ks *KeyService) GetX25519PrivateKey() *ecdh.PrivateKey {
	return ks.x25519PrivateKey
}

// GetX25519PublicKey returns the derived x25519 public key for ECDH
func (ks *KeyService) GetX25519PublicKey() *ecdh.PublicKey {
	return ks.x25519PublicKey
}

// deriveX25519FromEd25519 converts ed25519 keys to x25519 keys
// This follows the standard conversion defined in RFC 7748
func deriveX25519FromEd25519(ed25519Priv ed25519.PrivateKey, ed25519Pub ed25519.PublicKey) (*ecdh.PrivateKey, *ecdh.PublicKey, error) {
	// Convert ed25519 private key to x25519 private key
	// The ed25519 private key is 64 bytes: [32-byte seed][32-byte public key]
	// We use the 32-byte seed and hash it to get the x25519 scalar
	seed := ed25519Priv.Seed()

	// Hash the seed to get the x25519 private scalar
	// This follows the ed25519 key generation process
	hash := sha512.Sum512(seed)

	// Clamp the hash as required for Curve25519 scalars
	hash[0] &= 248
	hash[31] &= 127
	hash[31] |= 64

	// Create x25519 private key from the clamped scalar
	curve := ecdh.X25519()
	x25519Priv, err := curve.NewPrivateKey(hash[:32])
	if err != nil {
		return nil, nil, fmt.Errorf("failed to create x25519 private key: %w", err)
	}

	// Convert ed25519 public key to x25519 public key
	// This requires converting the Edwards curve point to Montgomery curve point
	x25519Pub, err := ed25519PublicKeyToX25519(ed25519Pub)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to convert ed25519 public key to x25519: %w", err)
	}

	return x25519Priv, x25519Pub, nil
}

// ed25519PublicKeyToX25519 converts an ed25519 public key to x25519 public key
// This implements the birational map from Edwards to Montgomery curve
func ed25519PublicKeyToX25519(ed25519Pub ed25519.PublicKey) (*ecdh.PublicKey, error) {
	// Parse the ed25519 public key as an Edwards point
	edPoint, err := (&edwards25519.Point{}).SetBytes(ed25519Pub)
	if err != nil {
		return nil, fmt.Errorf("failed to parse ed25519 public key: %w", err)
	}

	// Convert Edwards to Montgomery using the formula: u = (1+y)/(1-y)
	// We'll use the edwards25519 package to get the coordinates
	edPointBytes := edPoint.BytesMontgomery()

	// Create x25519 public key
	curve := ecdh.X25519()
	x25519Pub, err := curve.NewPublicKey(edPointBytes)
	if err != nil {
		return nil, fmt.Errorf("failed to create x25519 public key: %w", err)
	}

	return x25519Pub, nil
}

// newSecretsManager creates a secrets manager instance
func newSecretsManager() (secrets.SecretsManager, error) {
	gcpProjectID := os.Getenv("GCP_PROJECT_ID")
	secretsPath := os.Getenv("SECRETS_FILE_PATH")

	if gcpProjectID == "" && secretsPath == "" {
		slog.Warn("No configuration provided for secret manager path, using a potentially insecure default.")
		return secrets.NewDebugFileSecrets("/tmp/tavern-secrets")
	}

	if secretsPath == "" {
		return secrets.NewGcp(gcpProjectID)
	}

	return secrets.NewDebugFileSecrets(secretsPath)
}
