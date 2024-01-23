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
	os.Setenv(EnvEnablePProf.Key, "1")
	defer func() {
		unsetList := []string{
			EnvEnableTestRunAndExit.Key,
			EnvHTTPListenAddr.Key,
			EnvEnablePProf.Key,
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
