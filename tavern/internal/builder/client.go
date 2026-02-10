package builder

import (
	"context"
	"crypto/ed25519"
	"crypto/x509"
	"encoding/base64"
	"encoding/pem"
	"fmt"
	"log/slog"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"

	"realm.pub/tavern/internal/builder/builderpb"
)

// builderCredentials implements grpc.PerRPCCredentials for mTLS authentication.
type builderCredentials struct {
	certDERBase64 string
	privKey       ed25519.PrivateKey
}

// GetRequestMetadata generates fresh authentication metadata for each RPC call.
// It signs the current timestamp with the builder's private key to prove possession.
func (c *builderCredentials) GetRequestMetadata(ctx context.Context, uri ...string) (map[string]string, error) {
	timestamp := time.Now().UTC().Format(time.RFC3339Nano)

	sig := ed25519.Sign(c.privKey, []byte(timestamp))

	return map[string]string{
		mdKeyBuilderCert:      c.certDERBase64,
		mdKeyBuilderSignature: base64.StdEncoding.EncodeToString(sig),
		mdKeyBuilderTimestamp: timestamp,
	}, nil
}

// RequireTransportSecurity returns false since Tavern uses h2c (HTTP/2 cleartext)
// with TLS terminated at the reverse proxy level.
func (c *builderCredentials) RequireTransportSecurity() bool {
	return false
}

// NewCredentialsFromConfig creates gRPC per-RPC credentials from a builder config.
func NewCredentialsFromConfig(cfg *Config) (credentials.PerRPCCredentials, error) {
	return parseMTLSCredentials(cfg.MTLS)
}

// parseMTLSCredentials loads the certificate and private key from the config's
// PEM bundle string.
func parseMTLSCredentials(mtlsPEM string) (*builderCredentials, error) {
	pemBundle := []byte(mtlsPEM)

	var certDER []byte
	var privKey ed25519.PrivateKey

	for {
		block, rest := pem.Decode(pemBundle)
		if block == nil {
			break
		}
		switch block.Type {
		case "CERTIFICATE":
			certDER = block.Bytes
		case "PRIVATE KEY":
			key, err := x509.ParsePKCS8PrivateKey(block.Bytes)
			if err != nil {
				return nil, fmt.Errorf("failed to parse private key: %w", err)
			}
			edKey, ok := key.(ed25519.PrivateKey)
			if !ok {
				return nil, fmt.Errorf("private key is not ED25519")
			}
			privKey = edKey
		}
		pemBundle = rest
	}

	if certDER == nil {
		return nil, fmt.Errorf("no certificate found in mTLS bundle")
	}
	if privKey == nil {
		return nil, fmt.Errorf("no private key found in mTLS bundle")
	}

	return &builderCredentials{
		certDERBase64: base64.StdEncoding.EncodeToString(certDER),
		privKey:       privKey,
	}, nil
}

// Run starts the builder process using the provided configuration.
// It connects to the configured upstream server with mTLS credentials and sends a ping request.
func Run(ctx context.Context, cfg *Config) error {
	slog.InfoContext(ctx, "builder started",
		"id", cfg.ID,
		"supported_targets", cfg.SupportedTargets,
		"upstream", cfg.Upstream,
	)

	creds, err := parseMTLSCredentials(cfg.MTLS)
	if err != nil {
		return fmt.Errorf("failed to parse mTLS credentials: %w", err)
	}

	conn, err := grpc.NewClient(cfg.Upstream,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithPerRPCCredentials(creds),
	)
	if err != nil {
		return fmt.Errorf("failed to connect to upstream %q: %w", cfg.Upstream, err)
	}
	defer conn.Close()

	client := builderpb.NewBuilderClient(conn)
	_, err = client.Ping(ctx, &builderpb.PingRequest{})
	if err != nil {
		return fmt.Errorf("failed to ping upstream: %w", err)
	}

	slog.InfoContext(ctx, "successfully pinged upstream", "upstream", cfg.Upstream)

	// Wait for context cancellation
	<-ctx.Done()
	return ctx.Err()
}
