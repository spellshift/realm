package builder

import (
	"bytes"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/pem"
	"errors"
	"fmt"
	"math/big"
	"time"
)

// ErrInvalidBuilderCSR identifies client-supplied certificate requests that cannot be signed.
var ErrInvalidBuilderCSR = errors.New("invalid builder CSR")

// ErrInvalidRedirectorCSR identifies invalid redirector certificate requests.
var ErrInvalidRedirectorCSR = errors.New("invalid redirector CSR")

// CreateCA creates a self-signed CA certificate from the provided ED25519 private key.
// The certificate is generated in-memory each time — only the private key needs to be persisted.
func CreateCA(privKey ed25519.PrivateKey) (*x509.Certificate, error) {
	pubKey := privKey.Public().(ed25519.PublicKey)

	serialNumber, err := rand.Int(rand.Reader, new(big.Int).Lsh(big.NewInt(1), 128))
	if err != nil {
		return nil, fmt.Errorf("failed to generate serial number: %w", err)
	}

	template := x509.Certificate{
		SerialNumber: serialNumber,
		Subject: pkix.Name{
			CommonName:   "Realm Builder CA",
			Organization: []string{"Realm"},
		},
		NotBefore:             time.Now(),
		NotAfter:              time.Now().Add(10 * 365 * 24 * time.Hour),
		KeyUsage:              x509.KeyUsageCertSign | x509.KeyUsageCRLSign,
		BasicConstraintsValid: true,
		IsCA:                  true,
		MaxPathLen:            0,
	}

	caDER, err := x509.CreateCertificate(rand.Reader, &template, &template, pubKey, privKey)
	if err != nil {
		return nil, fmt.Errorf("failed to create CA certificate: %w", err)
	}

	cert, err := x509.ParseCertificate(caDER)
	if err != nil {
		return nil, fmt.Errorf("failed to parse generated CA certificate: %w", err)
	}

	return cert, nil
}

// SignBuilderCertificate generates a client certificate for a builder, signed by the CA.
// The certificate CN is set to "builder-{identifier}" for identity attribution.
func SignBuilderCertificate(ca *x509.Certificate, caKey ed25519.PrivateKey, builderIdentifier string, csrPEM string) ([]byte, error) {
	return signIdentityCertificate(ca, caKey, "builder-"+builderIdentifier, csrPEM, "builder", ErrInvalidBuilderCSR)
}

// SignRedirectorCertificate signs a redirector CSR under the shared enrollment CA.
// Its CN prefix keeps redirector credentials distinct from builder credentials.
func SignRedirectorCertificate(ca *x509.Certificate, caKey ed25519.PrivateKey, identifier, csrPEM string) ([]byte, error) {
	return signIdentityCertificate(ca, caKey, "redirector-"+identifier, csrPEM, "redirector", ErrInvalidRedirectorCSR)
}

func signIdentityCertificate(ca *x509.Certificate, caKey ed25519.PrivateKey, commonName, csrPEM, kind string, invalidCSR error) ([]byte, error) {
	block, rest := pem.Decode([]byte(csrPEM))
	if block == nil || block.Type != "CERTIFICATE REQUEST" || len(bytes.TrimSpace(rest)) != 0 {
		return nil, fmt.Errorf("%w PEM", invalidCSR)
	}
	csr, err := x509.ParseCertificateRequest(block.Bytes)
	if err != nil {
		return nil, fmt.Errorf("%w: %w", invalidCSR, err)
	}
	if err := csr.CheckSignature(); err != nil {
		return nil, fmt.Errorf("%w signature: %w", invalidCSR, err)
	}
	publicKey, ok := csr.PublicKey.(ed25519.PublicKey)
	if !ok {
		return nil, fmt.Errorf("%w: must use an Ed25519 public key", invalidCSR)
	}
	if ca == nil || len(caKey) != ed25519.PrivateKeySize {
		return nil, fmt.Errorf("%s CA is not configured", kind)
	}

	serialNumber, err := rand.Int(rand.Reader, new(big.Int).Lsh(big.NewInt(1), 128))
	if err != nil {
		return nil, fmt.Errorf("failed to generate serial number: %w", err)
	}

	now := time.Now()
	template := x509.Certificate{
		SerialNumber: serialNumber,
		Subject: pkix.Name{
			CommonName:   commonName,
			Organization: []string{"Realm"},
		},
		NotBefore:             now.Add(-2 * time.Minute),
		NotAfter:              now.Add(365 * 24 * time.Hour),
		KeyUsage:              x509.KeyUsageDigitalSignature,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageClientAuth},
		BasicConstraintsValid: true,
	}

	certDER, err := x509.CreateCertificate(rand.Reader, &template, ca, publicKey, caKey)
	if err != nil {
		return nil, fmt.Errorf("failed to create %s certificate: %w", kind, err)
	}
	return pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: certDER}), nil
}
