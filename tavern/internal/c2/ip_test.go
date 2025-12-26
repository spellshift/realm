package c2

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestValidateIP(t *testing.T) {
	tests := []struct {
		name     string
		ipaddr   string
		expected bool
	}{
		{"IPv4", "127.0.0.1", true},
		{"IPv6", "::1", true},
		{"Unknown", "unknown", true},
		{"Invalid", "invalid", false},
		{"Empty", "", false},
		{"IPv4 with port", "127.0.0.1:80", false},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			assert.Equal(t, tc.expected, validateIP(tc.ipaddr))
		})
	}
}
