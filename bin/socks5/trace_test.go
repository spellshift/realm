package main

import (
	"testing"
)

func TestFormatDuration(t *testing.T) {
	tests := []struct {
		micros int64
		want   string
	}{
		{1_543_200, "1.5432 seconds"},
		{123_543, "123.5430ms"},
		{40, "40µs"},
		{1_000, "1.0000ms"},
		{1_000_000, "1.0000 seconds"},
		{0, "0µs"},
	}

	for _, tt := range tests {
		got := formatDuration(tt.micros)
		if got != tt.want {
			t.Errorf("formatDuration(%d) = %q; want %q", tt.micros, got, tt.want)
		}
	}
}
