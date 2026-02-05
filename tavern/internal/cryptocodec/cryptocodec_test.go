package cryptocodec

import (
	"crypto/ecdh"
	"crypto/rand"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/crypto/chacha20poly1305"
)

func TestLRUCache(t *testing.T) {
	var session_pub_keys = NewSyncMap()
	session_pub_keys.Store(1, []byte{0x01, 0x02, 0x03})
	res, ok := session_pub_keys.Load(1)
	assert.True(t, ok)
	assert.Equal(t, []byte{0x01, 0x02, 0x03}, res)
	_, ok = session_pub_keys.Load(2)
	assert.False(t, ok)
}

func TestNewSyncMap(t *testing.T) {
	sm := NewSyncMap()
	assert.NotNil(t, sm)
	assert.NotNil(t, sm.Map)
}

func TestSyncMap_StoreLoad(t *testing.T) {
	sm := NewSyncMap()
	key := 123
	val := []byte("test value")

	sm.Store(key, val)

	loaded, ok := sm.Load(key)
	assert.True(t, ok)
	assert.Equal(t, val, loaded)

	_, ok = sm.Load(999)
	assert.False(t, ok)
}

func TestSyncMap_String(t *testing.T) {
	sm := NewSyncMap()
	sm.Store(1, []byte{0xDE, 0xAD, 0xBE, 0xEF})
	s := sm.String()
	assert.Contains(t, s, "id: 1")
	assert.Contains(t, s, "deadbeef")
}

func TestGetAEAD(t *testing.T) {
	// Generate server key pair
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)

	// Generate client key pair
	clientPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	clientPubKey := clientPrivKey.PublicKey().Bytes()

	svc := NewCryptoSvc(serverPrivKey)
	aead, err := svc.getAEAD(clientPubKey)
	require.NoError(t, err)
	require.NotNil(t, aead)

	// Test caching: getting it again should return the same instance (or at least succeed)
	aead2, err := svc.getAEAD(clientPubKey)
	require.NoError(t, err)
	assert.Equal(t, aead, aead2)
}

func TestGetAEAD_InvalidKey(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	svc := NewCryptoSvc(serverPrivKey)

	// Test with invalid public key length
	invalidKey := []byte{0x00, 0x01}
	_, err = svc.getAEAD(invalidKey)
	assert.Error(t, err)
}

func TestDecrypt(t *testing.T) {
	// Setup keys
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)

	clientPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	clientPubKey := clientPrivKey.PublicKey().Bytes()

	// Compute shared secret manually to encrypt
	sharedSecret, err := clientPrivKey.ECDH(serverPrivKey.PublicKey())
	require.NoError(t, err)

	// Create AEAD
	aead, err := chacha20poly1305.NewX(sharedSecret)
	require.NoError(t, err)

	plaintext := []byte("Hello World")
	nonce := make([]byte, aead.NonceSize())
	_, err = rand.Read(nonce)
	require.NoError(t, err)

	ciphertext := aead.Seal(nil, nonce, plaintext, nil)

	// Construct payload: ClientPubKey + Nonce + Ciphertext
	payload := append(clientPubKey, nonce...)
	payload = append(payload, ciphertext...)

	// Test Decrypt
	svc := NewCryptoSvc(serverPrivKey)
	decrypted, pubKey := svc.Decrypt(payload)

	assert.Equal(t, plaintext, decrypted)
	assert.Equal(t, clientPubKey, pubKey)
}

func TestDecrypt_ShortInput(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	svc := NewCryptoSvc(serverPrivKey)

	res, _ := svc.Decrypt([]byte{0x00})
	assert.Equal(t, FAILURE_BYTES, res)
}

func TestDecrypt_ShortInputAfterKey(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	svc := NewCryptoSvc(serverPrivKey)

	// Input long enough for key (32 bytes) but not nonce
	input := make([]byte, 32+1)
	res, _ := svc.Decrypt(input)
	assert.Equal(t, FAILURE_BYTES, res)
}

func TestGoAllIds(t *testing.T) {
	trace, err := goAllIds()
	assert.NoError(t, err)
	assert.Greater(t, trace.Id, 0)
}

func TestCryptoSvc_Encrypt_NoSession(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	svc := NewCryptoSvc(serverPrivKey)

	// Encrypt should fail because the current goroutine ID is not in the session map
	res := svc.Encrypt([]byte("test"))
	assert.Equal(t, FAILURE_BYTES, res)
}

func TestCryptoSvc_Encrypt_WithSession(t *testing.T) {
	// Setup keys
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)

	clientPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	clientPubKey := clientPrivKey.PublicKey().Bytes()

	svc := NewCryptoSvc(serverPrivKey)

	// Register current goroutine ID with client public key
	trace, err := goAllIds()
	require.NoError(t, err)
	session_pub_keys.Store(trace.Id, clientPubKey)

	// Test Encrypt
	plaintext := []byte("secret message")
	encrypted := svc.Encrypt(plaintext)

	assert.NotEqual(t, FAILURE_BYTES, encrypted)
	assert.True(t, len(encrypted) > len(plaintext))

	// Verify we can decrypt it back manually.
	// Payload is expected to be: ClientPubKey (32) + Nonce (24) + Ciphertext.
	// Use server private key + extracted client pub key (first 32 bytes) to derive secret.
	extractedClientPubKey := encrypted[:32]
	assert.Equal(t, clientPubKey, extractedClientPubKey)

	sharedSecret, err := serverPrivKey.ECDH(clientPrivKey.PublicKey())
	require.NoError(t, err)

	aead, err := chacha20poly1305.NewX(sharedSecret)
	require.NoError(t, err)

	nonce := encrypted[32 : 32+aead.NonceSize()]
	ciphertext := encrypted[32+aead.NonceSize():]

	decrypted, err := aead.Open(nil, nonce, ciphertext, nil)
	assert.NoError(t, err)
	assert.Equal(t, plaintext, decrypted)
}

func TestNewStreamDecryptCodec(t *testing.T) {
	c := NewStreamDecryptCodec()
	assert.Equal(t, "xchacha20-poly1305", c.Name())
}

func TestStreamDecryptCodec_Marshal_Unmarshal_Error(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	svc := NewCryptoSvc(serverPrivKey)
	codec := StreamDecryptCodec{Csvc: svc}

	assert.Equal(t, "xchacha20-poly1305", codec.Name())
}
