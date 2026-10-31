package main

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestLoadBuilderConfig(t *testing.T) {
	cfg, err := loadBuilderConfig("", nil)
	require.NoError(t, err)
	require.Equal(t, "builder.identity", cfg.IdentityFile)
	require.Equal(t, []string{"linux", "windows"}, cfg.SupportedTargets)
	require.Equal(t, "docker", cfg.Executor)

	path := filepath.Join(t.TempDir(), "builder.yaml")
	require.NoError(t, os.WriteFile(path, []byte("upstream: https://tavern.example.com\nidentity_file: custom.identity\nexecutor: local\n"), 0600))
	docker := "docker"
	cfg, err = loadBuilderConfig(path, &docker)
	require.NoError(t, err)
	require.Equal(t, "https://tavern.example.com", cfg.Upstream)
	require.Equal(t, filepath.Join(filepath.Dir(path), "custom.identity"), cfg.IdentityFile)
	require.Equal(t, "docker", cfg.Executor)

	require.NoError(t, os.WriteFile(path, []byte("upstream: https://tavern.example.com\nexecutor: unsupported\n"), 0600))
	cfg, err = loadBuilderConfig(path, &docker)
	require.NoError(t, err)
	require.Equal(t, "docker", cfg.Executor)
	_, err = loadBuilderConfig(path, nil)
	require.ErrorContains(t, err, "unsupported executor")

	require.NoError(t, os.WriteFile(path, []byte("upstream: invalid\nexecutor: unsupported\n"), 0600))
	_, err = loadBuilderConfig(path, &docker)
	require.ErrorContains(t, err, "upstream")

	invalid := "invalid"
	_, err = loadBuilderConfig("", &invalid)
	require.ErrorContains(t, err, "unsupported executor")
	empty := ""
	_, err = loadBuilderConfig("", &empty)
	require.ErrorContains(t, err, "unsupported executor")
	_, err = loadBuilderConfig(filepath.Join(t.TempDir(), "missing.yaml"), nil)
	require.ErrorContains(t, err, "failed to parse builder config")
}

func TestBuilderCLIRejectsExplicitEmptyExecutor(t *testing.T) {
	err := newApp(context.Background()).Run([]string{"tavern", "builder", "--executor", ""})
	require.ErrorContains(t, err, "unsupported executor")
}
