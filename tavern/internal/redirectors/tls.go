package redirectors

import (
	"context"
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/tls"
	"crypto/x509"
	"crypto/x509/pkix"
	"fmt"
	"log/slog"
	"math/big"
	"net"
	"os"
	"time"

	"github.com/caddyserver/certmagic"
)

// NewTLSConfig attempts to provision a TLS certificate using certmagic (ACME).
// If all ACME challenges fail, it falls back to a self-signed certificate.
// The hostname is used for certificate provisioning (e.g. "redirector.example.com").
func NewTLSConfig(ctx context.Context, hostname string) (*tls.Config, error) {
	slog.DebugContext(ctx, "redirectors: configuring TLS", "hostname", hostname)

	// Try certmagic if we have a real hostname (not empty, not an IP)
	if hostname != "" && net.ParseIP(hostname) == nil {
		slog.DebugContext(ctx, "redirectors: attempting ACME certificate provisioning", "hostname", hostname)
		tlsCfg, err := tryACME(ctx, hostname)
		if err != nil {
			slog.WarnContext(ctx, "ACME certificate provisioning failed, falling back to self-signed", "hostname", hostname, "error", err)
		} else {
			slog.InfoContext(ctx, "ACME certificate provisioned successfully", "hostname", hostname)
			slog.DebugContext(ctx, "redirectors: TLS config ready (ACME)", "hostname", hostname, "min_version", tlsCfg.MinVersion, "num_certificates", len(tlsCfg.Certificates))
			return tlsCfg, nil
		}
	} else {
		slog.DebugContext(ctx, "redirectors: no hostname for ACME, will use self-signed", "hostname", hostname)
		slog.InfoContext(ctx, "no hostname provided for ACME, using self-signed certificate", "hostname", hostname)
	}

	// Fallback to self-signed
	slog.DebugContext(ctx, "redirectors: generating self-signed certificate", "hostname", hostname)
	tlsCfg, err := selfSignedTLSConfig(hostname)
	if err != nil {
		return nil, fmt.Errorf("failed to generate self-signed certificate: %w", err)
	}
	slog.WarnContext(ctx, "using self-signed TLS certificate")
	slog.DebugContext(ctx, "redirectors: TLS config ready (self-signed)", "hostname", hostname, "num_certificates", len(tlsCfg.Certificates))
	return tlsCfg, nil
}

// tryACME attempts to obtain a TLS certificate via ACME using certmagic.
func tryACME(ctx context.Context, host string) (tlsCfg *tls.Config, err error) {
	// Recover from any panics inside certmagic (e.g. nil pointer dereferences
	// when ACME challenge solvers are not fully initialized).
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("certmagic panicked: %v", r)
		}
	}()

	acme := certmagic.ACMEIssuer{
		Agreed: true,
		Email:  os.Getenv("REDIRECTOR_ACME_EMAIL"),
		CA:     certmagic.LetsEncryptProductionCA,
	}

	cfg := certmagic.NewDefault()
	acme.Logger = cfg.Logger
	cfg.Issuers = []certmagic.Issuer{certmagic.NewACMEIssuer(cfg, acme)}

	if err := cfg.ManageSync(ctx, []string{host}); err != nil {
		return nil, fmt.Errorf("certmagic failed to manage certificate for %q: %w", host, err)
	}

	tlsCfg = cfg.TLSConfig()
	if tlsCfg == nil {
		return nil, fmt.Errorf("certmagic returned nil TLS config for %q", host)
	}
	return tlsCfg, nil
}

// selfSignedTLSConfig generates a self-signed TLS certificate and returns a tls.Config.
func selfSignedTLSConfig(host string) (*tls.Config, error) {
	key, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		return nil, fmt.Errorf("failed to generate private key: %w", err)
	}

	serialNumber, err := rand.Int(rand.Reader, new(big.Int).Lsh(big.NewInt(1), 128))
	if err != nil {
		return nil, fmt.Errorf("failed to generate serial number: %w", err)
	}

	template := x509.Certificate{
		SerialNumber: serialNumber,
		Subject: pkix.Name{
			Organization: []string{"Realm Redirector"},
		},
		NotBefore:             time.Now(),
		NotAfter:              time.Now().Add(365 * 24 * time.Hour),
		KeyUsage:              x509.KeyUsageDigitalSignature | x509.KeyUsageKeyEncipherment,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageServerAuth},
		BasicConstraintsValid: true,
	}

	if ip := net.ParseIP(host); ip != nil {
		template.IPAddresses = []net.IP{ip}
	} else if host != "" {
		template.DNSNames = []string{host}
	} else {
		template.IPAddresses = []net.IP{net.IPv4(127, 0, 0, 1)}
		template.DNSNames = []string{"localhost"}
	}

	certDER, err := x509.CreateCertificate(rand.Reader, &template, &template, &key.PublicKey, key)
	if err != nil {
		return nil, fmt.Errorf("failed to create certificate: %w", err)
	}

	tlsCert := tls.Certificate{
		Certificate: [][]byte{certDER},
		PrivateKey:  key,
	}

	return &tls.Config{
		Certificates: []tls.Certificate{tlsCert},
	}, nil
}
