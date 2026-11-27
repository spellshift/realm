package main

import (
	"fmt"
	"strings"
	"time"

	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"
	"realm.pub/tavern/internal/oauthproxy"
	"realm.pub/tavern/internal/secrets"
)

var (
	// EnvHTTPListenAddr sets the address (ip:port) for the proxy's HTTP server to bind to.
	EnvHTTPListenAddr = envString{"HTTP_LISTEN_ADDR", "0.0.0.0:8080"}

	// EnvOAuthClientID and EnvOAuthClientSecret configure the single, shared Google OAuth client
	// the proxy uses on behalf of every tenant. This client has exactly one Authorized redirect
	// URI registered with Google: "<OAUTH_PROXY_DOMAIN>/oauth/callback".
	EnvOAuthClientID     = envString{"OAUTH_CLIENT_ID", ""}
	EnvOAuthClientSecret = envString{"OAUTH_CLIENT_SECRET", ""}

	// EnvProxyDomain is the proxy's own public domain (e.g. "oauth-proxy-123.us-central1.run.app"
	// or a custom domain), used to build its one fixed redirect_uri and as the JWKS issuer.
	EnvProxyDomain = envString{"OAUTH_PROXY_DOMAIN", ""}

	// EnvAllowedSuffixes is a comma-separated list of host patterns the proxy will redirect back
	// to after a successful login. Each entry may be an exact host (e.g. "tavern.example.com"),
	// a dot-prefixed domain suffix (e.g. ".example.com", matching any subdomain and the bare
	// domain itself), or a leading-wildcard single-label pattern (e.g.
	// "*-tavern-749076704275.us-east4.run.app"). Defaults to
	// "*-tavern-749076704275.us-east4.run.app" (hardcoded for the Spellshift dev GCP project
	// and us-east4 region; override this env var when deploying for other projects or regions).
	EnvAllowedSuffixes = envString{"OAUTH_PROXY_ALLOWED_SUFFIXES", "*-tavern-749076704275.us-east4.run.app"}

	// EnvGCPProjectID and EnvSecretsManagerPath select where the proxy's ed25519 signing key is
	// persisted, mirroring Tavern's own secrets manager selection (tavern/app.go,
	// newSecretsManager). The key must persist across Cloud Run revisions/restarts so the
	// proxy's published JWKS stays stable for tenants that cached it.
	EnvGCPProjectID       = envString{"GCP_PROJECT_ID", ""}
	EnvSecretsManagerPath = envString{"SECRETS_FILE_PATH", ""}

	// EnvResourcePrefix namespaces the proxy's secrets, in case the proxy's Secret Manager is
	// shared with other resources in the same GCP project.
	EnvResourcePrefix = envString{"OAUTH_PROXY_RESOURCE_PREFIX", "oauthproxy"}
)

// stateTTL bounds how long a tenant has to complete the Google consent screen.
// assertionTTL bounds how long a proxy-issued identity assertion remains valid; it only needs to
// survive a single browser redirect back to the tenant, so it is kept very short.
const (
	stateTTL     = 10 * time.Minute
	assertionTTL = 60 * time.Second
)

// Config holds the proxy's runtime configuration.
type Config struct {
	OAuth           oauth2.Config
	ProfileURL      string
	AllowedSuffixes []string
	ProxyDomain     string
}

func (c *Config) secureCookies() bool {
	return strings.HasPrefix(c.ProxyDomain, "https://")
}

// loadConfig reads and validates the proxy's configuration from the environment.
func loadConfig() (*Config, error) {
	clientID := EnvOAuthClientID.String()
	clientSecret := EnvOAuthClientSecret.String()
	domain := EnvProxyDomain.String()

	if clientID == "" {
		return nil, fmt.Errorf("must provide value for environment var 'OAUTH_CLIENT_ID'")
	}
	if clientSecret == "" {
		return nil, fmt.Errorf("must provide value for environment var 'OAUTH_CLIENT_SECRET'")
	}
	if domain == "" {
		return nil, fmt.Errorf("must provide value for environment var 'OAUTH_PROXY_DOMAIN'")
	}
	domain, err := oauthproxy.NormalizeBaseURL(domain)
	if err != nil {
		return nil, fmt.Errorf("invalid 'OAUTH_PROXY_DOMAIN': %w", err)
	}
	if err := oauthproxy.RequireSecureURL(domain); err != nil {
		return nil, fmt.Errorf("invalid 'OAUTH_PROXY_DOMAIN': %w", err)
	}

	var suffixes []string
	for _, s := range strings.Split(EnvAllowedSuffixes.String(), ",") {
		s = strings.TrimSpace(s)
		if s == "" {
			continue
		}
		if err := oauthproxy.ValidateAllowedPattern(s); err != nil {
			return nil, fmt.Errorf("invalid entry %q in 'OAUTH_PROXY_ALLOWED_SUFFIXES': %w", s, err)
		}
		suffixes = append(suffixes, s)
	}
	if len(suffixes) == 0 {
		return nil, fmt.Errorf("must provide at least one suffix in 'OAUTH_PROXY_ALLOWED_SUFFIXES'")
	}

	return &Config{
		OAuth: oauth2.Config{
			ClientID:     clientID,
			ClientSecret: clientSecret,
			RedirectURL:  domain + "/oauth/callback",
			Scopes: []string{
				"https://www.googleapis.com/auth/userinfo.profile",
				"https://www.googleapis.com/auth/userinfo.email",
			},
			Endpoint: google.Endpoint,
		},
		ProfileURL:      "https://www.googleapis.com/oauth2/v3/userinfo",
		AllowedSuffixes: suffixes,
		ProxyDomain:     domain,
	}, nil
}

// newSecretsManager mirrors tavern/app.go's newSecretsManager: GCP Secret Manager when
// GCP_PROJECT_ID is set, an explicit debug file when SECRETS_FILE_PATH is set, otherwise an
// insecure local file default (fine for local development, never for production).
func newSecretsManager() (secrets.SecretsManager, error) {
	if EnvGCPProjectID.String() == "" && EnvSecretsManagerPath.String() == "" {
		defaultPath := fmt.Sprintf("/tmp/%s-secrets", EnvResourcePrefix.String())
		return secrets.NewDebugFileSecrets(defaultPath)
	}
	if EnvSecretsManagerPath.String() == "" {
		return secrets.NewGcp(EnvGCPProjectID.String(), EnvResourcePrefix.String())
	}
	return secrets.NewDebugFileSecrets(EnvSecretsManagerPath.String())
}
