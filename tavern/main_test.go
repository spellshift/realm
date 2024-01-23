package main

import (
	"os"
	"testing"
)

// TestMainFunc runs main after configuring the application to immediately exit.
// This validates our default configurations are successful.
func TestMainFunc(t *testing.T) {
	os.Setenv(EnvEnableTestRunAndExit.Key, "1")
	os.Setenv(EnvHTTPListenAddr.Key, "127.0.0.1:8080")
	defer func() {
		if err := os.Unsetenv(EnvEnableTestRunAndExit.Key); err != nil {
			t.Fatalf("failed to unset env var %s: %v", EnvEnableTestRunAndExit.Key, err)
		}
		if err := os.Unsetenv(EnvHTTPListenAddr.Key); err != nil {
			t.Fatalf("failed to unset env var %s: %v", EnvHTTPListenAddr.Key, err)
		}
	}()
	os.Args = []string{"tavern"}
	main()
}
