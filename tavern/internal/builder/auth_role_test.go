package builder_test

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"encoding/pem"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestBuilderAuthRejectsRedirectorCertificate(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()
	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	ca, err := builder.CreateCA(caKey)
	require.NoError(t, err)
	_, identityKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	csrDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, identityKey)
	require.NoError(t, err)
	csr := string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrDER}))
	certPEM, err := builder.SignRedirectorCertificate(ca, caKey, "example", csr)
	require.NoError(t, err)
	block, _ := pem.Decode(certPEM)
	require.NotNil(t, block)
	timestamp := time.Now().UTC().Format(time.RFC3339Nano)
	signature := ed25519.Sign(identityKey, []byte(timestamp))
	ctx := metadata.NewIncomingContext(context.Background(), metadata.Pairs(
		"builder-cert-bin", string(block.Bytes),
		"builder-signature-bin", string(signature),
		"builder-timestamp", timestamp,
	))
	_, err = builder.NewMTLSAuthInterceptor(ca, graph)(ctx, nil, &grpc.UnaryServerInfo{}, func(context.Context, any) (any, error) {
		t.Fatal("redirector certificate reached builder API")
		return nil, nil
	})
	require.Equal(t, codes.Unauthenticated, status.Code(err))
	require.Equal(t, "invalid certificate CN format", status.Convert(err).Message())
}
