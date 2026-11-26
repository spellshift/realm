package main

import (
	"context"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/metadata"
	"realm.pub/tavern/cli/auth"
)

func TestNormalizeUpstreamURL(t *testing.T) {
	tests := []struct {
		name     string
		upstream string
		want     string
	}{
		{
			name:     "schemeless_ipv4_loopback",
			upstream: "127.0.0.1:8000",
			want:     "http://127.0.0.1:8000",
		},
		{
			name:     "schemeless_ipv4_loopback_range",
			upstream: "127.0.0.2:8000",
			want:     "http://127.0.0.2:8000",
		},
		{
			name:     "schemeless_localhost_with_port",
			upstream: "localhost:8000",
			want:     "http://localhost:8000",
		},
		{
			name:     "schemeless_localhost_uppercase",
			upstream: "LOCALHOST:8000",
			want:     "http://LOCALHOST:8000",
		},
		{
			name:     "schemeless_ipv6_loopback_bracketed_port",
			upstream: "[::1]:8000",
			want:     "http://[::1]:8000",
		},
		{
			name:     "schemeless_ipv6_loopback_bare",
			upstream: "::1",
			want:     "http://[::1]",
		},
		{
			name:     "schemeless_remote_host_defaults_https",
			upstream: "tavern.example.com:8443",
			want:     "https://tavern.example.com:8443",
		},
		{
			name:     "schemeless_remote_host_no_port_defaults_https",
			upstream: "tavern.example.com",
			want:     "https://tavern.example.com",
		},
		{
			name:     "schemeless_remote_ipv4_defaults_https",
			upstream: "10.0.0.5:8000",
			want:     "https://10.0.0.5:8000",
		},
		{
			name:     "schemeless_remote_ipv6_defaults_https",
			upstream: "[2001:db8::1]:8443",
			want:     "https://[2001:db8::1]:8443",
		},
		{
			name:     "schemeless_localhost_prefix_domain_defaults_https",
			upstream: "localhost.example.com:8000",
			want:     "https://localhost.example.com:8000",
		},
		{
			name:     "explicit_http_remote_preserved",
			upstream: "http://tavern.example.com:8000",
			want:     "http://tavern.example.com:8000",
		},
		{
			name:     "explicit_https_loopback_preserved",
			upstream: "https://127.0.0.1:8443",
			want:     "https://127.0.0.1:8443",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			assert.Equal(t, tc.want, normalizeUpstreamURL(tc.upstream))
		})
	}
}

func TestParseUpstream_TransportSecurity(t *testing.T) {
	tests := []struct {
		name         string
		upstream     string
		wantHost     string
		wantProtocol string
	}{
		{
			name:         "loopback_defaults_insecure",
			upstream:     "127.0.0.1:8000",
			wantHost:     "127.0.0.1:8000",
			wantProtocol: "insecure",
		},
		{
			name:         "localhost_defaults_insecure",
			upstream:     "localhost:8000",
			wantHost:     "localhost:8000",
			wantProtocol: "insecure",
		},
		{
			name:         "ipv6_loopback_defaults_insecure",
			upstream:     "[::1]:8000",
			wantHost:     "[::1]:8000",
			wantProtocol: "insecure",
		},
		{
			name:         "remote_host_defaults_tls",
			upstream:     "tavern.example.com:8443",
			wantHost:     "tavern.example.com:8443",
			wantProtocol: "tls",
		},
		{
			name:         "explicit_http_remote_uses_insecure",
			upstream:     "http://tavern.example.com:8000",
			wantHost:     "tavern.example.com:8000",
			wantProtocol: "insecure",
		},
		{
			name:         "explicit_https_loopback_uses_tls",
			upstream:     "https://127.0.0.1:8443",
			wantHost:     "127.0.0.1:8443",
			wantProtocol: "tls",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			u, tcCreds, err := parseUpstream(tc.upstream)
			require.NoError(t, err)
			assert.Equal(t, tc.wantHost, u.Host)
			assert.Equal(t, tc.wantProtocol, tcCreds.Info().SecurityProtocol)
		})
	}
}

func TestGetAuthToken_PrioritizesEnvAPITokenAndTrims(t *testing.T) {
	cachePath := filepath.Join(t.TempDir(), ".tavern-auth")
	require.NoError(t, os.WriteFile(cachePath, []byte("cached-token"), 0600))

	t.Setenv(auth.EnvAPIToken, "  env-token-value\n")

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	token, err := getAuthToken(ctx, "http://127.0.0.1:8000", cachePath)
	require.NoError(t, err)
	assert.Equal(t, "env-token-value", string(token))
}

func TestGetAuthToken_IgnoresLegacyAPIKeyVar(t *testing.T) {
	const cachedToken = "valid-cached-token"

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Header.Get(auth.HeaderAPIAccessToken) == cachedToken {
			w.WriteHeader(http.StatusOK)
			_, _ = w.Write([]byte(`{"data":{"me":{"id":"1"}}}`))
			return
		}
		http.Error(w, "unauthorized", http.StatusUnauthorized)
	}))
	defer srv.Close()

	cachePath := filepath.Join(t.TempDir(), ".tavern-auth")
	require.NoError(t, os.WriteFile(cachePath, []byte(cachedToken+"\n"), 0600))

	t.Setenv(auth.EnvAPIToken, "")
	t.Setenv("TAVERN_API_KEY", "legacy-should-be-ignored")

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	token, err := getAuthToken(ctx, srv.URL, cachePath)
	require.NoError(t, err)
	assert.Equal(t, cachedToken, string(token))
}

func TestAuthGRPCContext_AttachesTrimmedTokenMetadata(t *testing.T) {
	t.Setenv(auth.EnvAPIToken, "  grpc-token-123\r\n")

	ctx := authGRPCContext(context.Background(), "tavern.example.com:8443", filepath.Join(t.TempDir(), ".tavern-auth"))

	md, ok := metadata.FromOutgoingContext(ctx)
	require.True(t, ok)
	vals := md.Get(strings.ToLower(auth.HeaderAPIAccessToken))
	assert.Equal(t, []string{"grpc-token-123"}, vals)
}
