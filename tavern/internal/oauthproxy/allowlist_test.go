package oauthproxy

import (
	"strings"
	"testing"
)

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
	suffixes := []string{"localhost", "127.0.0.1", "::1"}

	tests := []struct {
		name     string
		returnTo string
		want     bool
	}{
		{"http localhost allowed", "http://localhost:8000/oauth/complete", true},
		{"http 127.0.0.1 allowed", "http://127.0.0.1:8000/oauth/complete", true},
		{"http ipv6 loopback allowed", "http://[::1]:8000/oauth/complete", true},
		{"http ipv6 loopback without port allowed", "http://[::1]/oauth/complete", true},
		{"http non-canonical ipv6 loopback canonicalized", "http://[0:0:0:0:0:0:0:1]:8000/oauth/complete", true},
		{"https localhost still allowed", "https://localhost:8000/oauth/complete", true},
		{"http non-loopback host rejected", "http://tavern.example.com/oauth/complete", false},
		{"bracketed ipv4 rejected", "http://[127.0.0.1]/oauth/complete", false},
		{"bracketed hostname rejected", "http://[localhost]/oauth/complete", false},
		{"empty port on ipv6 rejected", "http://[::1]:/oauth/complete", false},
		{"out-of-range port rejected", "http://localhost:99999/oauth/complete", false},
		{"zero port rejected", "http://localhost:0/oauth/complete", false},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			got := IsAllowedReturnTo(tc.returnTo, suffixes)
			if got != tc.want {
				t.Errorf("IsAllowedReturnTo(%q) = %v, want %v", tc.returnTo, got, tc.want)
			}
		})
	}

	// Non-canonical IPv6 pattern should also canonicalize to match ::1.
	if !IsAllowedReturnTo("http://[::1]:8000/oauth/complete", []string{"0:0:0:0:0:0:0:1"}) {
		t.Error("expected canonicalized IPv6 pattern to match [::1]")
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
		{"trailing dot rejected against valid pattern", "https://abc123-tavern-749076704275.us-east4.run.app./oauth/complete", false},
		{"http scheme rejected", "http://abc123-tavern-749076704275.us-east4.run.app/oauth/complete", false},
		{"unicode ideographic dot U+3002 rejected", "https://a\u3002b-tavern-749076704275.us-east4.run.app/oauth/complete", false},
		{"unicode fullwidth dot U+FF0E rejected", "https://a\uff0eb-tavern-749076704275.us-east4.run.app/oauth/complete", false},
		{"unicode halfwidth dot U+FF61 rejected", "https://a\uff61b-tavern-749076704275.us-east4.run.app/oauth/complete", false},
		{"underscore in wildcard prefix rejected", "https://_-tavern-749076704275.us-east4.run.app/oauth/complete", false},
		{"leading hyphen in wildcard prefix rejected", "https://-a-tavern-749076704275.us-east4.run.app/oauth/complete", false},
		{"percent-encoded dot in wildcard prefix rejected", "https://a%2eb-tavern-749076704275.us-east4.run.app/oauth/complete", false},
		{"bracketed dns host rejected", "https://[abc123-tavern-749076704275.us-east4.run.app]/oauth/complete", false},
		{"empty port rejected", "https://abc123-tavern-749076704275.us-east4.run.app:/oauth/complete", false},
		{"out-of-range port rejected", "https://abc123-tavern-749076704275.us-east4.run.app:99999/oauth/complete", false},
		{"zero port rejected", "https://abc123-tavern-749076704275.us-east4.run.app:0/oauth/complete", false},
		{"valid explicit port accepted", "https://abc123-tavern-749076704275.us-east4.run.app:8443/oauth/complete", true},
		{"oversized 64-char leftmost DNS label rejected", "https://" + strings.Repeat("a", 44) + "-tavern-749076704275.us-east4.run.app/oauth/complete", false},
		{"max 63-char leftmost DNS label accepted", "https://" + strings.Repeat("a", 43) + "-tavern-749076704275.us-east4.run.app/oauth/complete", true},
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
	if !IsAllowedReturnTo("https://sub.example.com/oauth/complete", []string{"*.EXAMPLE.COM"}) {
		t.Error("expected mixed-case wildcard pattern to match case-insensitively")
	}
	if !IsAllowedReturnTo("https://sub.example.com/oauth/complete", []string{".Example.COM"}) {
		t.Error("expected mixed-case dot-prefix pattern to match case-insensitively")
	}
	if !IsAllowedReturnTo("https://tavern.example.com/oauth/complete", []string{"Tavern.Example.COM"}) {
		t.Error("expected mixed-case exact pattern to match case-insensitively")
	}
	if IsAllowedReturnTo("https://a.b.example.com/oauth/complete", []string{"*.example.com"}) {
		t.Error("expected nested subdomain to be rejected by *.example.com")
	}
	if IsAllowedReturnTo("https://a\u3002b.example.com/oauth/complete", []string{"*.example.com"}) {
		t.Error("expected unicode dot separator to be rejected by *.example.com")
	}
	if IsAllowedReturnTo("https://example.com/oauth/complete", []string{"*.example.com"}) {
		t.Error("expected bare domain to be rejected by *.example.com")
	}
	if IsAllowedReturnTo("https://sub.example.com./oauth/complete", []string{"*.example.com"}) {
		t.Error("expected trailing dot host to be rejected by *.example.com")
	}
	if IsAllowedReturnTo("https://0177.0.0.1/oauth/complete", []string{"*.0.0.1"}) {
		t.Error("expected octal IPv4 host to be rejected against IP-like wildcard")
	}
	if IsAllowedReturnTo("https://127.0.0.1/oauth/complete", []string{".0.0.1", "*.0.0.1"}) {
		t.Error("expected IPv4 literal to be rejected against suffix/wildcard patterns")
	}
	if IsAllowedReturnTo("http://[::1]/oauth/complete", []string{".example.com", "*.example.com"}) {
		t.Error("expected IPv6 literal to be rejected against DNS suffix/wildcard patterns")
	}

	invalidPatterns := []string{"*", "*.", "*com", "**-tavern-749076704275.us-east4.run.app", ".", ".com", "*.com", "*.0.0.1"}
	for _, pat := range invalidPatterns {
		if IsAllowedReturnTo("https://abc123-tavern-749076704275.us-east4.run.app/oauth/complete", []string{pat}) {
			t.Errorf("expected invalid/overbroad pattern %q to be rejected", pat)
		}
		if IsAllowedReturnTo("https://anything./oauth/complete", []string{pat}) {
			t.Errorf("expected trailing-dot host to be rejected for pattern %q", pat)
		}
	}
}

