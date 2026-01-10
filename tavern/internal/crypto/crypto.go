package crypto

import (
	"crypto/ecdh"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/sha512"
	"fmt"
	"log/slog"

	"filippo.io/edwards25519"
	"realm.pub/tavern/internal/secrets"
)

// https://github.com/openssl/openssl/discussions/23905
// ed25519ToX25519PrivateKey converts an ED25519 private key to X25519
func ed25519ToX25519PrivateKey(edPrivKey ed25519.PrivateKey) ([]byte, error) {
	// ED25519 private key is 64 bytes: [32-byte seed || 32-byte public key]
	// The seed is the first 32 bytes
	seed := edPrivKey.Seed()

	// Hash the seed to get the scalar
	h := sha512.Sum512(seed)
	// Clamp the scalar as per Curve25519 spec
	h[0] &= 248
	h[31] &= 127
	h[31] |= 64

	// The first 32 bytes of the hash is the X25519 private key
	x25519PrivKey := h[:32]
	return x25519PrivKey, nil
}

// ed25519ToX25519PublicKey converts an ED25519 public key to X25519
func ed25519ToX25519PublicKey(edPubKey ed25519.PublicKey) ([]byte, error) {
	// Convert the ED25519 public key point to Montgomery form (X25519)
	point, err := (&edwards25519.Point{}).SetBytes(edPubKey)
	if err != nil {
		return nil, fmt.Errorf("failed to parse ED25519 public key: %w", err)
	}

	// Convert to Montgomery form for X25519
	x25519PubKey := point.BytesMontgomery()
	return x25519PubKey, nil
}

// generateKeyPair generates an ED25519 key pair
func generateKeyPair() (ed25519.PublicKey, ed25519.PrivateKey, error) {
	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		slog.Error(fmt.Sprintf("failed to generate ED25519 key pair: %v\n", err))
		return nil, nil, err
	}
	return pubKey, privKey, nil
}

// getKeyPair retrieves or generates the ED25519 key pair from the secrets manager
func getKeyPair(secretsManager secrets.SecretsManager) (ed25519.PublicKey, ed25519.PrivateKey, error) {
	if secretsManager == nil {
		return nil, nil, fmt.Errorf("secrets manager is nil")
	}

	// Check if we already have a key stored (now using ED25519)
	privateKeyBytes, err := secretsManager.GetValue("tavern_ed25519_private_key")
	if err != nil {
		// Generate a new ED25519 key pair if it doesn't exist
		pubKey, privKey, err := generateKeyPair()
		if err != nil {
			return nil, nil, fmt.Errorf("key generation failed: %v", err)
		}

		// Store the ED25519 private key (64 bytes: seed + public key)
		_, err = secretsManager.SetValue("tavern_ed25519_private_key", privKey)
		if err != nil {
			return nil, nil, fmt.Errorf("unable to set 'tavern_ed25519_private_key' using secrets manager: %v", err)
		}
		return pubKey, privKey, nil
	}

	// ED25519 private key is 64 bytes
	if len(privateKeyBytes) != ed25519.PrivateKeySize {
		return nil, nil, fmt.Errorf("invalid ED25519 private key size: got %d, expected %d", len(privateKeyBytes), ed25519.PrivateKeySize)
	}

	privateKey := ed25519.PrivateKey(privateKeyBytes)
	publicKey := privateKey.Public().(ed25519.PublicKey)

	return publicKey, privateKey, nil
}

// GetPubKeyED25519 returns the server's ED25519 public key
func GetPubKeyED25519(secretsManager secrets.SecretsManager) (ed25519.PublicKey, error) {
	pub, _, err := getKeyPair(secretsManager)
	if err != nil {
		return nil, err
	}
	return pub, nil
}

// GetPrivKeyED25519 returns the server's ED25519 private key
func GetPrivKeyED25519(secretsManager secrets.SecretsManager) (ed25519.PrivateKey, error) {
	_, priv, err := getKeyPair(secretsManager)
	if err != nil {
		return nil, err
	}
	return priv, nil
}

// GetPubKeyX25519 returns the server's X25519 public key (derived from ED25519)
func GetPubKeyX25519(secretsManager secrets.SecretsManager) (*ecdh.PublicKey, error) {
	edPub, _, err := getKeyPair(secretsManager)
	if err != nil {
		return nil, err
	}

	x25519PubBytes, err := ed25519ToX25519PublicKey(edPub)
	if err != nil {
		return nil, fmt.Errorf("failed to convert ED25519 to X25519 public key: %w", err)
	}

	curve := ecdh.X25519()
	x25519Pub, err := curve.NewPublicKey(x25519PubBytes)
	if err != nil {
		return nil, fmt.Errorf("failed to create X25519 public key: %w", err)
	}

	return x25519Pub, nil
}

// GetPrivKeyX25519 returns the server's X25519 private key (derived from ED25519)
func GetPrivKeyX25519(secretsManager secrets.SecretsManager) (*ecdh.PrivateKey, error) {
	_, edPriv, err := getKeyPair(secretsManager)
	if err != nil {
		return nil, err
	}

	x25519PrivBytes, err := ed25519ToX25519PrivateKey(edPriv)
	if err != nil {
		return nil, fmt.Errorf("failed to convert ED25519 to X25519 private key: %w", err)
	}

	curve := ecdh.X25519()
	x25519Priv, err := curve.NewPrivateKey(x25519PrivBytes)
	if err != nil {
		return nil, fmt.Errorf("failed to create X25519 private key: %w", err)
	}

	return x25519Priv, nil
}
