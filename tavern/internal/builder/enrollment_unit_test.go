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
	"realm.pub/tavern/internal/builder"
)

// signedTestIdentity builds a self-consistent Identity (CSR signed by a
// freshly generated CA) for use in LoadIdentity/SaveIdentity/EnsureIdentity
// unit tests that don't need a running GraphQL server.
func signedTestIdentity(t *testing.T, id string) *builder.Identity {
	t.Helper()
	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	ca, err := builder.CreateCA(caKey)
	require.NoError(t, err)

	_, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	csrDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, privKey)
	require.NoError(t, err)
	csr := string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrDER}))

	certPEM, err := builder.SignBuilderCertificate(ca, caKey, id, csr)
	require.NoError(t, err)
	keyDER, err := x509.MarshalPKCS8PrivateKey(privKey)
	require.NoError(t, err)

	return &builder.Identity{
		ID:          id,
		Certificate: string(certPEM),
		PrivateKey:  string(pem.EncodeToMemory(&pem.Block{Type: "PRIVATE KEY", Bytes: keyDER})),
	}
}

func TestSignBuilderCertificate(t *testing.T) {
	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	ca, err := builder.CreateCA(caKey)
	require.NoError(t, err)

	_, subjectKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	csrDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{Subject: pkix.Name{CommonName: "attacker"}}, subjectKey)
	require.NoError(t, err)
	csr := string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrDER}))

	_, err = builder.SignBuilderCertificate(nil, nil, "example", csr)
	require.ErrorContains(t, err, "builder CA is not configured")

	certPEM, err := builder.SignBuilderCertificate(ca, caKey, "example", csr)
	require.NoError(t, err)
	require.NotContains(t, string(certPEM), "PRIVATE KEY")
	block, _ := pem.Decode(certPEM)
	cert, err := x509.ParseCertificate(block.Bytes)
	require.NoError(t, err)
	// The cert CN is derived from the identifier, independent of the CSR subject.
	require.Equal(t, "builder-example", cert.Subject.CommonName)
	require.NoError(t, cert.CheckSignatureFrom(ca))

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

func TestLoadSaveIdentity(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "builder.identity")
	identity := signedTestIdentity(t, "example")

	require.NoError(t, builder.SaveIdentity(path, identity))
	info, err := os.Stat(path)
	require.NoError(t, err)
	require.Equal(t, os.FileMode(0600), info.Mode().Perm())

	loaded, err := builder.LoadIdentity(path)
	require.NoError(t, err)
	require.Equal(t, identity.ID, loaded.ID)
	require.Equal(t, identity.Certificate, loaded.Certificate)

	// Refuses to overwrite an existing identity file.
	require.ErrorContains(t, builder.SaveIdentity(path, signedTestIdentity(t, "other")), "already exists")
	stillOriginal, err := builder.LoadIdentity(path)
	require.NoError(t, err)
	require.Equal(t, "example", stillOriginal.ID)

	// Missing file.
	_, err = builder.LoadIdentity(filepath.Join(dir, "missing"))
	require.True(t, os.IsNotExist(err))

	// Malformed YAML.
	malformedPath := filepath.Join(dir, "malformed.identity")
	require.NoError(t, os.WriteFile(malformedPath, []byte("not: [valid"), 0600))
	_, err = builder.LoadIdentity(malformedPath)
	require.ErrorContains(t, err, "invalid builder identity")

	// Key/certificate mismatch.
	other := signedTestIdentity(t, "example")
	mismatched := &builder.Identity{ID: identity.ID, Certificate: identity.Certificate, PrivateKey: other.PrivateKey}
	mismatchedPath := filepath.Join(dir, "mismatched.identity")
	require.NoError(t, os.WriteFile(mismatchedPath, marshalIdentityForTest(t, mismatched), 0600))
	_, err = builder.LoadIdentity(mismatchedPath)
	require.ErrorContains(t, err, "does not match")

	// Certificate CN doesn't correspond to the claimed id.
	tampered := &builder.Identity{ID: "someone-else", Certificate: identity.Certificate, PrivateKey: identity.PrivateKey}
	tamperedPath := filepath.Join(dir, "tampered.identity")
	require.NoError(t, os.WriteFile(tamperedPath, marshalIdentityForTest(t, tampered), 0600))
	_, err = builder.LoadIdentity(tamperedPath)
	require.ErrorContains(t, err, "id mismatch")
}

func TestEnsureIdentity_UsesExistingIdentity(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "builder.identity")
	identity := signedTestIdentity(t, "example")
	require.NoError(t, builder.SaveIdentity(path, identity))

	cfg := &builder.Config{IdentityFile: path, SupportedTargets: []string{"linux"}, Upstream: "http://127.0.0.1:1"}
	// No token set; EnsureIdentity must not need it since the identity already exists.
	t.Setenv("REALM_BUILDER_ENROLLMENT_TOKEN", "")
	loaded, err := builder.EnsureIdentity(t.Context(), cfg)
	require.NoError(t, err)
	require.Equal(t, "example", loaded.ID)
}

func TestEnsureIdentity_RequiresTokenWhenAbsent(t *testing.T) {
	cfg := &builder.Config{IdentityFile: filepath.Join(t.TempDir(), "builder.identity"), SupportedTargets: []string{"linux"}, Upstream: "http://127.0.0.1:1"}
	t.Setenv("REALM_BUILDER_ENROLLMENT_TOKEN", "")
	_, err := builder.EnsureIdentity(t.Context(), cfg)
	require.ErrorContains(t, err, "REALM_BUILDER_ENROLLMENT_TOKEN")
}

func TestEnsureIdentity_FailsWithoutExchangeOnMalformedIdentity(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "builder.identity")
	require.NoError(t, os.WriteFile(path, []byte("not: [valid"), 0600))

	cfg := &builder.Config{IdentityFile: path, SupportedTargets: []string{"linux"}, Upstream: "http://127.0.0.1:1"}
	// A bogus token proves the exchange was never attempted: if it were, this
	// unreachable upstream would surface a network error instead.
	t.Setenv("REALM_BUILDER_ENROLLMENT_TOKEN", "should-not-be-used")
	_, err := builder.EnsureIdentity(t.Context(), cfg)
	require.ErrorContains(t, err, "existing builder identity is invalid")
}

// marshalIdentityForTest writes an Identity the same way SaveIdentity would,
// without the overwrite guard, so tests can plant deliberately broken files.
func marshalIdentityForTest(t *testing.T, identity *builder.Identity) []byte {
	t.Helper()
	dir := t.TempDir()
	path := filepath.Join(dir, "identity")
	require.NoError(t, builder.SaveIdentity(path, identity))
	data, err := os.ReadFile(path)
	require.NoError(t, err)
	return data
}

func TestSaveIdentity_DoesNotClobberExisting(t *testing.T) {
	path := filepath.Join(t.TempDir(), "builder.identity")
	require.NoError(t, os.WriteFile(path, []byte("existing"), 0600))

	err := builder.SaveIdentity(path, signedTestIdentity(t, "abc"))
	require.ErrorContains(t, err, "already exists")

	data, err := os.ReadFile(path)
	require.NoError(t, err)
	require.Equal(t, "existing", string(data))

	entries, err := os.ReadDir(filepath.Dir(path))
	require.NoError(t, err)
	require.Len(t, entries, 1, "temporary files must be cleaned up")
}
