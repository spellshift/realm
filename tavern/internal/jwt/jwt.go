package jwt

import (
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"fmt"
	"log/slog"
	"os"
	"time"

	"github.com/golang-jwt/jwt"
	"realm.pub/tavern/internal/secrets"
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

// NewService creates a new JWT service with persistent ed25519 keys
func NewService() (*Service, error) {
	secretsManager, err := newSecretsManager()
	if err != nil || secretsManager == nil {
		return nil, fmt.Errorf("failed to configure secret manager: %w", err)
	}

	// Check if we already have a key
	privateKeyBytes, err := secretsManager.GetValue("tavern_jwt_ed25519_private_key")
	if err != nil {
		// Generate a new key pair if it doesn't exist
		pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
		if err != nil {
			return nil, fmt.Errorf("failed to generate ed25519 keypair: %w", err)
		}

		// Marshal and store the private key
		privateKeyBytes, err := x509.MarshalPKCS8PrivateKey(privKey)
		if err != nil {
			return nil, fmt.Errorf("unable to marshal private key: %w", err)
		}

		_, err = secretsManager.SetValue("tavern_jwt_ed25519_private_key", privateKeyBytes)
		if err != nil {
			return nil, fmt.Errorf("unable to set 'tavern_jwt_ed25519_private_key' using secrets manager: %w", err)
		}

		slog.Info("Generated new ed25519 keypair for JWT signing")

		return &Service{
			privateKey: privKey,
			publicKey:  pubKey,
		}, nil
	}

	// Parse existing private key
	tmp, err := x509.ParsePKCS8PrivateKey(privateKeyBytes)
	if err != nil {
		return nil, fmt.Errorf("unable to parse private key: %w", err)
	}

	privateKey, ok := tmp.(ed25519.PrivateKey)
	if !ok {
		return nil, fmt.Errorf("expected ed25519.PrivateKey, got %T", tmp)
	}

	publicKey := privateKey.Public().(ed25519.PublicKey)

	slog.Info("Loaded existing ed25519 keypair for JWT signing")

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

// newSecretsManager creates a secrets manager instance
// This function is intentionally package-private and duplicates logic from app.go
// to avoid circular dependencies
func newSecretsManager() (secrets.SecretsManager, error) {
	// Read environment variables directly to avoid circular imports
	gcpProjectID := os.Getenv("GCP_PROJECT_ID")
	secretsPath := os.Getenv("SECRETS_FILE_PATH")

	if gcpProjectID == "" && secretsPath == "" {
		slog.Warn("No configuration provided for secret manager path, using a potentially insecure default.")
		return secrets.NewDebugFileSecrets("/tmp/tavern-secrets")
	}

	if secretsPath == "" {
		return secrets.NewGcp(gcpProjectID)
	}

	return secrets.NewDebugFileSecrets(secretsPath)
}
