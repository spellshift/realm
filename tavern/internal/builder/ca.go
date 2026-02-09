package builder

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/pem"
	"fmt"
	"math/big"
	"time"

	"realm.pub/tavern/internal/secrets"
)

// GetOrCreateCA retrieves or generates the Builder CA certificate and private key.
// The CA is persisted via the secrets manager so it survives restarts.
// Data is stored as PEM (text-safe) to avoid binary corruption in YAML-based secrets managers.
func GetOrCreateCA(secretsManager secrets.SecretsManager) (*x509.Certificate, *ecdsa.PrivateKey, error) {
	if secretsManager == nil {
		return nil, nil, fmt.Errorf("secrets manager is nil")
	}

	// Try to load existing CA (stored as PEM for text-safety)
	certPEMBytes, certErr := secretsManager.GetValue("builder_ca_certificate")
	keyPEMBytes, keyErr := secretsManager.GetValue("builder_ca_private_key")

	if certErr == nil && keyErr == nil && len(certPEMBytes) > 0 && len(keyPEMBytes) > 0 {
		certBlock, _ := pem.Decode(certPEMBytes)
		if certBlock == nil {
			return nil, nil, fmt.Errorf("failed to decode stored CA certificate PEM")
		}
		cert, err := x509.ParseCertificate(certBlock.Bytes)
		if err != nil {
			return nil, nil, fmt.Errorf("failed to parse stored CA certificate: %w", err)
		}

		keyBlock, _ := pem.Decode(keyPEMBytes)
		if keyBlock == nil {
			return nil, nil, fmt.Errorf("failed to decode stored CA private key PEM")
		}
		key, err := x509.ParseECPrivateKey(keyBlock.Bytes)
		if err != nil {
			return nil, nil, fmt.Errorf("failed to parse stored CA private key: %w", err)
		}
		return cert, key, nil
	}

	// Generate new CA
	privKey, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to generate CA private key: %w", err)
	}

	serialNumber, err := rand.Int(rand.Reader, new(big.Int).Lsh(big.NewInt(1), 128))
	if err != nil {
		return nil, nil, fmt.Errorf("failed to generate serial number: %w", err)
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

	caDER, err := x509.CreateCertificate(rand.Reader, &template, &template, &privKey.PublicKey, privKey)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to create CA certificate: %w", err)
	}

	keyDER, err := x509.MarshalECPrivateKey(privKey)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to marshal CA private key: %w", err)
	}

	// Store as PEM to avoid binary corruption in text-based secrets managers
	certPEM := pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: caDER})
	keyPEM := pem.EncodeToMemory(&pem.Block{Type: "EC PRIVATE KEY", Bytes: keyDER})

	if _, err := secretsManager.SetValue("builder_ca_certificate", certPEM); err != nil {
		return nil, nil, fmt.Errorf("failed to store CA certificate: %w", err)
	}
	if _, err := secretsManager.SetValue("builder_ca_private_key", keyPEM); err != nil {
		return nil, nil, fmt.Errorf("failed to store CA private key: %w", err)
	}

	cert, err := x509.ParseCertificate(caDER)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to parse generated CA certificate: %w", err)
	}

	return cert, privKey, nil
}

// SignBuilderCertificate generates a client certificate for a builder, signed by the CA.
// The certificate CN is set to "builder-{identifier}" for identity attribution.
func SignBuilderCertificate(ca *x509.Certificate, caKey *ecdsa.PrivateKey, builderIdentifier string) (certPEM []byte, keyPEM []byte, err error) {
	privKey, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to generate builder private key: %w", err)
	}

	serialNumber, err := rand.Int(rand.Reader, new(big.Int).Lsh(big.NewInt(1), 128))
	if err != nil {
		return nil, nil, fmt.Errorf("failed to generate serial number: %w", err)
	}

	template := x509.Certificate{
		SerialNumber: serialNumber,
		Subject: pkix.Name{
			CommonName:   fmt.Sprintf("builder-%s", builderIdentifier),
			Organization: []string{"Realm"},
		},
		NotBefore:             time.Now(),
		NotAfter:              time.Now().Add(365 * 24 * time.Hour),
		KeyUsage:              x509.KeyUsageDigitalSignature | x509.KeyUsageKeyEncipherment,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageClientAuth},
		BasicConstraintsValid: true,
	}

	certDER, err := x509.CreateCertificate(rand.Reader, &template, ca, &privKey.PublicKey, caKey)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to create builder certificate: %w", err)
	}

	certPEM = pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: certDER})

	keyDER, err := x509.MarshalECPrivateKey(privKey)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to marshal builder private key: %w", err)
	}
	keyPEM = pem.EncodeToMemory(&pem.Block{Type: "EC PRIVATE KEY", Bytes: keyDER})

	return certPEM, keyPEM, nil
}
