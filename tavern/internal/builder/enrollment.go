package builder

import (
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"encoding/pem"
	"fmt"
	"os"
	"path/filepath"
)

// GenerateCSR creates the builder's private key and certificate request locally.
func GenerateCSR(dir string) error {
	info, err := os.Stat(dir)
	if err != nil {
		return fmt.Errorf("output directory: %w", err)
	}
	if !info.IsDir() {
		return fmt.Errorf("output path is not a directory")
	}
	keyPath := filepath.Join(dir, "builder.key")
	csrPath := filepath.Join(dir, "builder.csr")
	for _, path := range []string{keyPath, csrPath} {
		if _, err := os.Lstat(path); err == nil {
			return fmt.Errorf("refusing to overwrite %s", path)
		} else if !os.IsNotExist(err) {
			return err
		}
	}
	_, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		return err
	}
	keyDER, err := x509.MarshalPKCS8PrivateKey(priv)
	if err != nil {
		return err
	}
	csrDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, priv)
	if err != nil {
		return err
	}
	keyFile, err := os.OpenFile(keyPath, os.O_WRONLY|os.O_CREATE|os.O_EXCL, 0600)
	if err != nil {
		return fmt.Errorf("create builder key: %w", err)
	}
	complete := false
	defer func() {
		if !complete {
			os.Remove(keyPath)
		}
	}()
	if _, err := keyFile.Write(pem.EncodeToMemory(&pem.Block{Type: "PRIVATE KEY", Bytes: keyDER})); err != nil {
		if closeErr := keyFile.Close(); closeErr != nil {
			return fmt.Errorf("write builder key: %v; close builder key: %w", err, closeErr)
		}
		return err
	}
	if err := keyFile.Close(); err != nil {
		return err
	}
	csrFile, err := os.OpenFile(csrPath, os.O_WRONLY|os.O_CREATE|os.O_EXCL, 0644)
	if err != nil {
		return fmt.Errorf("create builder CSR: %w", err)
	}
	if _, err := csrFile.Write(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrDER})); err != nil {
		if closeErr := csrFile.Close(); closeErr != nil {
			os.Remove(csrPath)
			return fmt.Errorf("write builder CSR: %v; close builder CSR: %w", err, closeErr)
		}
		os.Remove(csrPath)
		return err
	}
	if err := csrFile.Close(); err != nil {
		os.Remove(csrPath)
		return err
	}
	complete = true
	return nil
}
