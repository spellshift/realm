package builder_test

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"encoding/pem"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"sync/atomic"
	"testing"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/require"

	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/tomes"
)

// newTestBuilderServer boots a Tavern GraphQL server (authentication bypassed,
// so the caller is treated as an admin) with the given Builder CA, listening
// on a real loopback address so builders can perform a genuine HTTP token
// exchange against it.
func newTestBuilderServer(t *testing.T, graph *ent.Client, caCert *x509.Certificate, caKey ed25519.PrivateKey) (*client.Client, string) {
	t.Helper()
	git := tomes.NewGitImporter(graph)
	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": handler.NewDefaultServer(graphql.NewSchema(graph, git, graphql.WithBuilderCA(caCert), graphql.WithBuilderCAKey(caKey))),
		},
		tavernhttp.WithAuthenticationBypass(graph),
	)
	httpSrv := httptest.NewServer(srv)
	t.Cleanup(httpSrv.Close)
	return client.New(srv, client.Path("/graphql")), httpSrv.URL
}

// enrollTestBuilder issues a one-time enrollment token via GraphQL as an
// admin, then exchanges it for a builder identity exactly as a real builder
// would (REALM_BUILDER_ENROLLMENT_TOKEN + EnsureIdentity), returning a ready
// to use Config pointed at upstream.
func enrollTestBuilder(t *testing.T, ctx context.Context, gqlClient *client.Client, upstream string, supportedTargets ...string) *builder.Config {
	t.Helper()

	var tokenResp struct{ CreateBuilderEnrollmentToken string }
	require.NoError(t, gqlClient.Post(`mutation { createBuilderEnrollmentToken }`, &tokenResp))
	require.NotEmpty(t, tokenResp.CreateBuilderEnrollmentToken)

	cfg := &builder.Config{
		PollInterval:     builder.DefaultPollInterval,
		SupportedTargets: supportedTargets,
		IdentityFile:     filepath.Join(t.TempDir(), "builder.identity"),
		Upstream:         upstream,
		Executor:         "local",
	}
	t.Setenv("REALM_BUILDER_ENROLLMENT_TOKEN", tokenResp.CreateBuilderEnrollmentToken)
	_, err := builder.EnsureIdentity(ctx, cfg)
	require.NoError(t, err)
	return cfg
}

// createTestEnrollmentToken issues a one-time enrollment token via GraphQL,
// for tests that need to drive exchangeBuilderEnrollmentToken directly.
func createTestEnrollmentToken(t *testing.T, gqlClient *client.Client) string {
	t.Helper()
	var resp struct{ CreateBuilderEnrollmentToken string }
	require.NoError(t, gqlClient.Post(`mutation { createBuilderEnrollmentToken }`, &resp))
	require.NotEmpty(t, resp.CreateBuilderEnrollmentToken)
	return resp.CreateBuilderEnrollmentToken
}

// saveIdentityAndConfig persists an Identity assembled from the pieces
// returned by a raw exchangeBuilderEnrollmentToken call (builder ID,
// certificate, and the private key matching the CSR that was submitted),
// returning a Config ready for NewCredentialsFromConfig.
func saveIdentityAndConfig(t *testing.T, builderID, certPEM, keyPEM, upstream string, supportedTargets ...string) *builder.Config {
	t.Helper()
	path := filepath.Join(t.TempDir(), "builder.identity")
	require.NoError(t, builder.SaveIdentity(path, &builder.Identity{ID: builderID, Certificate: certPEM, PrivateKey: keyPEM}))
	return &builder.Config{
		PollInterval:     builder.DefaultPollInterval,
		SupportedTargets: supportedTargets,
		IdentityFile:     path,
		Upstream:         upstream,
		Executor:         "local",
	}
}

// generateTestCSR creates a fresh Ed25519 key and PEM-encoded CSR for tests
// driving exchangeBuilderEnrollmentToken directly at the GraphQL layer.
func generateTestCSR(t *testing.T) (csrPEM string, keyPEM string) {
	t.Helper()
	_, key, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	csrDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, key)
	require.NoError(t, err)
	keyDER, err := x509.MarshalPKCS8PrivateKey(key)
	require.NoError(t, err)
	return string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrDER})),
		string(pem.EncodeToMemory(&pem.Block{Type: "PRIVATE KEY", Bytes: keyDER}))
}

// useHTTPClient swaps the enrollment HTTP client for the duration of a test.
func useHTTPClient(t *testing.T, c *http.Client) {
	t.Helper()
	prev := builder.HTTPClient
	builder.HTTPClient = c
	t.Cleanup(func() { builder.HTTPClient = prev })
}

