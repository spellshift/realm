package main

import "os"

// envString represents a string configured via an environment variable, falling back to Default
// when unset or empty. This is a small, local copy of the equivalent helper in tavern/env.go:
// that type lives in tavern's own `package main` and can't be imported from here.
type envString struct {
	Key     string
	Default string
}

// String returns the parsed value of the environment variable, or Default if unset.
func (env envString) String() string {
	if val := os.Getenv(env.Key); val != "" {
		return val
	}
	return env.Default
}
