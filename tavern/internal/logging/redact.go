package logging

import (
	"encoding/json"
	"log/slog"
	"strings"
)

// RedactedPlaceholder is substituted for sensitive values in logs and audit details.
const RedactedPlaceholder = "[REDACTED]"

type redactedValuer struct{}

func (redactedValuer) LogValue() slog.Value {
	return slog.StringValue(RedactedPlaceholder)
}

// Redacted returns an slog.LogValuer that always renders "[REDACTED]" without formatting the underlying secret.
func Redacted(_ any) slog.LogValuer {
	return redactedValuer{}
}

// IsSensitiveKey reports whether an attribute, GraphQL variable, or parameter key holds secret material.
// Non-secret identifier/metadata keys such as `token_id`, `credential_id`, `public_key`, or `server_pubkey` return false.
func IsSensitiveKey(key string) bool {
	norm := normalizeKey(key)
	if norm == "" {
		return false
	}

	// Safe suffixes/keys that reference IDs or public material rather than secret values.
	switch norm {
	case "tokenid", "credentialid", "secretid", "publickey", "pubkey", "serverpubkey", "ed25519pubkey", "x25519pubkey":
		return false
	}
	if strings.HasSuffix(norm, "tokenid") || strings.HasSuffix(norm, "credentialid") || strings.HasSuffix(norm, "pubkey") || strings.HasSuffix(norm, "publickey") {
		return false
	}

	switch norm {
	case "secret", "clientsecret", "oauthsecret",
		"password", "passwd", "passphrase",
		"privatekey", "privkey", "sshprivatekey",
		"token", "accesstoken", "sessiontoken", "refreshtoken", "bearertoken", "enrollmenttoken", "idtoken",
		"apikey", "jwt", "authorization", "cookie", "setcookie",
		"csr", "keypem":
		return true
	}

	sensitiveSuffixes := []string{
		"secret",
		"password",
		"passwd",
		"passphrase",
		"privatekey",
		"privkey",
		"accesstoken",
		"sessiontoken",
		"refreshtoken",
		"apikey",
	}
	for _, suf := range sensitiveSuffixes {
		if strings.HasSuffix(norm, suf) {
			return true
		}
	}
	return false
}

func normalizeKey(k string) string {
	var b strings.Builder
	b.Grow(len(k))
	for i := 0; i < len(k); i++ {
		c := k[i]
		if c == '_' || c == '-' || c == '.' || c == ' ' {
			continue
		}
		if c >= 'A' && c <= 'Z' {
			c += 'a' - 'A'
		}
		b.WriteByte(c)
	}
	return b.String()
}

// ScrubMap returns a deep-scrubbed copy of m with sensitive keys replaced by "[REDACTED]".
// It never mutates the input map in place.
func ScrubMap(m map[string]any) map[string]any {
	if m == nil {
		return nil
	}
	out := make(map[string]any, len(m))
	for k, v := range m {
		if IsSensitiveKey(k) {
			out[k] = RedactedPlaceholder
			continue
		}
		out[k] = ScrubValue(v)
	}
	return out
}

// ScrubValue recursively scrubs maps and slices without mutating caller data structures.
func ScrubValue(v any) any {
	switch typed := v.(type) {
	case map[string]any:
		return ScrubMap(typed)
	case map[string]string:
		out := make(map[string]string, len(typed))
		for k, val := range typed {
			if IsSensitiveKey(k) {
				out[k] = RedactedPlaceholder
				continue
			}
			out[k] = val
		}
		return out
	case []any:
		out := make([]any, len(typed))
		for i, elem := range typed {
			out[i] = ScrubValue(elem)
		}
		return out
	case []map[string]any:
		out := make([]map[string]any, len(typed))
		for i, elem := range typed {
			out[i] = ScrubMap(elem)
		}
		return out
	case string:
		trimmed := strings.TrimSpace(typed)
		if (strings.HasPrefix(trimmed, "{") && strings.HasSuffix(trimmed, "}")) ||
			(strings.HasPrefix(trimmed, "[") && strings.HasSuffix(trimmed, "]")) {
			return ScrubJSONString(typed)
		}
		return typed
	default:
		return v
	}
}

// ScrubJSONValue parses a JSON string into a scrubbed structured value (map/slice) when valid JSON,
// or falls back to returning the raw string when non-JSON.
func ScrubJSONValue(raw string) any {
	trimmed := strings.TrimSpace(raw)
	if trimmed == "" {
		return raw
	}
	var parsed any
	if err := json.Unmarshal([]byte(trimmed), &parsed); err != nil {
		return raw
	}
	return ScrubValue(parsed)
}

// ScrubJSONString parses a JSON object/array string (such as Tome parameters), scrubs any sensitive keys,
// and returns the scrubbed JSON string. Non-JSON or empty strings are returned unchanged.
func ScrubJSONString(raw string) string {
	trimmed := strings.TrimSpace(raw)
	if trimmed == "" {
		return raw
	}
	var parsed any
	if err := json.Unmarshal([]byte(trimmed), &parsed); err != nil {
		return raw
	}
	scrubbed := ScrubValue(parsed)
	encoded, err := json.Marshal(scrubbed)
	if err != nil {
		return raw
	}
	return string(encoded)
}