func newTestCA(t *testing.T) (*x509.Certificate, ed25519.PrivateKey) {
	t.Helper()
	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	caCert, err := builder.CreateCA(caKey)
	require.NoError(t, err)
	return caCert, caKey
}

func TestEnsureIdentity_HTTPSUpstream(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()
	caCert, caKey := newTestCA(t)

	git := tomes.NewGitImporter(graph)
	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": handler.NewDefaultServer(graphql.NewSchema(graph, git, graphql.WithBuilderCA(caCert), graphql.WithBuilderCAKey(caKey))),
		},
		tavernhttp.WithAuthenticationBypass(graph),
	)
	tlsSrv := httptest.NewTLSServer(srv)
	defer tlsSrv.Close()
	require.True(t, strings.HasPrefix(tlsSrv.URL, "https://"))

	// Without trusting the test certificate the exchange must fail and leave
	// the token untouched.
	gqlClient := client.New(srv, client.Path("/graphql"))
	token := createTestEnrollmentToken(t, gqlClient)
	t.Setenv("REALM_BUILDER_ENROLLMENT_TOKEN", token)
	cfg := &builder.Config{
		PollInterval:     builder.DefaultPollInterval,
		SupportedTargets: []string{"linux"},
		IdentityFile:     filepath.Join(t.TempDir(), "builder.identity"),
		Upstream:         tlsSrv.URL,
		Executor:         "local",
	}
	_, err := builder.EnsureIdentity(context.Background(), cfg)
	require.Error(t, err)
	require.NoFileExists(t, cfg.IdentityFile)

	useHTTPClient(t, tlsSrv.Client())
	identity, err := builder.EnsureIdentity(context.Background(), cfg)
	require.NoError(t, err)
	require.NotEmpty(t, identity.ID)

	info, err := os.Stat(cfg.IdentityFile)
	require.NoError(t, err)
	require.Equal(t, os.FileMode(0600), info.Mode().Perm())

	count, err := graph.Builder.Query().Count(context.Background())
	require.NoError(t, err)
	require.Equal(t, 1, count)
}

func TestEnsureIdentity_PreflightDoesNotConsumeToken(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()
	caCert, caKey := newTestCA(t)
	gqlClient, upstream := newTestBuilderServer(t, graph, caCert, caKey)

	token := createTestEnrollmentToken(t, gqlClient)
	t.Setenv("REALM_BUILDER_ENROLLMENT_TOKEN", token)

	// A regular file in place of the parent directory makes it unusable.
	blocker := filepath.Join(t.TempDir(), "blocker")
	require.NoError(t, os.WriteFile(blocker, nil, 0600))
	cfg := &builder.Config{
		PollInterval:     builder.DefaultPollInterval,
		SupportedTargets: []string{"linux"},
		IdentityFile:     filepath.Join(blocker, "builder.identity"),
		Upstream:         upstream,
		Executor:         "local",
	}
	_, err := builder.EnsureIdentity(context.Background(), cfg)
	require.ErrorContains(t, err, "cannot be saved")

	// The token is still redeemable once the destination is fixed.
	cfg.IdentityFile = filepath.Join(t.TempDir(), "nested", "builder.identity")
	_, err = builder.EnsureIdentity(context.Background(), cfg)
	require.NoError(t, err)
}

func TestEnsureIdentity_DoesNotFollowRedirects(t *testing.T) {
	var targetHits atomic.Int32
	target := httptest.NewServer(http.HandlerFunc(func(http.ResponseWriter, *http.Request) { targetHits.Add(1) }))
	defer target.Close()
	redirector := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Redirect(w, r, target.URL+"/graphql", http.StatusTemporaryRedirect)
	}))
	defer redirector.Close()

	t.Setenv("REALM_BUILDER_ENROLLMENT_TOKEN", "some-token")
	cfg := &builder.Config{
		PollInterval:     builder.DefaultPollInterval,
		SupportedTargets: []string{"linux"},
		IdentityFile:     filepath.Join(t.TempDir(), "builder.identity"),
		Upstream:         redirector.URL,
		Executor:         "local",
	}
	_, err := builder.EnsureIdentity(context.Background(), cfg)
	require.ErrorContains(t, err, "HTTP status 307")
	require.Zero(t, targetHits.Load())
	require.NoFileExists(t, cfg.IdentityFile)
}
