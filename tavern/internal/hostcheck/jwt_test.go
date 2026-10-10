package hostcheck

import (
	"crypto/ed25519"
	"crypto/rand"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewTokenAndVerifyToken(t *testing.T) {
	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	tokenStr, err := NewToken(privKey)
	require.NoError(t, err)
	assert.NotEmpty(t, tokenStr)

	err = VerifyToken(tokenStr, pubKey)
	assert.NoError(t, err)
}

func TestVerifyTokenWrongKey(t *testing.T) {
	_, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	otherPubKey, _, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	tokenStr, err := NewToken(privKey)
	require.NoError(t, err)

	err = VerifyToken(tokenStr, otherPubKey)
	assert.Error(t, err)
}

func TestVerifyTokenWrongAudience(t *testing.T) {
	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	// Create a token with a different audience
	claims := jwt.RegisteredClaims{
		Audience:  jwt.ClaimStrings{"/some/other/endpoint"},
		IssuedAt:  jwt.NewNumericDate(time.Now()),
		NotBefore: jwt.NewNumericDate(time.Now()),
	}
	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	tokenStr, err := token.SignedString(privKey)
	require.NoError(t, err)

	err = VerifyToken(tokenStr, pubKey)
	assert.Error(t, err)
}

func TestVerifyTokenInvalidString(t *testing.T) {
	pubKey, _, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	err = VerifyToken("not-a-valid-jwt", pubKey)
	assert.Error(t, err)
}
