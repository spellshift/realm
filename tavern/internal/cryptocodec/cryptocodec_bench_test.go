package cryptocodec

import (
	"crypto/ecdh"
	"crypto/rand"
	"testing"

	"github.com/stretchr/testify/require"
	"golang.org/x/crypto/chacha20poly1305"
)

func BenchmarkEncrypt(b *testing.B) {
	// Setup keys
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(b, err)

	clientPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(b, err)
	clientPubKey := clientPrivKey.PublicKey().Bytes()

	svc := NewCryptoSvc(serverPrivKey)

	// Register current goroutine ID with client public key
	trace, err := goAllIds()
	require.NoError(b, err)
	session_pub_keys.Store(trace.Id, clientPubKey)

	payload := make([]byte, 1024)
	_, err = rand.Read(payload)
	require.NoError(b, err)

	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(int64(len(payload)))

	for i := 0; i < b.N; i++ {
		encrypted := svc.Encrypt(payload)
		if len(encrypted) == 0 {
			b.Fatal("Encrypt returned empty slice")
		}
	}
}

func BenchmarkDecrypt(b *testing.B) {
	// Setup keys
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(b, err)

	clientPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(b, err)
	clientPubKey := clientPrivKey.PublicKey().Bytes()

	// Compute shared secret manually to encrypt
	sharedSecret, err := clientPrivKey.ECDH(serverPrivKey.PublicKey())
	require.NoError(b, err)

	// Create AEAD
	aead, err := chacha20poly1305.NewX(sharedSecret)
	require.NoError(b, err)

	plaintext := make([]byte, 1024)
	_, err = rand.Read(plaintext)
	require.NoError(b, err)

	nonce := make([]byte, aead.NonceSize())
	_, err = rand.Read(nonce)
	require.NoError(b, err)

	ciphertext := aead.Seal(nil, nonce, plaintext, nil)

	// Construct payload: ClientPubKey + Nonce + Ciphertext
	payload := append(clientPubKey, nonce...)
	payload = append(payload, ciphertext...)

	svc := NewCryptoSvc(serverPrivKey)

	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(int64(len(plaintext)))

	for i := 0; i < b.N; i++ {
		decrypted, _ := svc.Decrypt(payload)
		if len(decrypted) == 0 {
			b.Fatal("Decrypt returned empty slice")
		}
	}
}
