package main

import (
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestEnvBool(t *testing.T) {
	tests := []struct {
		name      string
		env       EnvBool
		osValue   string
		wantValue bool
	}{
		{
			name:      "Set",
			env:       EnvBool{"TEST_ENV_STRING"},
			osValue:   "VALUE_SET",
			wantValue: true,
		},
		{
			name:      "Unset",
			env:       EnvBool{"TEST_ENV_STRING"},
			osValue:   "",
			wantValue: false,
		},
		{
			name:      "Default",
			env:       EnvBool{"TEST_ENV_STRING"},
			osValue:   "",
			wantValue: false,
		},
	}

	// Run Tests
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			if tc.osValue != "" {
				os.Setenv(tc.env.Key, tc.osValue)
				defer os.Unsetenv(tc.env.Key)
			}

			assert.Equal(t, tc.wantValue, tc.env.Bool())
			assert.Equal(t, tc.wantValue, tc.env.IsSet())
			assert.Equal(t, !tc.wantValue, tc.env.IsUnset())
			assert.Equal(t, fmt.Sprintf("%t", tc.wantValue), tc.env.String())
		})
	}
}

func TestEnvString(t *testing.T) {
	// Test Cases
	tests := []struct {
		name string

		env       EnvString
		osValue   string
		wantValue string
	}{
		{
			name:      "Set",
			env:       EnvString{"TEST_ENV_STRING", ""},
			osValue:   "VALUE_SET",
			wantValue: "VALUE_SET",
		},
		{
			name:      "Unset",
			env:       EnvString{"TEST_ENV_STRING", ""},
			osValue:   "",
			wantValue: "",
		},
		{
			name:      "Default",
			env:       EnvString{"TEST_ENV_STRING", "BLAH_BLAH"},
			osValue:   "",
			wantValue: "BLAH_BLAH",
		},
	}

	// Run Tests
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			if tc.osValue != "" {
				os.Setenv(tc.env.Key, tc.osValue)
				defer os.Unsetenv(tc.env.Key)
			}

			assert.Equal(t, tc.wantValue, tc.env.String())
		})
	}
}

func TestEnvInteger(t *testing.T) {
	// Test Cases
	tests := []struct {
		name string

		env       EnvInteger
		osValue   string
		wantValue int
	}{
		{
			name:      "Set",
			env:       EnvInteger{"TEST_ENV_INT", 0},
			osValue:   "123",
			wantValue: 123,
		},
		{
			name:      "Unset",
			env:       EnvInteger{"TEST_ENV_INT", 0},
			osValue:   "",
			wantValue: 0,
		},
		{
			name:      "Default",
			env:       EnvInteger{"TEST_ENV_INT", 456},
			osValue:   "",
			wantValue: 456,
		},
	}

	// Run Tests
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			if tc.osValue != "" {
				os.Setenv(tc.env.Key, tc.osValue)
				defer os.Unsetenv(tc.env.Key)
			}

			assert.Equal(t, tc.wantValue, tc.env.Int())
		})
	}
}
