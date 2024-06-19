package main

import (
	"log"
	"os"
	"strconv"
)

// EnvString represents a string that is configured using environment variables.
type EnvString struct {
	Key     string
	Default string
}

// String parsed from the environment variable.
func (env EnvString) String() string {
	if val := os.Getenv(env.Key); val != "" {
		return val
	}
	if env.Default != "" {
		log.Printf("[WARN] No value for '%s' provided, defaulting to %s", env.Key, env.Default)
	}
	return env.Default
}

// EnvInteger represents an integer that is configured using environment variables.
type EnvInteger struct {
	Key     string
	Default int
}

// Int parsed from the environment variable.
func (env EnvInteger) Int() int {
	envVar := os.Getenv(env.Key)
	if envVar == "" {
		log.Printf("[WARN] No value for '%s' provided, defaulting to %d", env.Key, env.Default)
		return env.Default
	}
	val, err := strconv.Atoi(envVar)
	if err != nil {
		log.Fatalf("[FATAL] Invalid integer value (%q) provided for %s: %v", envVar, env.Key, err)
	}
	return val
}
