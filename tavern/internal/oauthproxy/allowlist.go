package oauthproxy

import (
	"net/url"
	"strings"
)

// CompletePath is the only path on a tenant that may receive an assertion.
const CompletePath = "/oauth/complete"

// IsAllowedReturnTo reports whether returnTo is a URL whose host matches one of the allowed
// patterns. A pattern is one of:
//   - an exact host (e.g. "tavern.example.com"),
//   - a dot-prefixed domain suffix (e.g. ".example.com", matching any subdomain of example.com as
//     well as example.com itself), or
//   - a leading-wildcard pattern (e.g. "*-tavern-749076704275.us-east4.run.app" or
//     "*.example.com"), where "*" matches a non-empty prefix within the leftmost DNS label (no "."
//     characters).
//
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

	host := strings.ToLower(u.Hostname())
	if host == "" {
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

func matchAllowedHost(host, pattern string) bool {
	pattern = strings.ToLower(strings.TrimSpace(pattern))
	if pattern == "" {
		return false
	}
	if strings.HasPrefix(pattern, ".") {
		bare := pattern[1:]
		return bare != "" && (host == bare || strings.HasSuffix(host, pattern))
	}
	if strings.HasPrefix(pattern, "*") {
		rest := pattern[1:]
		if rest == "" || strings.Contains(rest, "*") || !strings.Contains(rest, ".") || strings.HasSuffix(rest, ".") {
			return false
		}
		if !strings.HasSuffix(host, rest) {
			return false
		}
		prefix := strings.TrimSuffix(host, rest)
		return prefix != "" && !strings.Contains(prefix, ".")
	}
	return host == pattern
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
