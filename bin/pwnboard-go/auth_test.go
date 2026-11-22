package main

import (
	"context"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/cli/auth"
)

func TestGetAuthToken_PrioritizesTavernAPITokenOverLegacyKey(t *testing.T) {
	cachePath := filepath.Join(t.TempDir(), ".tavern-auth")
	require.NoError(t, os.WriteFile(cachePath, []byte("cached-token"), 0600))

	t.Setenv(auth.EnvAPIToken, "  primary-token\n")
	t.Setenv(legacyEnvAPIKey, "legacy-token")

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	token, err := getAuthToken(ctx, "https://tavern.example.com", cachePath)
	require.NoError(t, err)
	assert.Equal(t, "primary-token", string(token))
}

func TestGetAuthToken_FallsBackToLegacyAPIKey(t *testing.T) {
	cachePath := filepath.Join(t.TempDir(), ".tavern-auth")
	require.NoError(t, os.WriteFile(cachePath, []byte("cached-token"), 0600))

	t.Setenv(auth.EnvAPIToken, "   ")
	t.Setenv(legacyEnvAPIKey, "  legacy-token\r\n")

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	token, err := getAuthToken(ctx, "https://tavern.example.com", cachePath)
	require.NoError(t, err)
	assert.Equal(t, "legacy-token", string(token))
}