func TestIsAllowedReturnToMixedPatternsAndCompiledAllowlist(t *testing.T) {
	patterns := []string{
		"*-TAVERN-749076704275.US-EAST4.RUN.APP",
		".internal.example.com",
		"tavern.custom.net",
		"localhost",
	}

	compiled, err := NewAllowlist(patterns)
	if err != nil {
		t.Fatalf("NewAllowlist returned unexpected error: %v", err)
	}

	tests := []struct {
		returnTo string
		want     bool
	}{
		{"https://abc123-tavern-749076704275.us-east4.run.app/oauth/complete", true},
		{"https://dev.internal.example.com/oauth/complete", true},
		{"https://internal.example.com/oauth/complete", true},
		{"https://tavern.custom.net/oauth/complete", true},
		{"http://localhost:8000/oauth/complete", true},
		{"https://other.run.app/oauth/complete", false},
		{"https://sub.tavern.custom.net/oauth/complete", false},
	}

	for _, tc := range tests {
		if got := IsAllowedReturnTo(tc.returnTo, patterns); got != tc.want {
			t.Errorf("IsAllowedReturnTo(%q) = %v, want %v", tc.returnTo, got, tc.want)
		}
		if got := compiled.IsAllowedReturnTo(tc.returnTo); got != tc.want {
			t.Errorf("Allowlist.IsAllowedReturnTo(%q) = %v, want %v", tc.returnTo, got, tc.want)
		}
	}

	var nilAllowlist *Allowlist
	if nilAllowlist.IsAllowedReturnTo("https://tavern.custom.net/oauth/complete") {
		t.Error("expected nil Allowlist to reject all URLs")
	}
	if _, err := NewAllowlist([]string{"tavern.example.com", "*"}); err == nil {
		t.Error("expected NewAllowlist to fail when any pattern is invalid")
	}
}

