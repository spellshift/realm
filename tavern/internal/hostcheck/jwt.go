package hostcheck

import (
	"crypto/ed25519"
	"fmt"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

// Audience is the JWT audience value that restricts tokens to the host-check endpoint.
const Audience = "/internal/host-check"

// NewToken creates a signed JWT valid only for the host-check endpoint.
// The token has no expiration because it is used by a long-running scheduled job.
func NewToken(privKey ed25519.PrivateKey) (string, error) {
	claims := jwt.RegisteredClaims{
		Audience:  jwt.ClaimStrings{Audience},
		IssuedAt:  jwt.NewNumericDate(time.Now()),
		NotBefore: jwt.NewNumericDate(time.Now()),
	}
	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	tokenStr, err := token.SignedString(privKey)
	if err != nil {
		return "", fmt.Errorf("hostcheck: failed to sign JWT: %w", err)
	}
	return tokenStr, nil
}

// VerifyToken parses and validates a host-check JWT.
// It ensures the token was signed with the expected key and targets the correct audience.
func VerifyToken(tokenStr string, pubKey ed25519.PublicKey) error {
	token, err := jwt.ParseWithClaims(tokenStr, &jwt.RegisteredClaims{}, func(token *jwt.Token) (interface{}, error) {
		if _, ok := token.Method.(*jwt.SigningMethodEd25519); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
		}
		return pubKey, nil
	}, jwt.WithAudience(Audience))
	if err != nil {
		return fmt.Errorf("hostcheck: invalid token: %w", err)
	}
	if !token.Valid {
		return fmt.Errorf("hostcheck: token is not valid")
	}
	return nil
}
