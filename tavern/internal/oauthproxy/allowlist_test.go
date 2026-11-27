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

func TestIsAllowedReturnToLeadingWildcard(t *testing.T) {
	patterns := []string{"*-tavern-749076704275.us-east4.run.app"}

	tests := []struct {
		name     string
		returnTo string
		want     bool
	}{
		{"six-char prefix matches", "https://abc123-tavern-749076704275.us-east4.run.app/oauth/complete", true},
		{"default tavern prefix matches", "https://tavern-tavern-749076704275.us-east4.run.app/oauth/complete", true},
		{"hyphenated prefix matches", "https://my-dev-tavern-749076704275.us-east4.run.app/oauth/complete", true},
		{"uppercase host matches case-insensitively", "https://ABC123-TAVERN-749076704275.US-EAST4.RUN.APP/oauth/complete", true},
		{"empty wildcard prefix rejected", "https://-tavern-749076704275.us-east4.run.app/oauth/complete", false},
		{"missing prefix and hyphen rejected", "https://tavern-749076704275.us-east4.run.app/oauth/complete", false},
		{"multi-label wildcard match rejected", "https://sub.abc123-tavern-749076704275.us-east4.run.app/oauth/complete", false},
		{"different project id rejected", "https://abc123-tavern-999999999999.us-east4.run.app/oauth/complete", false},
		{"different region rejected", "https://abc123-tavern-749076704275.us-central1.run.app/oauth/complete", false},
		{"different service name rejected", "https://abc123-other-749076704275.us-east4.run.app/oauth/complete", false},
		{"arbitrary run.app host rejected", "https://attacker.run.app/oauth/complete", false},
		{"bare run.app rejected", "https://run.app/oauth/complete", false},
		{"lookalike suffix rejected", "https://abc123-tavern-749076704275.us-east4.run.app.attacker.net/oauth/complete", false},
		{"http scheme rejected", "http://abc123-tavern-749076704275.us-east4.run.app/oauth/complete", false},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			got := IsAllowedReturnTo(tc.returnTo, patterns)
			if got != tc.want {
				t.Errorf("IsAllowedReturnTo(%q) = %v, want %v", tc.returnTo, got, tc.want)
			}
		})
	}
}

func TestIsAllowedReturnToWildcardSubdomainAndInvalidPatterns(t *testing.T) {
	if !IsAllowedReturnTo("https://sub.example.com/oauth/complete", []string{"*.example.com"}) {
		t.Error("expected single-label subdomain to match *.example.com")
	}
	if IsAllowedReturnTo("https://a.b.example.com/oauth/complete", []string{"*.example.com"}) {
		t.Error("expected nested subdomain to be rejected by *.example.com")
	}
	if IsAllowedReturnTo("https://example.com/oauth/complete", []string{"*.example.com"}) {
		t.Error("expected bare domain to be rejected by *.example.com")
	}

	invalidPatterns := []string{"*", "*.", "*com", "**-tavern-749076704275.us-east4.run.app", "."}
	for _, pat := range invalidPatterns {
		if IsAllowedReturnTo("https://abc123-tavern-749076704275.us-east4.run.app/oauth/complete", []string{pat}) {
			t.Errorf("expected invalid/overbroad pattern %q to be rejected", pat)
		}
		if IsAllowedReturnTo("https://anything./oauth/complete", []string{pat}) {
			t.Errorf("expected trailing-dot host to be rejected for pattern %q", pat)
		}
	}
}
