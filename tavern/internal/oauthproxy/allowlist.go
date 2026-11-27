package oauthproxy

import (
	"fmt"
	"net/netip"
	"net/url"
	"strconv"
	"strings"
)

// CompletePath is the only path on a tenant that may receive an assertion.
const CompletePath = "/oauth/complete"

type patternKind uint8

const (
	patternExactHost patternKind = iota
	patternExactIP
	patternDomainSuffix
	patternLeadingWildcard
)

type allowedPattern struct {
	kind  patternKind
	value string
	bare  string // non-empty for patternDomainSuffix (value without leading ".")
}

// Allowlist is a pre-validated, compiled set of allowed return_to host patterns.
type Allowlist struct {
	patterns []allowedPattern
}

// NewAllowlist parses and validates every entry in patterns, returning a compiled Allowlist that
// can be reused across requests without re-parsing patterns.
func NewAllowlist(patterns []string) (*Allowlist, error) {
	compiled := make([]allowedPattern, 0, len(patterns))
	for _, raw := range patterns {
		p, err := parseAllowedPattern(raw)
		if err != nil {
			return nil, err
		}
		compiled = append(compiled, p)
	}
	return &Allowlist{patterns: compiled}, nil
}

// IsAllowedReturnTo reports whether returnTo is allowed by the compiled allowlist.
func (a *Allowlist) IsAllowedReturnTo(returnTo string) bool {
	if a == nil {
		return false
	}
	host, isIP, ok := parseReturnToHost(returnTo)
	if !ok {
		return false
	}
	for _, p := range a.patterns {
		if p.matches(host, isIP) {
			return true
		}
	}
	return false
}

// IsAllowedReturnTo reports whether returnTo is a URL whose host matches one of the allowed
// patterns. A pattern is one of:
//   - an exact host (e.g. "tavern.example.com", "localhost", or an IP literal like "127.0.0.1"),
//   - a dot-prefixed multi-label domain suffix (e.g. ".example.com", matching any subdomain of
//     example.com as well as example.com itself), or
//   - a leading-wildcard pattern over a multi-label domain (e.g.
//     "*-tavern-749076704275.us-east4.run.app" or "*.example.com"), where "*" matches a non-empty
//     prefix within the leftmost DNS label (no "." characters).
//
// returnTo's host must be an ASCII IP literal (with brackets required for and restricted to IPv6)
// or valid ASCII DNS hostname (rejecting non-ASCII Unicode dot separators like
// U+3002/U+FF0E/U+FF61, all-numeric TLDs / partial or octal IPv4 forms, and invalid label syntax).
// Any explicit port must be in 1..65535.
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
	host, isIP, ok := parseReturnToHost(returnTo)
	if !ok {
		return false
	}

	for _, raw := range allowedSuffixes {
		p, err := parseAllowedPattern(raw)
		if err != nil {
			continue
		}
		if p.matches(host, isIP) {
			return true
		}
	}

	return false
}

// ValidateAllowedPattern reports whether pattern is a syntactically valid host allowlist entry:
// an exact ASCII DNS hostname or IP literal, a dot-prefixed multi-label ASCII domain suffix (e.g.
// ".example.com"), or a leading-wildcard single-label pattern over a multi-label domain (e.g.
// "*-tavern-749076704275.us-east4.run.app" or "*.example.com").
func ValidateAllowedPattern(pattern string) error {
	_, err := parseAllowedPattern(pattern)
	return err
}

func parseReturnToHost(returnTo string) (host string, isIP bool, ok bool) {
	u, err := url.Parse(returnTo)
	if err != nil {
		return "", false, false
	}

	// Only Tavern's own completion endpoint may receive an assertion: no other path, no
	// userinfo, and no query or fragment that could corrupt or hide the appended assertion.
	if u.Path != CompletePath || u.User != nil || u.RawQuery != "" || u.Fragment != "" || u.Opaque != "" {
		return "", false, false
	}

	rawHost := u.Hostname()
	if rawHost == "" || !isASCII(rawHost) {
		return "", false, false
	}

	// Reject empty ports (e.g. "host:" or "[::1]:") and out-of-range ports (0 or >65535).
	if strings.HasSuffix(u.Host, ":") {
		return "", false, false
	}
	if portStr := u.Port(); portStr != "" {
		port, err := strconv.ParseUint(portStr, 10, 16)
		if err != nil || port == 0 {
			return "", false, false
		}
	}

	bracketed := strings.HasPrefix(u.Host, "[")
	if addr, err := netip.ParseAddr(rawHost); err == nil {
		if addr.Zone() != "" || bracketed != addr.Is6() {
			return "", false, false
		}
		host = addr.String()
		isIP = true
	} else {
		if bracketed {
			return "", false, false
		}
		host = strings.ToLower(rawHost)
		if !isValidDNSName(host) {
			return "", false, false
		}
	}

	if u.Scheme != "https" {
		if u.Scheme != "http" || !isLoopbackHost(host) {
			return "", false, false
		}
	}

	return host, isIP, true
}

