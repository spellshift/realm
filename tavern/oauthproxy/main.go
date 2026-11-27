// Command oauthproxy is the single, centrally-registered OAuth relay that fronts Google OAuth
// for every Tavern tenant deployment. It owns one shared Google OAuth client with exactly one
// Authorized redirect URI (this service's own /oauth/callback), so adding a new tenant never
// requires a manual change in the Google Cloud Console: see docs/_docs/admin-guide/tavern.md for
// the full setup flow and tavern/internal/auth/oauth.go for how a tenant Tavern consumes it.
package main

import (
	"context"
	"crypto/ed25519"
	"errors"
	"log"
	"net/http"
	"os/signal"
	"syscall"
	"time"

	"realm.pub/tavern/internal/crypto"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/internal/logging"
)

var startupLog = logging.For("oauthproxy.startup")

const (
	serverReadHeaderTimeout = 10 * time.Second
	serverReadTimeout       = 30 * time.Second
	serverWriteTimeout      = 30 * time.Second
	serverIdleTimeout       = 120 * time.Second
	serverShutdownTimeout   = 10 * time.Second
)

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
	httpSrv := &http.Server{
		Addr:              addr,
		Handler:           srv,
		ReadHeaderTimeout: serverReadHeaderTimeout,
		ReadTimeout:       serverReadTimeout,
		WriteTimeout:      serverWriteTimeout,
		IdleTimeout:       serverIdleTimeout,
	}

	sigCtx, stop := signal.NotifyContext(context.Background(), syscall.SIGTERM, syscall.SIGINT)
	defer stop()

	errCh := make(chan error, 1)
	go func() {
		startupLog.Info("oauth proxy starting", "addr", addr, "proxy_domain", cfg.ProxyDomain, "allowed_suffixes", cfg.AllowedSuffixes)
		if err := httpSrv.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			errCh <- err
		}
		close(errCh)
	}()

	select {
	case err := <-errCh:
		if err != nil {
			log.Fatalf("[FATAL] oauth proxy server failed: %v", err)
		}
	case <-sigCtx.Done():
		startupLog.Info("oauth proxy shutting down")
		shutdownCtx, cancel := context.WithTimeout(context.Background(), serverShutdownTimeout)
		defer cancel()
		if err := httpSrv.Shutdown(shutdownCtx); err != nil {
			log.Fatalf("[FATAL] oauth proxy graceful shutdown failed: %v", err)
		}
		if err := <-errCh; err != nil {
			log.Fatalf("[FATAL] oauth proxy server failed: %v", err)
		}
	}
}
