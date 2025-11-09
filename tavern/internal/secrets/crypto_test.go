package secrets

import (
	"crypto/ecdh"
	"crypto/rand"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestCrypto(t *testing.T) {
	// Generate two key pairs.
	curve := ecdh.X25519()
	priv1, err := curve.GenerateKey(rand.Reader)
	require.NoError(t, err)
	pub1 := priv1.PublicKey()
	priv2, err := curve.GenerateKey(rand.Reader)
	require.NoError(t, err)
	pub2 := priv2.PublicKey()

	// Generate two shared keys.
	shared1, err := GenerateSharedKey(priv1, pub2)
	require.NoError(t, err)
	shared2, err := GenerateSharedKey(priv2, pub1)
	require.NoError(t, err)
	assert.Equal(t, shared1, shared2)

	// Test encryption and decryption.
	plaintext := []byte("hello world")
	ciphertext, err := Encrypt(shared1, plaintext)
	require.NoError(t, err)
	decrypted, err := Decrypt(shared2, ciphertext)
	require.NoError(t, err)
	assert.Equal(t, plaintext, decrypted)
}
