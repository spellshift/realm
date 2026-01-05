package jwt

import (
	"crypto/ed25519"
	"fmt"
	"time"

	"github.com/golang-jwt/jwt"
)

// Service provides JWT generation and validation using ed25519 keys
type Service struct {
	privateKey ed25519.PrivateKey
	publicKey  ed25519.PublicKey
}

// TaskClaims represents the claims stored in a task JWT
type TaskClaims struct {
	TaskID   int64  `json:"task_id"`
	BeaconID string `json:"beacon_id,omitempty"`
	jwt.StandardClaims
}

// NewService creates a new JWT service with the provided ed25519 keys
func NewService(privateKey ed25519.PrivateKey, publicKey ed25519.PublicKey) (*Service, error) {
	if privateKey == nil || publicKey == nil {
		return nil, fmt.Errorf("private key and public key must not be nil")
	}

	return &Service{
		privateKey: privateKey,
		publicKey:  publicKey,
	}, nil
}

// GenerateTaskToken creates a JWT for a specific task
func (s *Service) GenerateTaskToken(taskID int64, beaconID string) (string, error) {
	now := time.Now()
	claims := TaskClaims{
		TaskID:   taskID,
		BeaconID: beaconID,
		StandardClaims: jwt.StandardClaims{
			IssuedAt:  now.Unix(),
			ExpiresAt: now.Add(24 * time.Hour).Unix(), // Token valid for 24 hours
			Issuer:    "tavern",
		},
	}

	token := jwt.NewWithClaims(&jwt.SigningMethodEd25519{}, claims)
	tokenString, err := token.SignedString(s.privateKey)
	if err != nil {
		return "", fmt.Errorf("failed to sign token: %w", err)
	}

	return tokenString, nil
}

// ValidateTaskToken validates a JWT and returns the task claims
func (s *Service) ValidateTaskToken(tokenString string) (*TaskClaims, error) {
	token, err := jwt.ParseWithClaims(tokenString, &TaskClaims{}, func(token *jwt.Token) (interface{}, error) {
		// Verify the signing method
		if _, ok := token.Method.(*jwt.SigningMethodEd25519); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
		}
		return s.publicKey, nil
	})

	if err != nil {
		return nil, fmt.Errorf("failed to parse token: %w", err)
	}

	if claims, ok := token.Claims.(*TaskClaims); ok && token.Valid {
		return claims, nil
	}

	return nil, fmt.Errorf("invalid token")
}

// GetPublicKey returns the public key for external verification
func (s *Service) GetPublicKey() ed25519.PublicKey {
	return s.publicKey
}
