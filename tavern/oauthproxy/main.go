// Command oauthproxy is the single, centrally-registered OAuth relay that fronts Google OAuth
// for every Tavern tenant deployment. It owns one shared Google OAuth client with exactly one
// Authorized redirect URI (this service's own /oauth/callback), so adding a new tenant never
// requires a manual change in the Google Cloud Console: see docs/_docs/admin-guide/tavern.md for
// the full setup flow and tavern/internal/auth/oauth.go for how a tenant Tavern consumes it.
package main

import (
	"crypto/ed25519"
	"log"
	"net/http"

	"realm.pub/tavern/internal/crypto"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/internal/logging"
)

var startupLog = logging.For("oauthproxy.startup")

func main() {
	cfg, err := loadConfig()
	if err != nil {
		log.Fatalf("[FATAL] %v", err)
	}

	secretsManager, err := newSecretsManager()
	if err != nil {
		log.Fatalf("[FATAL] failed to configure secrets manager: %v", err)
	}

	privKey, err := crypto.GetPrivKeyED25519(secretsManager)
	if err != nil {
		log.Fatalf("[FATAL] failed to load signing key: %v", err)
	}
	pubKey, ok := privKey.Public().(ed25519.PublicKey)
	if !ok {
		log.Fatalf("[FATAL] signing key is not an ed25519 public key")
	}

	routes := tavernhttp.RouteMap{
		"/status":                newStatusHandler(),
		"/oauth/start":           newStartHandler(cfg, privKey),
		"/oauth/callback":        newCallbackHandler(cfg, privKey),
		"/.well-known/jwks.json": newJWKSHandler(pubKey),
	}

	srv := tavernhttp.NewServer(routes)

	addr := EnvHTTPListenAddr.String()
	startupLog.Info("oauth proxy starting", "addr", addr, "proxy_domain", cfg.ProxyDomain, "allowed_suffixes", cfg.AllowedSuffixes)
	if err := http.ListenAndServe(addr, srv); err != nil {
		log.Fatalf("[FATAL] oauth proxy server failed: %v", err)
	}
}
