package cryptocodec

import (
	"crypto/ecdh"
	"crypto/rand"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/crypto/chacha20poly1305"
	_ "google.golang.org/grpc/encoding/proto"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

func encryptFrameForTest(t *testing.T, serverPubKey *ecdh.PublicKey, clientPrivKey *ecdh.PrivateKey, plaintext []byte) []byte {
	t.Helper()
	sharedSecret, err := clientPrivKey.ECDH(serverPubKey)
	require.NoError(t, err)

	aead, err := chacha20poly1305.NewX(sharedSecret)
	require.NoError(t, err)

	nonce := make([]byte, aead.NonceSize())
	_, err = rand.Read(nonce)
	require.NoError(t, err)

	ciphertext := aead.Seal(nil, nonce, plaintext, nil)
	clientPubKey := clientPrivKey.PublicKey().Bytes()

	payload := make([]byte, 0, len(clientPubKey)+len(nonce)+len(ciphertext))
	payload = append(payload, clientPubKey...)
	payload = append(payload, nonce...)
	payload = append(payload, ciphertext...)
	return payload
}

func TestLRUCache(t *testing.T) {
	var localMap = NewSyncMap()
	localMap.Store(1, []byte{0x01, 0x02, 0x03})
	res, ok := localMap.Load(1)
	assert.True(t, ok)
	assert.Equal(t, []byte{0x01, 0x02, 0x03}, res)
	_, ok = localMap.Load(2)
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

func TestGenerateSharedKey(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)

	clientPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	clientPubKey := clientPrivKey.PublicKey().Bytes()

	svc := NewCryptoSvc(serverPrivKey)
	sharedKey, err := svc.generate_shared_key(clientPubKey)
	require.NoError(t, err)
	assert.Len(t, sharedKey, 32)
}

func TestGenerateSharedKey_InvalidKey(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	svc := NewCryptoSvc(serverPrivKey)

	invalidKey := []byte{0x00, 0x01}
	sharedKey, err := svc.generate_shared_key(invalidKey)
	assert.Error(t, err)
	assert.Nil(t, sharedKey)
}

func TestGenerateSharedKey_NilPrivKey(t *testing.T) {
	svc := CryptoSvc{}
	sharedKey, err := svc.generate_shared_key(make([]byte, 32))
	assert.Error(t, err)
	assert.Nil(t, sharedKey)
}

func TestDecrypt(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)

	clientPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	clientPubKey := clientPrivKey.PublicKey().Bytes()

	plaintext := []byte("Hello World")
	payload := encryptFrameForTest(t, serverPrivKey.PublicKey(), clientPrivKey, plaintext)

	svc := NewCryptoSvc(serverPrivKey)
	trace, err := goAllIds()
	require.NoError(t, err)
	t.Cleanup(func() { session_pub_keys.Remove(trace.Id) })

	decrypted, pubKey, err := svc.Decrypt(payload)
	require.NoError(t, err)
	assert.Equal(t, plaintext, decrypted)
	assert.Equal(t, clientPubKey, pubKey)
}

func TestDecrypt_RejectsEmptyAndShortInputs(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	svc := NewCryptoSvc(serverPrivKey)

	tests := []struct {
		name  string
		input []byte
	}{
		{name: "empty", input: []byte{}},
		{name: "shorter than pubkey", input: []byte{0x00}},
		{name: "shorter than nonce", input: make([]byte, 32+1)},
		{name: "shorter than nonce+tag", input: make([]byte, 32+24+15)},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			res, pk, err := svc.Decrypt(tc.input)
			assert.Error(t, err)
			assert.Nil(t, res)
			assert.Nil(t, pk)
		})
	}
}

func TestDecrypt_UnauthenticatedDoesNotPoisonSessionCache(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)

	legitClientPriv, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	legitClientPub := legitClientPriv.PublicKey().Bytes()

	attackerClientPriv, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)

	svc := NewCryptoSvc(serverPrivKey)
	trace, err := goAllIds()
	require.NoError(t, err)
	t.Cleanup(func() { session_pub_keys.Remove(trace.Id) })

	// Establish a valid session on this goroutine.
	validFrame := encryptFrameForTest(t, serverPrivKey.PublicKey(), legitClientPriv, []byte("legit"))
	_, _, err = svc.Decrypt(validFrame)
	require.NoError(t, err)

	// Send a tampered frame claiming to be from a different client public key.
	tamperedFrame := encryptFrameForTest(t, serverPrivKey.PublicKey(), attackerClientPriv, []byte("attack"))
	tamperedFrame[len(tamperedFrame)-1] ^= 0xff

	_, _, err = svc.Decrypt(tamperedFrame)
	require.Error(t, err)

	// Session cache must still hold the legitimate client's public key.
	cachedPub, ok := session_pub_keys.Load(trace.Id)
	require.True(t, ok)
	assert.Equal(t, legitClientPub, cachedPub)
}

func TestGoAllIds(t *testing.T) {
	trace, err := goAllIds()
	assert.NoError(t, err)
	assert.Greater(t, trace.Id, 0)
}

