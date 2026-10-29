package main

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestLoadBuilderConfig(t *testing.T) {
	cfg, err := loadBuilderConfig("", "")
	require.NoError(t, err)
	require.Equal(t, "builder.identity", cfg.IdentityFile)
	require.Equal(t, []string{"linux", "windows"}, cfg.SupportedTargets)
	require.Equal(t, "docker", cfg.Executor)

	path := filepath.Join(t.TempDir(), "builder.yaml")
	require.NoError(t, os.WriteFile(path, []byte("upstream: https://tavern.example.com\nidentity_file: custom.identity\nexecutor: local\n"), 0600))
	cfg, err = loadBuilderConfig(path, "docker")
	require.NoError(t, err)
	require.Equal(t, "https://tavern.example.com", cfg.Upstream)
	require.Equal(t, filepath.Join(filepath.Dir(path), "custom.identity"), cfg.IdentityFile)
	require.Equal(t, "docker", cfg.Executor)

	_, err = loadBuilderConfig("", "invalid")
	require.ErrorContains(t, err, "unsupported executor")
	_, err = loadBuilderConfig(filepath.Join(t.TempDir(), "missing.yaml"), "")
	require.ErrorContains(t, err, "failed to parse builder config")
}
