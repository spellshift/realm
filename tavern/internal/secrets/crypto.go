package secrets

import (
	"crypto/ecdh"
	"crypto/rand"
	"fmt"
	"log/slog"

	"golang.org/x/crypto/chacha20poly1305"
)

// GenerateSharedKey derives a shared key from a private key and a public key using ECDH.
func GenerateSharedKey(privKey *ecdh.PrivateKey, pubKey *ecdh.PublicKey) ([]byte, error) {
	return privKey.ECDH(pubKey)
}

// Encrypt encrypts the input data using XChaCha20-Poly1305.
func Encrypt(sharedKey, plaintext []byte) ([]byte, error) {
	aead, err := chacha20poly1305.NewX(sharedKey)
	if err != nil {
		return nil, fmt.Errorf("failed to create new xchacha20poly1305 cipher: %w", err)
	}

	nonce := make([]byte, aead.NonceSize(), aead.NonceSize()+len(plaintext)+aead.Overhead())
	if _, err := rand.Read(nonce); err != nil {
		return nil, fmt.Errorf("failed to generate nonce: %w", err)
	}

	ciphertext := aead.Seal(nil, nonce, plaintext, nil)
	return append(nonce, ciphertext...), nil
}

// Decrypt decrypts the input data using XChaCha20-Poly1305.
func Decrypt(sharedKey, ciphertext []byte) ([]byte, error) {
	aead, err := chacha20poly1305.NewX(sharedKey)
	if err != nil {
		slog.Error(fmt.Sprintf("failed to create xchacha key %v", err))
		return nil, fmt.Errorf("failed to create new xchacha20poly1305 cipher: %w", err)
	}

	if len(ciphertext) < aead.NonceSize() {
		return nil, fmt.Errorf("ciphertext is too short")
	}

	nonce, ciphertext := ciphertext[:aead.NonceSize()], ciphertext[aead.NonceSize():]
	plaintext, err := aead.Open(nil, nonce, ciphertext, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to decrypt ciphertext: %w", err)
	}

	return plaintext, nil
}
