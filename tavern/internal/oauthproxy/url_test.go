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
