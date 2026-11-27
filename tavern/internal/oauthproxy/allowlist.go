package oauthproxy

import (
	"fmt"
	"net"
	"net/url"
	"strings"
)

// CompletePath is the only path on a tenant that may receive an assertion.
const CompletePath = "/oauth/complete"

// IsAllowedReturnTo reports whether returnTo is a URL whose host matches one of the allowed
// patterns. A pattern is one of:
//   - an exact host (e.g. "tavern.example.com", "localhost", or an IP literal like "127.0.0.1"),
//   - a dot-prefixed domain suffix (e.g. ".example.com", matching any subdomain of example.com as
//     well as example.com itself), or
//   - a leading-wildcard pattern (e.g. "*-tavern-749076704275.us-east4.run.app" or
//     "*.example.com"), where "*" matches a non-empty prefix within the leftmost DNS label (no "."
//     characters).
//
// returnTo's host must be an ASCII IP literal or valid ASCII DNS hostname (rejecting non-ASCII
// Unicode dot separators like U+3002/U+FF0E/U+FF61 and invalid label syntax).
// returnTo's path must be exactly CompletePath, with no userinfo, query or fragment.
//
// returnTo must use https, with one exception: a loopback host (localhost/127.0.0.1/::1) may use
// plain http, since there is no network between the browser and a loopback address for a
// man-in-the-middle to exploit. This mirrors the same allowance Google's own OAuth client
// console makes for "http://localhost" redirect URIs, and exists solely so the whole flow can be
// exercised against a local `go run ./tavern` + `go run ./tavern/oauthproxy` without TLS.
//
// This is the proxy's only defense against being tricked into completing a Google login for an
// attacker-controlled destination (an open-redirect / credential-relay risk), so callers must
// invoke it both before starting a flow and again before completing one.
func IsAllowedReturnTo(returnTo string, allowedSuffixes []string) bool {
	u, err := url.Parse(returnTo)
	if err != nil {
		return false
	}

	// Only Tavern's own completion endpoint may receive an assertion: no other path, no
	// userinfo, and no query or fragment that could corrupt or hide the appended assertion.
	if u.Path != CompletePath || u.User != nil || u.RawQuery != "" || u.Fragment != "" || u.Opaque != "" {
		return false
	}

	rawHost := u.Hostname()
	if rawHost == "" || !isASCII(rawHost) {
		return false
	}
	host := strings.ToLower(rawHost)
	if net.ParseIP(host) == nil && !isValidDNSName(host) {
		return false
	}

	if u.Scheme != "https" {
		if u.Scheme != "http" || !isLoopbackHost(host) {
			return false
		}
	}

	for _, pattern := range allowedSuffixes {
		if matchAllowedHost(host, pattern) {
			return true
		}
	}

	return false
}

// ValidateAllowedPattern reports whether pattern is a syntactically valid host allowlist entry:
// an exact ASCII DNS hostname or IP literal, a dot-prefixed ASCII domain suffix (e.g.
// ".example.com"), or a leading-wildcard single-label pattern (e.g.
// "*-tavern-749076704275.us-east4.run.app" or "*.example.com").
func ValidateAllowedPattern(pattern string) error {
	raw := strings.TrimSpace(pattern)
	if raw == "" {
		return fmt.Errorf("oauthproxy: empty allowlist pattern")
	}
	if !isASCII(raw) {
		return fmt.Errorf("oauthproxy: allowlist pattern %q must contain only ASCII characters", raw)
	}
	p := strings.ToLower(raw)

	if strings.HasPrefix(p, ".") {
		bare := p[1:]
		if net.ParseIP(bare) != nil || !isValidDNSName(bare) {
			return fmt.Errorf("oauthproxy: invalid dot-prefixed allowlist suffix %q", raw)
		}
		return nil
	}

	if strings.HasPrefix(p, "*") {
		rest := p[1:]
		if rest == "" || strings.Contains(rest, "*") {
			return fmt.Errorf("oauthproxy: invalid wildcard allowlist pattern %q", raw)
		}
		if strings.HasPrefix(rest, ".") {
			domain := rest[1:]
			if net.ParseIP(domain) != nil || !isValidDNSName(domain) {
				return fmt.Errorf("oauthproxy: invalid wildcard allowlist pattern %q", raw)
			}
			return nil
		}
		labelSuffix, domain, ok := strings.Cut(rest, ".")
		if !ok || net.ParseIP(domain) != nil || !isValidDNSName(domain) {
			return fmt.Errorf("oauthproxy: invalid wildcard allowlist pattern %q", raw)
		}
		if !isValidWildcardLabelSuffix(labelSuffix) {
			return fmt.Errorf("oauthproxy: invalid wildcard label suffix in %q", raw)
		}
		return nil
	}

	if net.ParseIP(p) != nil || isValidDNSName(p) {
		return nil
	}
	return fmt.Errorf("oauthproxy: invalid allowlist host %q", raw)
}

func matchAllowedHost(host, pattern string) bool {
	if err := ValidateAllowedPattern(pattern); err != nil {
		return false
	}
	pattern = strings.ToLower(strings.TrimSpace(pattern))
	if strings.HasPrefix(pattern, ".") {
		if net.ParseIP(host) != nil {
			return false
		}
		bare := pattern[1:]
		return host == bare || strings.HasSuffix(host, pattern)
	}
	if strings.HasPrefix(pattern, "*") {
		if net.ParseIP(host) != nil {
			return false
		}
		rest := pattern[1:]
		if !strings.HasSuffix(host, rest) {
			return false
		}
		prefix := strings.TrimSuffix(host, rest)
		return prefix != "" && !strings.Contains(prefix, ".")
	}
	return host == pattern
}

func isASCII(s string) bool {
	for i := 0; i < len(s); i++ {
		if s[i] >= 0x80 {
			return false
		}
	}
	return true
}

func isValidDNSName(host string) bool {
	if len(host) == 0 || len(host) > 253 {
		return false
	}
	for _, label := range strings.Split(host, ".") {
		if !isValidDNSLabel(label) {
			return false
		}
	}
	return true
}

func isValidDNSLabel(label string) bool {
	if len(label) == 0 || len(label) > 63 {
		return false
	}
	if label[0] == '-' || label[len(label)-1] == '-' {
		return false
	}
	for i := 0; i < len(label); i++ {
		if !isDNSLabelChar(label[i]) {
			return false
		}
	}
	return true
}

// isValidWildcardLabelSuffix validates the literal suffix of a leftmost DNS label following "*"
// (e.g. "-tavern-749076704275"). It may begin with "-", must leave room for at least one leading
// character in the 63-char DNS label limit, and cannot end with "-".
func isValidWildcardLabelSuffix(suffix string) bool {
	if len(suffix) == 0 || len(suffix) > 62 {
		return false
	}
	if suffix[len(suffix)-1] == '-' {
		return false
	}
	for i := 0; i < len(suffix); i++ {
		if !isDNSLabelChar(suffix[i]) {
			return false
		}
	}
	return true
}

func isDNSLabelChar(c byte) bool {
	return (c >= 'a' && c <= 'z') || (c >= '0' && c <= '9') || c == '-'
}

// isLoopbackHost reports whether host is a well-known loopback address/name.
func isLoopbackHost(host string) bool {
	switch host {
	case "localhost", "127.0.0.1", "::1":
		return true
	default:
		return false
	}
}
