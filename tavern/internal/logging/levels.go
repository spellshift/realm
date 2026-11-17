package logging

import (
	"log/slog"
	"strings"
)

const (
	// LevelTrace is used for high-frequency wire/chunk/mote diagnostics below DEBUG.
	LevelTrace slog.Level = slog.LevelDebug - 4

	// LevelOff disables logging for a target subsystem or globally.
	LevelOff slog.Level = slog.LevelError + 100
)

// ParseLevel parses a case-insensitive level string into an slog.Level.
func ParseLevel(raw string) (slog.Level, bool) {
	switch strings.ToLower(strings.TrimSpace(raw)) {
	case "trace":
		return LevelTrace, true
	case "debug":
		return slog.LevelDebug, true
	case "info":
		return slog.LevelInfo, true
	case "warn", "warning":
		return slog.LevelWarn, true
	case "error", "err":
		return slog.LevelError, true
	case "off", "none", "silent":
		return LevelOff, true
	default:
		return slog.LevelInfo, false
	}
}

// LevelName returns the canonical uppercase name of a level.
func LevelName(l slog.Level) string {
	switch {
	case l <= LevelTrace:
		return "TRACE"
	case l < slog.LevelInfo:
		return "DEBUG"
	case l < slog.LevelWarn:
		return "INFO"
	case l < slog.LevelError:
		return "WARN"
	case l >= LevelOff:
		return "OFF"
	default:
		return "ERROR"
	}
}

// LevelShortLabel returns a fixed 3-char label for compact console output.
func LevelShortLabel(l slog.Level, isAudit bool) string {
	if isAudit && l == slog.LevelInfo {
		return "AUD"
	}
	switch {
	case l <= LevelTrace:
		return "TRC"
	case l < slog.LevelInfo:
		return "DBG"
	case l < slog.LevelWarn:
		return "INF"
	case l < slog.LevelError:
		return "WRN"
	default:
		return "ERR"
	}
}

// GCPSeverity maps slog levels and audit flags to Google Cloud Logging LogSeverity strings.
func GCPSeverity(l slog.Level, isAudit bool) string {
	if isAudit && l == slog.LevelInfo {
		return "NOTICE"
	}
	switch {
	case l < slog.LevelInfo:
		return "DEBUG"
	case l < slog.LevelWarn:
		return "INFO"
	case l < slog.LevelError:
		return "WARNING"
	default:
		return "ERROR"
	}
}
