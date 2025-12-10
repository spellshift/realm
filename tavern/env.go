package main

import (
	"fmt"
	"log"
	"log/slog"
	"os"
	"strconv"
)

// EnvBool represents a boolean that is configured using environment variables.
// Any non-empty value for the variable sets it to true, however the common format is VAR=1.
// The default is always false, so plan accordingly.
type EnvBool struct {
	Key string
}

// String value of if the boolean is true or false.
func (env EnvBool) String() string {
	return fmt.Sprintf("%t", env.Bool())
}

// Bool based on the environment variable value.
// True if any non-empty value is set, false otherwise.
func (env EnvBool) Bool() bool {
	if val := os.Getenv(env.Key); val != "" {
		return true
	}
	slog.Warn("missing configuration, using default value", "env_var", env.Key, "type", "bool", "default", false)
	return false
}

// IsSet returns true if the boolean is true.
func (env EnvBool) IsSet() bool {
	return env.Bool()
}

// IsUnset returns false if the boolean is true.
func (env EnvBool) IsUnset() bool {
	return !env.Bool()
}

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
	slog.Warn("missing configuration, using default value", "env_var", env.Key, "type", "string", "default", env.Default)
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
		slog.Warn("missing configuration, using default value", "env_var", env.Key, "type", "int", "default", env.Default)
		return env.Default
	}
	val, err := strconv.Atoi(envVar)
	if err != nil {
		log.Fatalf("[FATAL] invalid integer value (%q) provided for %s: %v", envVar, env.Key, err)
	}
	return val
}
