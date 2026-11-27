package oauthproxy

import (
	"fmt"
	"net/url"
	"strings"
)

// NormalizeBaseURL parses raw as a base URL for a service, defaulting to https:// when no scheme
// is given. It rejects anything other than http/https, URLs without a host, and URLs carrying
// userinfo, a non-empty path, a query, or a fragment, and strips any trailing slash so callers
// can safely append a path.
func NormalizeBaseURL(raw string) (string, error) {
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return "", fmt.Errorf("oauthproxy: empty url")
	}
	if !strings.Contains(raw, "://") {
		raw = "https://" + raw
	}

	u, err := url.Parse(raw)
	if err != nil {
		return "", fmt.Errorf("oauthproxy: invalid url %q: %w", raw, err)
	}
	if u.Scheme != "http" && u.Scheme != "https" {
		return "", fmt.Errorf("oauthproxy: url %q must use http or https", raw)
	}
	if u.User != nil {
		return "", fmt.Errorf("oauthproxy: url %q must not contain userinfo", raw)
	}
	if u.Hostname() == "" || u.Opaque != "" {
		return "", fmt.Errorf("oauthproxy: url %q has no host", raw)
	}
	if strings.TrimRight(u.Path, "/") != "" {
		return "", fmt.Errorf("oauthproxy: url %q must not contain a path", raw)
	}
	if u.RawQuery != "" || u.Fragment != "" {
		return "", fmt.Errorf("oauthproxy: url %q must not contain a query or fragment", raw)
	}

	u.Path = ""
	u.RawPath = ""
	return u.String(), nil
}

// RequireSecureURL returns an error unless raw uses https, or http with a loopback host (so a
// local development setup works without TLS). It is meant for URLs whose responses are trusted,
// such as the JWKS endpoint, where plaintext HTTP would let a man-in-the-middle supply the key.
func RequireSecureURL(raw string) error {
	u, err := url.Parse(raw)
	if err != nil {
		return fmt.Errorf("oauthproxy: invalid url %q: %w", raw, err)
	}
	switch {
	case u.Scheme == "https":
		return nil
	case u.Scheme == "http" && isLoopbackHost(strings.ToLower(u.Hostname())):
		return nil
	default:
		return fmt.Errorf("oauthproxy: url %q must use https (http is only allowed for loopback hosts)", raw)
	}
}
