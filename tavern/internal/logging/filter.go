package logging

import (
	"fmt"
	"log/slog"
	"strings"
	"sync"
)

var dynamicSubsystemRoots sync.Map

var builtinSubsystemRoots = map[string]struct{}{
	"app":         {},
	"audit":       {},
	"auth":        {},
	"builder":     {},
	"c2":          {},
	"cdn":         {},
	"cli":         {},
	"graphql":     {},
	"grpc":        {},
	"http":        {},
	"mcp":         {},
	"portals":     {},
	"redirectors": {},
	"scheduler":   {},
	"shells":      {},
	"storage":     {},
}

func registerSubsystemRoot(subsystem string) {
	root := subsystemRoot(subsystem)
	if root != "" {
		dynamicSubsystemRoots.Store(root, struct{}{})
	}
}

func subsystemRoot(subsystem string) string {
	s := strings.ToLower(strings.TrimSpace(subsystem))
	if idx := strings.IndexByte(s, '.'); idx >= 0 {
		return s[:idx]
	}
	return s
}

func isValidSubsystemIdentifier(s string) bool {
	if s == "" || strings.HasPrefix(s, ".") || strings.HasSuffix(s, ".") || strings.Contains(s, "..") {
		return false
	}
	for i := 0; i < len(s); i++ {
		ch := s[i]
		if (ch >= 'a' && ch <= 'z') || (ch >= '0' && ch <= '9') || ch == '.' || ch == '_' || ch == '-' {
			continue
		}
		return false
	}
	return true
}

func isKnownBareSubsystem(s string) bool {
	if !isValidSubsystemIdentifier(s) {
		return false
	}
	root := subsystemRoot(s)
	if _, ok := builtinSubsystemRoots[root]; ok {
		return true
	}
	_, ok := dynamicSubsystemRoots.Load(root)
	return ok
}

// Format controls how log records are serialized.
type Format string

const (
	FormatAuto Format = ""
	FormatText Format = "text"
	FormatJSON Format = "json"
)

// ParseFormat parses a log format string ("text", "json", or ""/"auto").
func ParseFormat(raw string) (Format, error) {
	switch strings.ToLower(strings.TrimSpace(raw)) {
	case "", "auto":
		return FormatAuto, nil
	case "text", "console", "pretty":
		return FormatText, nil
	case "json", "gcp":
		return FormatJSON, nil
	default:
		return FormatAuto, fmt.Errorf("invalid log format %q (expected text or json)", raw)
	}
}

// Preset identifies a named logging profile.
type Preset string

const (
	PresetNone Preset = ""
	PresetDev  Preset = "dev"
	PresetProd Preset = "prod"
)

// Spec holds a parsed logging filter specification and preset defaults.
type Spec struct {
	Raw                string
	Preset             Preset
	DefaultFormat      Format
	IncludeInstanceID  bool
	LogGraphQLRawQuery bool
	DefaultLevel       slog.Level
	Rules              map[string]slog.Level
}

// ParseSpec parses a comma-separated `--log` / `TAVERN_LOG` filter string.
//
// Supported directives (evaluated left-to-right):
//   - "prod": JSON format, instance ID, raw GraphQL queries, *=info (all per-request & audit logs enabled)
//   - "dev":  Text format, *=info with noisy ingress request logs (http.access, graphql.*, grpc.access) at warn
//   - "<level>" or "*=<level>": sets the fallback level for unmatched subsystems
//   - "<subsystem>": shorthand for "<subsystem>=debug" (e.g. `TAVERN_LOG=portals`)
//   - "<subsystem>=<level>": sets the threshold for a subsystem and its dotted children
//
// When only subsystem directives ("<subsystem>" or "<subsystem>=<level>") are specified without a
// preset or global level (e.g. `--log=portals` or `--log=portals=debug`), unmatched subsystems default to `error`.
func ParseSpec(raw string) (Spec, error) {
	trimmed := strings.TrimSpace(raw)
	if trimmed == "" {
		trimmed = string(PresetDev)
	}

	spec := Spec{
		Raw:           trimmed,
		Preset:        PresetNone,
		DefaultFormat: FormatText,
		DefaultLevel:  slog.LevelInfo,
		Rules:         make(map[string]slog.Level),
	}

	hasBase := false
	tokens := strings.Split(trimmed, ",")
	for _, tok := range tokens {
		tok = strings.TrimSpace(tok)
		if tok == "" {
			continue
		}

		lower := strings.ToLower(tok)
		if lower == string(PresetProd) {
			spec.Preset = PresetProd
			spec.DefaultFormat = FormatJSON
			spec.IncludeInstanceID = true
			spec.LogGraphQLRawQuery = true
			spec.DefaultLevel = slog.LevelInfo
			spec.Rules = make(map[string]slog.Level)
			hasBase = true
			continue
		}

		if lower == string(PresetDev) {
			spec.Preset = PresetDev
			spec.DefaultFormat = FormatText
			spec.IncludeInstanceID = false
			spec.LogGraphQLRawQuery = false
			spec.DefaultLevel = slog.LevelInfo
			spec.Rules = map[string]slog.Level{
				"http.access":      slog.LevelWarn,
				"graphql.query":    slog.LevelWarn,
				"graphql.mutation": slog.LevelWarn,
				"grpc.access":      slog.LevelWarn,
			}
			hasBase = true
			continue
		}

		if idx := strings.IndexByte(tok, '='); idx >= 0 {
			target := strings.ToLower(strings.TrimSpace(tok[:idx]))
			lvlStr := strings.TrimSpace(tok[idx+1:])
			if target == "" {
				return Spec{}, fmt.Errorf("invalid log directive %q: empty subsystem name", tok)
			}
			lvl, ok := ParseLevel(lvlStr)
			if !ok {
				return Spec{}, fmt.Errorf("invalid log level %q in directive %q", lvlStr, tok)
			}
			if target == "*" {
				spec.DefaultLevel = lvl
				hasBase = true
				continue
			}
			if !isValidSubsystemIdentifier(target) {
				return Spec{}, fmt.Errorf("invalid subsystem name %q in directive %q", target, tok)
			}
			spec.Rules[target] = lvl
			continue
		}

		if lower == "*" {
			spec.DefaultLevel = slog.LevelDebug
			hasBase = true
			continue
		}

		if lvl, ok := ParseLevel(tok); ok {
			spec.DefaultLevel = lvl
			hasBase = true
			continue
		}

		if isKnownBareSubsystem(lower) {
			spec.Rules[lower] = slog.LevelDebug
			continue
		}

		return Spec{}, fmt.Errorf("invalid log directive %q: expected preset (prod, dev), level, subsystem, or subsystem=level", tok)
	}

	if !hasBase && len(spec.Rules) > 0 {
		spec.DefaultLevel = slog.LevelError
	}

	return spec, nil
}

// ResolveLevel returns the effective minimum slog.Level for a dotted subsystem name
// using longest-prefix matching on dot boundaries.
func (s Spec) ResolveLevel(subsystem string) slog.Level {
	if len(s.Rules) == 0 {
		return s.DefaultLevel
	}

	curr := strings.ToLower(strings.TrimSpace(subsystem))
	for curr != "" {
		if lvl, ok := s.Rules[curr]; ok {
			return lvl
		}
		idx := strings.LastIndexByte(curr, '.')
		if idx < 0 {
			break
		}
		curr = curr[:idx]
	}

	return s.DefaultLevel
}
