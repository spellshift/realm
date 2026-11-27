package oauthproxy

import "testing"

func TestNormalizeBaseURL(t *testing.T) {
	tests := []struct {
		in      string
		want    string
		wantErr bool
	}{
		{in: "tavern.example.com", want: "https://tavern.example.com"},
		{in: "https://tavern.example.com/", want: "https://tavern.example.com"},
		{in: "http://localhost:8000/", want: "http://localhost:8000"},
		{in: "  https://proxy.run.app  ", want: "https://proxy.run.app"},
		// A host that merely starts with "http" must still get a scheme.
		{in: "http-proxy.example.com", want: "https://http-proxy.example.com"},
		{in: "", wantErr: true},
		{in: "ftp://tavern.example.com", wantErr: true},
		{in: "https://", wantErr: true},
		{in: "https://tavern.example.com/?a=b", wantErr: true},
		{in: "https://tavern.example.com/#frag", wantErr: true},
		{in: "https://user:pass@tavern.example.com", wantErr: true},
		{in: "https://tavern.example.com/subpath", wantErr: true},
		{in: "https://tavern.example.com/a/b/", wantErr: true},
	}
	for _, tt := range tests {
		got, err := NormalizeBaseURL(tt.in)
		if (err != nil) != tt.wantErr {
			t.Errorf("NormalizeBaseURL(%q) error = %v, wantErr %v", tt.in, err, tt.wantErr)
			continue
		}
		if got != tt.want {
			t.Errorf("NormalizeBaseURL(%q) = %q, want %q", tt.in, got, tt.want)
		}
	}
}

func TestRequireSecureURL(t *testing.T) {
	tests := []struct {
		in      string
		wantErr bool
	}{
		{in: "https://proxy.run.app"},
		{in: "http://localhost:8080"},
		{in: "http://127.0.0.1:8080"},
		{in: "http://[::1]:8080"},
		{in: "http://proxy.run.app", wantErr: true},
		{in: "http://localhost.evil.com", wantErr: true},
		{in: "ftp://proxy.run.app", wantErr: true},
	}
	for _, tt := range tests {
		if err := RequireSecureURL(tt.in); (err != nil) != tt.wantErr {
			t.Errorf("RequireSecureURL(%q) error = %v, wantErr %v", tt.in, err, tt.wantErr)
		}
	}
}
