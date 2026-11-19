package oauthproxy

import (
	"fmt"
	"net/url"
	"strings"
)

// NormalizeBaseURL parses raw as a base URL for a service, defaulting to https:// when no scheme
// is given. It rejects anything other than http/https, URLs without a host, and URLs carrying a
// query or fragment, and strips any trailing slash so callers can safely append a path.
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
	if u.Hostname() == "" {
		return "", fmt.Errorf("oauthproxy: url %q has no host", raw)
	}
	if u.RawQuery != "" || u.Fragment != "" {
		return "", fmt.Errorf("oauthproxy: url %q must not contain a query or fragment", raw)
	}

	u.Path = strings.TrimRight(u.Path, "/")
	u.RawPath = ""
	return u.String(), nil
}
