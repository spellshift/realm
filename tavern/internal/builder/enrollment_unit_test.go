package builder_test

import (
	"crypto/ed25519"
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/pem"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v3"
	"realm.pub/tavern/internal/builder"
)

func TestGenerateCSR(t *testing.T) {
	dir := t.TempDir()
	require.NoError(t, builder.GenerateCSR(dir))
	keyPath := filepath.Join(dir, "builder.key")
	key, err := os.ReadFile(keyPath)
	require.NoError(t, err)
	info, err := os.Stat(keyPath)
	require.NoError(t, err)
	require.Equal(t, os.FileMode(0600), info.Mode().Perm())
	csr, err := os.ReadFile(filepath.Join(dir, "builder.csr"))
	require.NoError(t, err)
	block, _ := pem.Decode(csr)
	require.NotNil(t, block)
	request, err := x509.ParseCertificateRequest(block.Bytes)
	require.NoError(t, err)
	require.NoError(t, request.CheckSignature())
	_, ok := request.PublicKey.(ed25519.PublicKey)
	require.True(t, ok)
	require.ErrorContains(t, builder.GenerateCSR(dir), "refusing to overwrite")
	after, err := os.ReadFile(keyPath)
	require.NoError(t, err)
	require.Equal(t, key, after)
	require.Error(t, builder.GenerateCSR(filepath.Join(dir, "missing")))

	other := t.TempDir()
	require.NoError(t, os.WriteFile(filepath.Join(other, "builder.csr"), []byte("existing"), 0600))
	require.ErrorContains(t, builder.GenerateCSR(other), "refusing to overwrite")
	_, err = os.Stat(filepath.Join(other, "builder.key"))
	require.True(t, os.IsNotExist(err))
}

func TestSignBuilderCertificate(t *testing.T) {
	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	ca, err := builder.CreateCA(caKey)
	require.NoError(t, err)
	csr, keyPath := testEnrollment(t)
	_, err = builder.SignBuilderCertificate(nil, nil, "example", csr)
	require.ErrorContains(t, err, "builder CA is not configured")
	certPEM, err := builder.SignBuilderCertificate(ca, caKey, "example", csr)
	require.NoError(t, err)
	require.NotContains(t, string(certPEM), "PRIVATE KEY")
	block, _ := pem.Decode(certPEM)
	cert, err := x509.ParseCertificate(block.Bytes)
	require.NoError(t, err)
	require.Equal(t, "builder-example", cert.Subject.CommonName)
	require.NoError(t, cert.CheckSignatureFrom(ca))
	_, subjectKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	subjectCSR, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{Subject: pkix.Name{CommonName: "attacker"}}, subjectKey)
	require.NoError(t, err)
	subjectCertPEM, err := builder.SignBuilderCertificate(ca, caKey, "generated", string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: subjectCSR})))
	require.NoError(t, err)
	subjectBlock, _ := pem.Decode(subjectCertPEM)
	subjectCert, err := x509.ParseCertificate(subjectBlock.Bytes)
	require.NoError(t, err)
	require.Equal(t, "builder-generated", subjectCert.Subject.CommonName)
	config := &builder.Config{ID: "example", PollInterval: 5, SupportedTargets: []string{"linux"}, Upstream: "http://localhost", MTLSCert: string(certPEM), MTLSKeyFile: keyPath}
	_, err = builder.NewCredentialsFromConfig(config)
	require.NoError(t, err)
	config.MTLSKeyFile = filepath.Join(t.TempDir(), "missing.key")
	_, err = builder.NewCredentialsFromConfig(config)
	require.Error(t, err)
	_, wrongKeyPath := testEnrollment(t)
	config.MTLSKeyFile = wrongKeyPath
	_, err = builder.NewCredentialsFromConfig(config)
	require.ErrorContains(t, err, "does not match")
	dir := filepath.Dir(keyPath)
	configFile := filepath.Join(dir, "builder.yaml")
	data, err := yaml.Marshal(config)
	require.NoError(t, err)
	require.NoError(t, os.WriteFile(configFile, data, 0600))
	_, err = builder.ParseConfig(configFile)
	require.ErrorContains(t, err, "does not match")
	config.MTLSKeyFile = "builder.key"
	data, err = yaml.Marshal(config)
	require.NoError(t, err)
	require.NoError(t, os.WriteFile(configFile, data, 0600))
	loaded, err := builder.ParseConfig(configFile)
	require.NoError(t, err)
	require.Equal(t, keyPath, loaded.MTLSKeyFile)

	_, err = builder.SignBuilderCertificate(ca, caKey, "example", "bad")
	require.ErrorContains(t, err, "invalid builder CSR")
	rsaKey, err := rsa.GenerateKey(rand.Reader, 2048)
	require.NoError(t, err)
	rsaCSR, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{Subject: pkix.Name{CommonName: "attacker"}}, rsaKey)
	require.NoError(t, err)
	_, err = builder.SignBuilderCertificate(ca, caKey, "example", string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: rsaCSR})))
	require.ErrorContains(t, err, "Ed25519")

	// A valid request with a damaged signature must also be refused.
	csrBlock, _ := pem.Decode([]byte(csr))
	csrBlock.Bytes[len(csrBlock.Bytes)-1] ^= 1
	_, err = builder.SignBuilderCertificate(ca, caKey, "example", string(pem.EncodeToMemory(csrBlock)))
	require.Error(t, err)
	require.True(t, strings.Contains(err.Error(), "CSR"))
}
