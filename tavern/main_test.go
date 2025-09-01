package main

import (
	"os"
	"path"
	"testing"
)

// TestMainFunc runs main after configuring the application to immediately exit.
// This validates our default configurations are successful.
func TestMainFunc(t *testing.T) {
	tmpDir := t.TempDir()
	path := path.Join(tmpDir, "secrets.yaml")

	os.Setenv(EnvEnableTestRunAndExit.Key, "1")
	os.Setenv(EnvHTTPListenAddr.Key, "127.0.0.1:8080")
	os.Setenv(EnvHTTPMetricsListenAddr.Key, "127.0.0.1:8081")
	os.Setenv(EnvEnablePProf.Key, "1")
	os.Setenv(EnvEnableMetrics.Key, "1")
	os.Setenv(EnvSecretsManagerPath.Key, path)
	defer func() {
		unsetList := []string{
			EnvEnableTestRunAndExit.Key,
			EnvHTTPListenAddr.Key,
			EnvHTTPMetricsListenAddr.Key,
			EnvEnablePProf.Key,
			EnvEnableMetrics.Key,
		}
		for _, unset := range unsetList {
			if err := os.Unsetenv(unset); err != nil {
				t.Fatalf("failed to unset env var %s: %v", unset, err)
			}
		}
	}()
	os.Args = []string{"tavern"}
	main()
}
