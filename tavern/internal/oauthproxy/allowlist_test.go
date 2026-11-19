package oauthproxy

import "testing"

func TestIsAllowedReturnTo(t *testing.T) {
	suffixes := []string{".run.app", "tavern.example.com"}

	tests := []struct {
		name     string
		returnTo string
		want     bool
	}{
		{"run.app subdomain", "https://abc123-tavern-456.us-central1.run.app/oauth/complete", true},
		{"bare run.app", "https://run.app/oauth/complete", true},
		{"exact custom domain", "https://tavern.example.com/oauth/complete", true},
		{"unrelated domain", "https://attacker.example.net/oauth/complete", false},
		{"lookalike suffix", "https://evil-run.app.attacker.net/oauth/complete", false},
		{"http scheme rejected", "http://abc123-tavern-456.us-central1.run.app/oauth/complete", false},
		{"subdomain of custom domain not allowed unless suffix is dot-prefixed", "https://sub.tavern.example.com/oauth/complete", false},
		{"malformed url", "https://[::not-valid", false},
		{"empty", "", false},
		{"wrong path", "https://abc123.us-central1.run.app/other", false},
		{"root path", "https://abc123.us-central1.run.app/", false},
		{"path prefix trick", "https://abc123.us-central1.run.app/oauth/complete/../x", false},
		{"query string", "https://abc123.us-central1.run.app/oauth/complete?a=b", false},
		{"fragment", "https://abc123.us-central1.run.app/oauth/complete#frag", false},
		{"userinfo", "https://tavern.example.com@evil.example.net/oauth/complete", false},
		{"userinfo on allowed host", "https://user@abc123.us-central1.run.app/oauth/complete", false},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			got := IsAllowedReturnTo(tc.returnTo, suffixes)
			if got != tc.want {
				t.Errorf("IsAllowedReturnTo(%q) = %v, want %v", tc.returnTo, got, tc.want)
			}
		})
	}
}

func TestIsAllowedReturnToAllowsLoopbackOverHTTP(t *testing.T) {
	suffixes := []string{"localhost", "127.0.0.1"}

	tests := []struct {
		name     string
		returnTo string
		want     bool
	}{
		{"http localhost allowed", "http://localhost:8000/oauth/complete", true},
		{"http 127.0.0.1 allowed", "http://127.0.0.1:8000/oauth/complete", true},
		{"https localhost still allowed", "https://localhost:8000/oauth/complete", true},
		{"http non-loopback host rejected", "http://tavern.example.com/oauth/complete", false},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			got := IsAllowedReturnTo(tc.returnTo, suffixes)
			if got != tc.want {
				t.Errorf("IsAllowedReturnTo(%q) = %v, want %v", tc.returnTo, got, tc.want)
			}
		})
	}
}

func TestIsAllowedReturnToDotPrefixMatchesSubdomains(t *testing.T) {
	suffixes := []string{".example.com"}

	if !IsAllowedReturnTo("https://example.com/oauth/complete", suffixes) {
		t.Error("expected bare domain to match dot-prefixed suffix")
	}
	if !IsAllowedReturnTo("https://sub.example.com/oauth/complete", suffixes) {
		t.Error("expected subdomain to match dot-prefixed suffix")
	}
	if IsAllowedReturnTo("https://notexample.com/oauth/complete", suffixes) {
		t.Error("expected lookalike domain to be rejected")
	}
}
