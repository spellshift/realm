package cryptocodec

import (
	"crypto/ecdh"
	"crypto/rand"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/crypto/chacha20poly1305"
)

func TestCryptoSvc_EncryptDecrypt(t *testing.T) {
	// Generate server key pair
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)

	// Generate client key pair
	clientPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	clientPubKeyBytes := clientPrivKey.PublicKey().Bytes()

	svc := NewCryptoSvc(serverPrivKey)

	// Simulate registering the client public key in the session map
	// In the real code, this happens during Decrypt or is pre-populated.
	// For Encrypt to work, it looks up the key based on the goroutine ID.
	// Since we can't easily control the goroutine ID lookup in a test without modifying the code or using a hack,
	// we'll focus on unit testing the logic if possible, or we need to simulate the flow.

	// However, `Encrypt` logic relies on `goAllIds()` which parses `debug.Stack()`.
	// This makes it tightly coupled to the runtime environment.
	// To test `Encrypt`, we must ensure the `session_pub_keys` has an entry for the current goroutine ID.

	ids, err := goAllIds()
	require.NoError(t, err)
	session_pub_keys.Store(ids.Id, clientPubKeyBytes)

	plaintext := []byte("hello world")

	// Test Encryption
	ciphertext := svc.Encrypt(plaintext)
	assert.NotEqual(t, FAILURE_BYTES, ciphertext)
	assert.NotEmpty(t, ciphertext)

	// The ciphertext format is [client_pub_key (32)] + [nonce (24)] + [encrypted_data]
	assert.Greater(t, len(ciphertext), 32+24)

	// Test Decryption
	// Decrypt expects [client_pub_key] + [nonce] + [ciphertext]
	// But wait, the `Decrypt` method also stores the key in the session map.

	// Let's create a valid encrypted message from the client's perspective to test Decrypt properly.
	// Server Public Key
	serverPubKey := serverPrivKey.PublicKey()

	// Client generates shared secret: ECDH(clientPriv, serverPub)
	sharedSecret, err := clientPrivKey.ECDH(serverPubKey)
	require.NoError(t, err)

	aead, err := chacha20poly1305.NewX(sharedSecret)
	require.NoError(t, err)

	nonce := make([]byte, aead.NonceSize())
	_, err = rand.Read(nonce)
	require.NoError(t, err)

	encryptedMsg := aead.Seal(nil, nonce, plaintext, nil)

	// Construct the full message expected by Decrypt: [client_pub_key] + [nonce] + [encryptedMsg]
	fullMsg := append(clientPubKeyBytes, nonce...)
	fullMsg = append(fullMsg, encryptedMsg...)

	decrypted, pubKey := svc.Decrypt(fullMsg)

	assert.Equal(t, plaintext, decrypted)
	assert.Equal(t, clientPubKeyBytes, pubKey)
}

func TestCryptoSvc_Decrypt_InvalidInput(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	svc := NewCryptoSvc(serverPrivKey)

	// Test too short
	res, pub := svc.Decrypt([]byte("short"))
	assert.Equal(t, FAILURE_BYTES, res)
	assert.Equal(t, FAILURE_BYTES, pub)

	// Test just pubkey
	res, pub = svc.Decrypt(make([]byte, 32))
	assert.Equal(t, FAILURE_BYTES, res)
	assert.Equal(t, FAILURE_BYTES, pub)
}

func TestCryptoSvc_Encrypt_NoKeyFound(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	svc := NewCryptoSvc(serverPrivKey)

	// Ensure no key is stored for this goroutine
	_, err = goAllIds()
	require.NoError(t, err)

	// We might need to ensure we don't accidentally pick up a key from a previous test running in the same goroutine?
	// The cache is global. We can't easily clear it for the specific ID without exposing internal methods,
	// but we can generate a new unique key pair that won't be in the cache if we hadn't put it there.
	// Actually, `Encrypt` looks up by goroutine ID. If we haven't stored anything for *this* goroutine ID, it fails.
	// But `TestCryptoSvc_EncryptDecrypt` might have run in the same goroutine.
	// So we might need to rely on the fact that `go test` runs tests in different goroutines or we can spawn one.

	done := make(chan bool)
	go func() {
		res := svc.Encrypt([]byte("data"))
		assert.Equal(t, FAILURE_BYTES, res)
		done <- true
	}()
	<-done
}

func TestSyncMap(t *testing.T) {
	m := NewSyncMap()

	key := 123
	val := []byte("test")

	m.Store(key, val)

	got, ok := m.Load(key)
	assert.True(t, ok)
	assert.Equal(t, val, got)

	_, ok = m.Load(999)
	assert.False(t, ok)

	str := m.String()
	assert.Contains(t, str, "id: 123")
}
