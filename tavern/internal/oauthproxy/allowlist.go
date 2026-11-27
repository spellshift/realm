package oauthproxy

import (
	"net/url"
	"strings"
)

// CompletePath is the only path on a tenant that may receive an assertion.
const CompletePath = "/oauth/complete"

// IsAllowedReturnTo reports whether returnTo is a URL whose host matches one of the allowed
// suffixes. A suffix is either an exact host (e.g. "tavern.example.com") or a dot-prefixed
// domain suffix (e.g. ".run.app", matching any subdomain of run.app as well as run.app itself).
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

	for _, suffix := range allowedSuffixes {
		suffix = strings.ToLower(strings.TrimSpace(suffix))
		if suffix == "" {
			continue
		}
		if strings.HasPrefix(suffix, ".") {
			bare := suffix[1:]
			if host == bare || strings.HasSuffix(host, suffix) {
				return true
			}
			continue
		}
		if host == suffix {
			return true
		}
	}

	return false
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
