package http_test

import (
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"encoding/base64"
	"encoding/pem"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/ent/enttest"
	tavernhttp "realm.pub/tavern/internal/http"
)

func TestRedirectorCheckinRejectsBuilderCertificate(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()
	_, err := graph.Redirector.Create().SetIdentifier("example").SetTransport("http1").SetDomains([]string{"r.example.com"}).Save(t.Context())
	require.NoError(t, err)
	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	ca, err := builder.CreateCA(caKey)
	require.NoError(t, err)
	_, key, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	csrDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, key)
	require.NoError(t, err)
	csr := string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrDER}))

	for _, tc := range []struct {
		name     string
		sign     func(*x509.Certificate, ed25519.PrivateKey, string, string) ([]byte, error)
		rejected bool
	}{
		{"builder", builder.SignBuilderCertificate, true},
		{"redirector", builder.SignRedirectorCertificate, false},
	} {
		t.Run(tc.name, func(t *testing.T) {
			certPEM, err := tc.sign(ca, caKey, "example", csr)
			require.NoError(t, err)
			block, _ := pem.Decode(certPEM)
			require.NotNil(t, block)
			timestamp := time.Now().UTC().Format(time.RFC3339Nano)
			req := httptest.NewRequest(http.MethodPost, "/auth/redirector/checkin", nil)
			req.Header.Set("X-Realm-Redirector-Cert", base64.StdEncoding.EncodeToString(block.Bytes))
			req.Header.Set("X-Realm-Redirector-Timestamp", timestamp)
			req.Header.Set("X-Realm-Redirector-Signature", base64.StdEncoding.EncodeToString(ed25519.Sign(key, []byte(timestamp))))
			res := httptest.NewRecorder()
			tavernhttp.NewRedirectorCheckinHandler(graph, ca)(res, req)
			if tc.rejected {
				require.Equal(t, http.StatusUnauthorized, res.Code)
				return
			}
			require.GreaterOrEqual(t, res.Code, http.StatusOK)
			require.Less(t, res.Code, http.StatusMultipleChoices)
		})
	}
}
