package builder_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder"
)

func writeTestConfig(t *testing.T, dir string, body string) string {
	t.Helper()
	path := filepath.Join(dir, "builder.yaml")
	require.NoError(t, os.WriteFile(path, []byte(body), 0600))
	return path
}

func TestParseConfig_ResolvesIdentityFileRelativeToConfigDir(t *testing.T) {
	dir := t.TempDir()
	path := writeTestConfig(t, dir, "supported_targets: [linux]\nidentity_file: builder.identity\nupstream: http://127.0.0.1:8000\n")

	cfg, err := builder.ParseConfig(path)
	require.NoError(t, err)
	require.Equal(t, filepath.Join(dir, "builder.identity"), cfg.IdentityFile)
}

func TestParseConfig_AbsoluteIdentityFileUnchanged(t *testing.T) {
	dir := t.TempDir()
	abs := filepath.Join(t.TempDir(), "builder.identity")
	path := writeTestConfig(t, dir, "supported_targets: [linux]\nidentity_file: "+abs+"\nupstream: http://127.0.0.1:8000\n")

	cfg, err := builder.ParseConfig(path)
	require.NoError(t, err)
	require.Equal(t, abs, cfg.IdentityFile)
}

func TestParseConfigBytes_DefaultsAndPartialOverrides(t *testing.T) {
	cfg, err := builder.ParseConfigBytes(nil)
	require.NoError(t, err)
	require.Equal(t, &builder.Config{
		PollInterval: 5, SupportedTargets: []string{"linux", "windows"},
		IdentityFile: "builder.identity", Upstream: "http://127.0.0.1:8000", Executor: "docker",
	}, cfg)

	cfg, err = builder.ParseConfigBytes([]byte("upstream: https://tavern.example.com\npoll_interval: 10\n"))
	require.NoError(t, err)
	require.Equal(t, "https://tavern.example.com", cfg.Upstream)
	require.Equal(t, 10, cfg.PollInterval)
	require.Equal(t, []string{"linux", "windows"}, cfg.SupportedTargets)
}

func TestParseConfigBytes_RejectsExplicitInvalidFields(t *testing.T) {
	tests := []struct {
		name string
		body string
		want string
	}{
		{"identity file", "identity_file: ''\n", "identity_file"},
		{"supported targets", "supported_targets: []\n", "supported_target"},
		{"upstream", "upstream: ''\n", "upstream"},
		{"executor", "executor: ''\n", "executor"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := builder.ParseConfigBytes([]byte(tt.body))
			require.ErrorContains(t, err, tt.want)
		})
	}
}

func TestParseConfigBytes_RejectsUnsupportedTarget(t *testing.T) {
	_, err := builder.ParseConfigBytes([]byte("supported_targets: [plan9]\nidentity_file: builder.identity\nupstream: http://127.0.0.1:8000\n"))
	require.ErrorContains(t, err, "unsupported target")
}

func TestParseConfigBytes_RejectsPollIntervalOutOfRange(t *testing.T) {
	_, err := builder.ParseConfigBytes([]byte("supported_targets: [linux]\nidentity_file: builder.identity\nupstream: http://127.0.0.1:8000\npoll_interval: 0\n"))
	require.ErrorContains(t, err, "poll_interval")

	_, err = builder.ParseConfigBytes([]byte("supported_targets: [linux]\nidentity_file: builder.identity\nupstream: http://127.0.0.1:8000\npoll_interval: 999999\n"))
	require.ErrorContains(t, err, "poll_interval")
}

func TestParseConfigBytes_DefaultsAndRejectsUnsupportedExecutor(t *testing.T) {
	cfg, err := builder.ParseConfigBytes([]byte("supported_targets: [linux]\nidentity_file: builder.identity\nupstream: http://127.0.0.1:8000\n"))
	require.NoError(t, err)
	require.Equal(t, "docker", cfg.Executor)
	require.Equal(t, builder.DefaultPollInterval, cfg.PollInterval)

	_, err = builder.ParseConfigBytes([]byte("supported_targets: [linux]\nidentity_file: builder.identity\nupstream: http://127.0.0.1:8000\nexecutor: unsupported\n"))
	require.ErrorContains(t, err, "unsupported executor")
}

func TestParseConfigBytes_ValidatesUpstreamURL(t *testing.T) {
	for _, upstream := range []string{"tavern:8000", "ftp://tavern.example.com", "http://", "https:///path", "127.0.0.1:8000"} {
		_, err := builder.ParseConfigBytes([]byte("supported_targets: [linux]\nidentity_file: id\nupstream: " + upstream + "\n"))
		require.Error(t, err, upstream)
	}
	for _, upstream := range []string{"http://127.0.0.1:8000", "https://tavern.example.com"} {
		_, err := builder.ParseConfigBytes([]byte("supported_targets: [linux]\nidentity_file: id\nupstream: " + upstream + "\n"))
		require.NoError(t, err, upstream)
	}
}