func parseAllowedPattern(pattern string) (allowedPattern, error) {
	raw := strings.TrimSpace(pattern)
	if raw == "" {
		return allowedPattern{}, fmt.Errorf("oauthproxy: empty allowlist pattern")
	}
	if !isASCII(raw) {
		return allowedPattern{}, fmt.Errorf("oauthproxy: allowlist pattern %q must contain only ASCII characters", raw)
	}
	p := strings.ToLower(raw)

	if strings.HasPrefix(p, ".") {
		bare := p[1:]
		if !isValidMultiLabelDNSName(bare) {
			return allowedPattern{}, fmt.Errorf("oauthproxy: invalid dot-prefixed allowlist suffix %q", raw)
		}
		return allowedPattern{kind: patternDomainSuffix, value: p, bare: bare}, nil
	}

	if strings.HasPrefix(p, "*") {
		rest := p[1:]
		if rest == "" || strings.Contains(rest, "*") || len(p) > 253 {
			return allowedPattern{}, fmt.Errorf("oauthproxy: invalid wildcard allowlist pattern %q", raw)
		}
		if strings.HasPrefix(rest, ".") {
			domain := rest[1:]
			if !isValidMultiLabelDNSName(domain) {
				return allowedPattern{}, fmt.Errorf("oauthproxy: invalid wildcard allowlist pattern %q", raw)
			}
			return allowedPattern{kind: patternLeadingWildcard, value: rest}, nil
		}
		labelSuffix, domain, ok := strings.Cut(rest, ".")
		if !ok || !isValidMultiLabelDNSName(domain) {
			return allowedPattern{}, fmt.Errorf("oauthproxy: invalid wildcard allowlist pattern %q", raw)
		}
		if !isValidWildcardLabelSuffix(labelSuffix) {
			return allowedPattern{}, fmt.Errorf("oauthproxy: invalid wildcard label suffix in %q", raw)
		}
		return allowedPattern{kind: patternLeadingWildcard, value: rest}, nil
	}

	if addr, err := netip.ParseAddr(p); err == nil {
		if addr.Zone() != "" {
			return allowedPattern{}, fmt.Errorf("oauthproxy: invalid allowlist host %q", raw)
		}
		return allowedPattern{kind: patternExactIP, value: addr.String()}, nil
	}
	if isValidDNSName(p) {
		return allowedPattern{kind: patternExactHost, value: p}, nil
	}
	return allowedPattern{}, fmt.Errorf("oauthproxy: invalid allowlist host %q", raw)
}

func (p allowedPattern) matches(host string, isIP bool) bool {
	switch p.kind {
	case patternExactIP:
		return isIP && host == p.value
	case patternExactHost:
		return !isIP && host == p.value
	case patternDomainSuffix:
		if isIP {
			return false
		}
		return host == p.bare || strings.HasSuffix(host, p.value)
	case patternLeadingWildcard:
		if isIP || !strings.HasSuffix(host, p.value) {
			return false
		}
		prefix := strings.TrimSuffix(host, p.value)
		return prefix != "" && !strings.Contains(prefix, ".")
	default:
		return false
	}
}

func isASCII(s string) bool {
	for i := 0; i < len(s); i++ {
		if s[i] >= 0x80 {
			return false
		}
	}
	return true
}

func isValidMultiLabelDNSName(host string) bool {
	return strings.Contains(host, ".") && isValidDNSName(host)
}

func isValidDNSName(host string) bool {
	if len(host) == 0 || len(host) > 253 {
		return false
	}
	labels := strings.Split(host, ".")
	for _, label := range labels {
		if !isValidDNSLabel(label) {
			return false
		}
	}
	// Reject all-numeric rightmost labels (RFC 1123 §2.1 / RFC 3696 §2) so partial, octal, or
	// decimal IPv4 forms (e.g. "0177.0.0.1", "0.0.1", "2130706433") cannot pass as DNS names.
	return !isAllDigits(labels[len(labels)-1])
}

func isAllDigits(s string) bool {
	if len(s) == 0 {
		return false
	}
	for i := 0; i < len(s); i++ {
		if s[i] < '0' || s[i] > '9' {
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