func TestParseGoroutineIDs(t *testing.T) {
	t.Run("no goroutines", func(t *testing.T) {
		_, err := parseGoroutineIDs([]byte("not a valid stack trace"))
		assert.Error(t, err)
	})

	t.Run("single goroutine without parent does not panic", func(t *testing.T) {
		stack := []byte("goroutine 1 [running]:\nmain.main()\n\t/app/main.go:10 +0x20\n")
		trace, err := parseGoroutineIDs(stack)
		require.NoError(t, err)
		assert.Equal(t, 1, trace.Id)
		assert.Equal(t, 0, trace.ParentId)
		assert.Empty(t, trace.Others)
	})

	t.Run("two goroutines", func(t *testing.T) {
		stack := []byte("goroutine 42 [running]:\ncreated by foo in goroutine 7\n")
		trace, err := parseGoroutineIDs(stack)
		require.NoError(t, err)
		assert.Equal(t, 42, trace.Id)
		assert.Equal(t, 7, trace.ParentId)
		assert.Empty(t, trace.Others)
	})

	t.Run("three or more goroutines and non-numeric token", func(t *testing.T) {
		stack := []byte("goroutine 42 [running]:\n// goroutine note\ncreated by foo in goroutine 7\ncreated by bar in goroutine 3\n")
		trace, err := parseGoroutineIDs(stack)
		require.NoError(t, err)
		assert.Equal(t, 42, trace.Id)
		assert.Equal(t, 7, trace.ParentId)
		assert.Equal(t, []int{3}, trace.Others)
	})
}

func TestCastBytesToBufSlice(t *testing.T) {
	data := []byte("test data")
	bufSlice, err := castBytesToBufSlice(data)
	assert.NoError(t, err)
	assert.NotNil(t, bufSlice)
	assert.Equal(t, data, bufSlice.Materialize())
}

func TestCryptoSvc_Encrypt_NoSession(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	svc := NewCryptoSvc(serverPrivKey)

	// Run on a fresh goroutine with no registered session in ancestry.
	done := make(chan struct{})
	go func() {
		defer close(done)
		trace, err := goAllIds()
		require.NoError(t, err)
		session_pub_keys.Remove(trace.Id)
		session_pub_keys.Remove(trace.ParentId)

		res, err := svc.Encrypt([]byte("test"))
		assert.Error(t, err)
		assert.Nil(t, res)
	}()
	<-done
}

func TestCryptoSvc_Encrypt_WithSession(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)

	clientPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	clientPubKey := clientPrivKey.PublicKey().Bytes()

	svc := NewCryptoSvc(serverPrivKey)

	trace, err := goAllIds()
	require.NoError(t, err)
	session_pub_keys.Store(trace.Id, clientPubKey)
	t.Cleanup(func() { session_pub_keys.Remove(trace.Id) })

	plaintext := []byte("secret message")
	encrypted, err := svc.Encrypt(plaintext)
	require.NoError(t, err)
	assert.True(t, len(encrypted) > len(plaintext))

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

func TestStreamDecryptCodec_UnmarshalRejectsInvalidAndEmptyFrames(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	codec := StreamDecryptCodec{Csvc: NewCryptoSvc(serverPrivKey)}

	emptySlice, err := castBytesToBufSlice([]byte{})
	require.NoError(t, err)
	defer emptySlice.Free()

	var out wrapperspb.StringValue
	err = codec.Unmarshal(emptySlice, &out)
	assert.Error(t, err)

	garbageSlice, err := castBytesToBufSlice(make([]byte, 80))
	require.NoError(t, err)
	defer garbageSlice.Free()

	err = codec.Unmarshal(garbageSlice, &out)
	assert.Error(t, err)
}

func TestStreamDecryptCodec_MarshalErrorsWithoutSession(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	codec := StreamDecryptCodec{Csvc: NewCryptoSvc(serverPrivKey)}

	done := make(chan struct{})
	go func() {
		defer close(done)
		trace, err := goAllIds()
		require.NoError(t, err)
		session_pub_keys.Remove(trace.Id)
		session_pub_keys.Remove(trace.ParentId)

		buf, err := codec.Marshal(wrapperspb.String("hello"))
		assert.Error(t, err)
		assert.Empty(t, buf)
	}()
	<-done
}

func TestStreamDecryptCodec_RoundTrip(t *testing.T) {
	serverPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)
	clientPrivKey, err := ecdh.X25519().GenerateKey(rand.Reader)
	require.NoError(t, err)

	codec := StreamDecryptCodec{Csvc: NewCryptoSvc(serverPrivKey)}
	trace, err := goAllIds()
	require.NoError(t, err)
	t.Cleanup(func() { session_pub_keys.Remove(trace.Id) })

	// Register session and verify Marshal -> Unmarshal round-trip (including empty message).
	RegisterCurrentGoroutine(clientPrivKey.PublicKey().Bytes())

	for _, msg := range []*wrapperspb.StringValue{
		wrapperspb.String("hello encrypted rpc"),
		wrapperspb.String(""),
	} {
		wire, err := codec.Marshal(msg)
		require.NoError(t, err)

		var decoded wrapperspb.StringValue
		err = codec.Unmarshal(wire, &decoded)
		wire.Free()
		require.NoError(t, err)
		assert.Equal(t, msg.GetValue(), decoded.GetValue())
	}
}
