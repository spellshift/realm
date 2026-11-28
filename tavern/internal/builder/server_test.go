package builder

import (
	"strings"
	"testing"
	"unicode/utf8"
)

func TestSanitizeBuilderHostname(t *testing.T) {
	tests := []struct {
		name string
		raw  string
		want string
	}{
		{name: "empty", raw: "", want: ""},
		{name: "whitespace only", raw: " \t\n ", want: ""},
		{name: "trims surrounding whitespace", raw: "  builder-1.example  ", want: "builder-1.example"},
		{name: "exact max length", raw: strings.Repeat("a", maxBuilderHostnameLen), want: strings.Repeat("a", maxBuilderHostnameLen)},
		{name: "truncates oversized ASCII", raw: strings.Repeat("a", maxBuilderHostnameLen+50), want: strings.Repeat("a", maxBuilderHostnameLen)},
		{
			name: "truncates on valid UTF-8 boundary",
			raw:  strings.Repeat("a", maxBuilderHostnameLen-1) + "é",
			want: strings.Repeat("a", maxBuilderHostnameLen-1),
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			got := sanitizeBuilderHostname(tc.raw)
			if got != tc.want {
				t.Fatalf("sanitizeBuilderHostname(%q) = %q (len %d), want %q (len %d)", tc.raw, got, len(got), tc.want, len(tc.want))
			}
			if !utf8.ValidString(got) {
				t.Fatalf("sanitizeBuilderHostname returned invalid UTF-8: %q", got)
			}
			if len(got) > maxBuilderHostnameLen {
				t.Fatalf("sanitizeBuilderHostname len = %d, want <= %d", len(got), maxBuilderHostnameLen)
			}
		})
	}
}
