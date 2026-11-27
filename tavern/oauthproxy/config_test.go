package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func setValidProxyEnv(t *testing.T) {
	t.Helper()
	t.Setenv(EnvOAuthClientID.Key, "client-id")
	t.Setenv(EnvOAuthClientSecret.Key, "client-secret")
	t.Setenv(EnvProxyDomain.Key, "https://proxy.example.com")
	t.Setenv(EnvAllowedSuffixes.Key, "")
}

func TestLoadConfigValid(t *testing.T) {
	setValidProxyEnv(t)

	cfg, err := loadConfig()
	require.NoError(t, err)
	assert.Equal(t, "client-id", cfg.OAuth.ClientID)
	assert.Equal(t, "client-secret", cfg.OAuth.ClientSecret)
	assert.Equal(t, "https://proxy.example.com", cfg.ProxyDomain)
	assert.Equal(t, "https://proxy.example.com/oauth/callback", cfg.OAuth.RedirectURL)
	assert.Equal(t, []string{"*-tavern-749076704275.us-east4.run.app"}, cfg.AllowedSuffixes)
	assert.True(t, cfg.secureCookies())
}

func TestLoadConfigLoopbackHTTP(t *testing.T) {
	setValidProxyEnv(t)
	t.Setenv(EnvProxyDomain.Key, "http://localhost:8080/")
	t.Setenv(EnvAllowedSuffixes.Key, " localhost , *-tavern-749076704275.us-east4.run.app ")

	cfg, err := loadConfig()
	require.NoError(t, err)
	assert.Equal(t, "http://localhost:8080", cfg.ProxyDomain)
	assert.Equal(t, "http://localhost:8080/oauth/callback", cfg.OAuth.RedirectURL)
	assert.Equal(t, []string{"localhost", "*-tavern-749076704275.us-east4.run.app"}, cfg.AllowedSuffixes)
	assert.False(t, cfg.secureCookies())
}

func TestLoadConfigErrors(t *testing.T) {
	cases := []struct {
		name string
		env  map[string]string
	}{
		{
			name: "missing client id",
			env:  map[string]string{EnvOAuthClientID.Key: ""},
		},
		{
			name: "missing client secret",
			env:  map[string]string{EnvOAuthClientSecret.Key: ""},
		},
		{
			name: "missing proxy domain",
			env:  map[string]string{EnvProxyDomain.Key: ""},
		},
		{
			name: "insecure non-loopback domain",
			env:  map[string]string{EnvProxyDomain.Key: "http://proxy.example.com"},
		},
		{
			name: "domain with path",
			env:  map[string]string{EnvProxyDomain.Key: "https://proxy.example.com/prefix"},
		},
		{
			name: "domain with userinfo",
			env:  map[string]string{EnvProxyDomain.Key: "https://user:pass@proxy.example.com"},
		},
		{
			name: "whitespace only allowed suffixes",
			env:  map[string]string{EnvAllowedSuffixes.Key: " , , "},
		},
		{
			name: "bare wildcard allowed suffix",
			env:  map[string]string{EnvAllowedSuffixes.Key: "*"},
		},
		{
			name: "double wildcard allowed suffix",
			env:  map[string]string{EnvAllowedSuffixes.Key: "**-tavern-749076704275.us-east4.run.app"},
		},
		{
			name: "mid-host wildcard allowed suffix",
			env:  map[string]string{EnvAllowedSuffixes.Key: "foo.*.example.com"},
		},
		{
			name: "url instead of host in allowed suffixes",
			env:  map[string]string{EnvAllowedSuffixes.Key: "https://tenant.example.com"},
		},
		{
			name: "bare dot allowed suffix",
			env:  map[string]string{EnvAllowedSuffixes.Key: "."},
		},
		{
			name: "wildcard label suffix ending with hyphen",
			env:  map[string]string{EnvAllowedSuffixes.Key: "*-tavern-.us-east4.run.app"},
		},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			setValidProxyEnv(t)
			for k, v := range tc.env {
				t.Setenv(k, v)
			}
			_, err := loadConfig()
			assert.Error(t, err)
		})
	}
}