func TestValidateAllowedPattern(t *testing.T) {
	// Build boundary-length valid DNS domains: 4 labels of 60 + "." + "com" => 247 chars, etc.
	// 251-char domain: 4*(60+1) + 7 = 251 -> "*." + 251 = 253 chars (exact max).
	domain251 := strings.Repeat(strings.Repeat("a", 60)+".", 4) + "abc.com" // 244 + 7 = 251
	// 252-char domain: "*." + 252 = 254 chars (>253, dead no-op).
	domain252 := strings.Repeat(strings.Repeat("a", 60)+".", 4) + "abcd.com" // 244 + 8 = 252
	// 253-char exact DNS name: 4*(60+1) + 9 = 253.
	host253 := strings.Repeat(strings.Repeat("a", 60)+".", 4) + "abcde.com"  // 244 + 9 = 253
	host254 := strings.Repeat(strings.Repeat("a", 60)+".", 4) + "abcdef.com" // 244 + 10 = 254

	suffix62 := "-" + strings.Repeat("a", 61) // len 62 (max valid wildcard label suffix)
	suffix63 := "-" + strings.Repeat("a", 62) // len 63 (leaves 0 chars for "*")

	valid := []string{
		"*-tavern-749076704275.us-east4.run.app",
		"*-TAVERN-749076704275.US-EAST4.RUN.APP",
		"*.example.com",
		".example.com",
		".run.app",
		"tavern.example.com",
		"xn--bcher-kva.example",
		"localhost",
		"127.0.0.1",
		"::1",
		"0:0:0:0:0:0:0:1",
		"  *-tavern-749076704275.us-east4.run.app  ",
		"*" + suffix62 + ".example.com",
		"*." + domain251,
		host253,
	}
	for _, pat := range valid {
		if err := ValidateAllowedPattern(pat); err != nil {
			t.Errorf("ValidateAllowedPattern(%q) returned unexpected error: %v", pat, err)
		}
	}

	invalid := []string{
		"",
		"   ",
		"*",
		"*.",
		"*com",
		"*.com",
		"*.app",
		"*-tavern.app",
		"**-tavern-749076704275.us-east4.run.app",
		"*.*.example.com",
		"foo.*.example.com",
		"*-tavern-.us-east4.run.app",
		"*..example.com",
		"*" + suffix63 + ".example.com",
		"*." + domain252,
		host254,
		".",
		".com",
		".app",
		"..example.com",
		".example.com.",
		"tavern.example.com.",
		"https://tavern.example.com",
		"tavern.example.com/oauth/complete",
		"tavern.example.com:8000",
		"a\u3002b.example.com",
		"*.127.0.0.1",
		".127.0.0.1",
		"*.0.0.1",
		".10.0",
		"*.192.168.1",
		".0177.0.0.1",
		"0177.0.0.1",
		"127.0.1",
		"2130706433",
		"[::1]",
		"fe80::1%eth0",
	}
	for _, pat := range invalid {
		if err := ValidateAllowedPattern(pat); err == nil {
			t.Errorf("ValidateAllowedPattern(%q) = nil, want error", pat)
		}
	}

	// Verify >253-char hostnames are rejected at runtime too.
	if IsAllowedReturnTo("https://"+host254+"/oauth/complete", []string{".com", ".example.com", "*." + domain251}) {
		t.Error("expected >253-char hostname to be rejected")
	}
	if !IsAllowedReturnTo("https://a."+domain251+"/oauth/complete", []string{"*." + domain251}) {
		t.Error("expected exact 253-char wildcard match to be accepted")
	}
}
