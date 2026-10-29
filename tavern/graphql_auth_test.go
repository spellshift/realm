package main

import (
	"bytes"
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"encoding/json"
	"encoding/pem"
	"io"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/tomes"
)

// newGraphQLAuthTestServer wires up the GraphQL route exactly as NewServer
// does: a real (non-bypass) authenticator, with the route itself allowing
// unauthenticated HTTP requests through so that AroundOperations is
// responsible for rejecting them at the operation level.
func newGraphQLAuthTestServer(t *testing.T) *httptest.Server {
	t.Helper()
	graph := enttest.OpenTempDB(t)
	t.Cleanup(func() { graph.Close() })

	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	caCert, err := builder.CreateCA(caKey)
	require.NoError(t, err)

	git := tomes.NewGitImporter(graph)
	gqlHandler := newGraphQLHandler(graph, git, graphql.WithBuilderCA(caCert), graphql.WithBuilderCAKey(caKey))
	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": tavernhttp.Endpoint{
				Handler:              gqlHandler,
				AllowUnauthenticated: true,
				AllowUnactivated:     true,
			},
		},
		tavernhttp.WithAuthentication(graph),
	)
	httpSrv := httptest.NewServer(srv)
	t.Cleanup(httpSrv.Close)

	// Seed an admin user for the authenticated-request assertions.
	graph.User.Create().
		SetOauthID("test_oauth_id").
		SetPhotoURL("https://photos.com").
		SetName("test").
		SetSessionToken("admin-session-token").
		SetAccessToken("access-token").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(t.Context())

	return httpSrv
}

// doGraphQLRequest POSTs a raw GraphQL query (no variables needed by these
// tests) to srv, optionally authenticated via sessionToken, and returns the
// response status and body.
func doGraphQLRequest(t *testing.T, srv *httptest.Server, query, sessionToken string) (status int, body string) {
	t.Helper()
	return doGraphQLRequestVars(t, srv, query, nil, sessionToken)
}

// doGraphQLRequestVars is doGraphQLRequest with GraphQL variables.
func doGraphQLRequestVars(t *testing.T, srv *httptest.Server, query string, variables map[string]any, sessionToken string) (status int, body string) {
	t.Helper()
	payload, err := json.Marshal(map[string]any{"query": query, "variables": variables})
	require.NoError(t, err)

	req, err := http.NewRequest(http.MethodPost, srv.URL+"/graphql", bytes.NewReader(payload))
	require.NoError(t, err)
	req.Header.Set("Content-Type", "application/json")
	if sessionToken != "" {
		req.AddCookie(&http.Cookie{
			Name:    auth.SessionCookieName,
			Value:   sessionToken,
			Expires: time.Now().Add(24 * time.Hour),
		})
	}

	resp, err := http.DefaultClient.Do(req)
	require.NoError(t, err)
	defer resp.Body.Close()

	data, err := io.ReadAll(resp.Body)
	require.NoError(t, err)
	return resp.StatusCode, string(data)
}

// TestGraphQLAnonymousOperationGating verifies that unauthenticated requests
// to /graphql are rejected before any field resolves, with a single,
// narrow exception: an operation that is solely exchangeBuilderEnrollmentToken.
func TestGraphQLAnonymousOperationGating(t *testing.T) {
	srv := newGraphQLAuthTestServer(t)

	t.Run("anonymous exchangeBuilderEnrollmentToken reaches the resolver", func(t *testing.T) {
		_, body := doGraphQLRequest(t, srv, `mutation {
			exchangeBuilderEnrollmentToken(token: "bogus", input: {supportedTargets: [PLATFORM_LINUX]}, csr: "invalid") { builderID }
		}`, "")
		// Rejected by the resolver (unknown token), not by the auth gate.
		assert.Contains(t, body, "invalid or used builder enrollment token")
		assert.NotContains(t, body, "permission denied: unauthenticated")
	})

	t.Run("anonymous createBuilderEnrollmentToken is rejected before resolving", func(t *testing.T) {
		_, body := doGraphQLRequest(t, srv, `mutation { createBuilderEnrollmentToken }`, "")
		assert.Contains(t, body, "permission denied: unauthenticated")
	})

	t.Run("anonymous me query is rejected before resolving", func(t *testing.T) {
		_, body := doGraphQLRequest(t, srv, `query { me { id } }`, "")
		assert.Contains(t, body, "permission denied: unauthenticated")
	})

	t.Run("anonymous introspection is rejected before resolving", func(t *testing.T) {
		_, body := doGraphQLRequest(t, srv, `query { __schema { queryType { name } } }`, "")
		assert.Contains(t, body, "permission denied: unauthenticated")
	})

	t.Run("anonymous node query is rejected before resolving", func(t *testing.T) {
		_, body := doGraphQLRequest(t, srv, `query { node(id: "1") { id } }`, "")
		assert.Contains(t, body, "permission denied: unauthenticated")
	})

	t.Run("mixing exchangeBuilderEnrollmentToken with another field is rejected", func(t *testing.T) {
		_, body := doGraphQLRequest(t, srv, `mutation {
			exchangeBuilderEnrollmentToken(token: "bogus", input: {supportedTargets: [PLATFORM_LINUX]}, csr: "invalid") { builderID }
			createBuilderEnrollmentToken
		}`, "")
		assert.Contains(t, body, "permission denied: unauthenticated")
	})

	t.Run("authenticated admin behavior is unchanged", func(t *testing.T) {
		_, body := doGraphQLRequest(t, srv, `mutation { createBuilderEnrollmentToken }`, "admin-session-token")
		assert.NotContains(t, body, "permission denied")
		assert.Contains(t, body, "createBuilderEnrollmentToken")
	})
}

func newTestCSR(t *testing.T) string {
	t.Helper()
	_, key, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	der, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, key)
	require.NoError(t, err)
	return string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: der}))
}

const exchangeMutation = `mutation($token: String!, $input: CreateBuilderInput!, $csr: String!) {
	exchangeBuilderEnrollmentToken(token: $token, input: $input, csr: $csr) { builderID mtlsCert }
}`

// TestGraphQLAnonymousExchangeSucceeds issues a token as an admin, then
// redeems it anonymously through the real authenticator.
func TestGraphQLAnonymousExchangeSucceeds(t *testing.T) {
	srv := newGraphQLAuthTestServer(t)

	_, body := doGraphQLRequest(t, srv, `mutation { createBuilderEnrollmentToken }`, "admin-session-token")
	var created struct {
		Data struct{ CreateBuilderEnrollmentToken string } `json:"data"`
	}
	require.NoError(t, json.Unmarshal([]byte(body), &created))
	token := created.Data.CreateBuilderEnrollmentToken
	require.NotEmpty(t, token)

	vars := map[string]any{
		"token": token,
		"csr":   newTestCSR(t),
		"input": map[string]any{"supportedTargets": []string{"PLATFORM_LINUX"}, "upstream": "https://tavern.example.com"},
	}
	_, body = doGraphQLRequestVars(t, srv, exchangeMutation, vars, "")
	var resp struct {
		Data struct {
			Exchange struct{ BuilderID, MtlsCert string } `json:"exchangeBuilderEnrollmentToken"`
		} `json:"data"`
		Errors []any `json:"errors"`
	}
	require.NoError(t, json.Unmarshal([]byte(body), &resp))
	require.Empty(t, resp.Errors, body)
	assert.NotEmpty(t, resp.Data.Exchange.BuilderID)
	assert.Contains(t, resp.Data.Exchange.MtlsCert, "BEGIN CERTIFICATE")

	vars["csr"] = newTestCSR(t)
	_, body = doGraphQLRequestVars(t, srv, exchangeMutation, vars, "")
	assert.Contains(t, body, "invalid or used builder enrollment token")
}

// TestGraphQLEnrollmentTokenNotExposed verifies token rows are not part of
// the GraphQL schema, so node/nodes cannot reach them even for an admin.
func TestGraphQLEnrollmentTokenNotExposed(t *testing.T) {
	srv := newGraphQLAuthTestServer(t)
	doGraphQLRequest(t, srv, `mutation { createBuilderEnrollmentToken }`, "admin-session-token")

	_, body := doGraphQLRequest(t, srv, `query { __type(name: "BuilderEnrollmentToken") { name } }`, "admin-session-token")
	assert.Contains(t, body, `"__type":null`)

	_, body = doGraphQLRequest(t, srv, `query { node(id: "1") { __typename ... on BuilderEnrollmentToken { id } } }`, "admin-session-token")
	assert.Contains(t, body, "errors")
}

type captureHandler struct {
	mu  *sync.Mutex
	buf *strings.Builder
}

func (h captureHandler) Enabled(context.Context, slog.Level) bool { return true }
func (h captureHandler) WithAttrs([]slog.Attr) slog.Handler       { return h }
func (h captureHandler) WithGroup(string) slog.Handler            { return h }
func (h captureHandler) Handle(_ context.Context, r slog.Record) error {
	h.mu.Lock()
	defer h.mu.Unlock()
	h.buf.WriteString(r.Message + "\n")
	r.Attrs(func(a slog.Attr) bool {
		h.buf.WriteString(a.Key + "=" + a.Value.String() + "\n")
		return true
	})
	return nil
}

// TestGraphQLLogsRedactEnrollmentSecrets verifies the enrollment token and
// CSR never reach logs, whether passed as variables or inlined in the query.
func TestGraphQLLogsRedactEnrollmentSecrets(t *testing.T) {
	t.Setenv(EnvLogGraphQLRawQuery.String(), "1")
	h := captureHandler{mu: &sync.Mutex{}, buf: &strings.Builder{}}
	prev := slog.Default()
	slog.SetDefault(slog.New(h))
	t.Cleanup(func() { slog.SetDefault(prev) })

	srv := newGraphQLAuthTestServer(t)
	input := map[string]any{"supportedTargets": []string{"PLATFORM_LINUX"}}

	doGraphQLRequestVars(t, srv, exchangeMutation, map[string]any{"token": "SECRET-VAR-TOKEN", "csr": "SECRET-VAR-CSR", "input": input}, "")
	doGraphQLRequestVars(t, srv, `mutation($t: String!) {
		exchangeBuilderEnrollmentToken(token: $t, input: {supportedTargets: [PLATFORM_LINUX]}, csr: "SECRET-INLINE-CSR") { builderID }
	}`, map[string]any{"t": "SECRET-RENAMED-TOKEN"}, "")
	doGraphQLRequestVars(t, srv, `mutation($t: String!, $c: String!) { ...Exchange }
	fragment Exchange on Mutation {
		exchangeBuilderEnrollmentToken(token: $t, input: {supportedTargets: [PLATFORM_LINUX]}, csr: $c) { builderID }
	}`, map[string]any{"t": "SECRET-FRAGMENT-TOKEN", "c": "SECRET-FRAGMENT-CSR"}, "admin-session-token")
	doGraphQLRequest(t, srv, `mutation($t: String! = "SECRET-DEFAULT-TOKEN") {
		exchangeBuilderEnrollmentToken(token: $t, input: {supportedTargets: [PLATFORM_LINUX]}, csr: "x") { builderID }
	}`, "")
	doGraphQLRequest(t, srv, `mutation {
		exchangeBuilderEnrollmentToken(token: "SECRET-INLINE-TOKEN", input: {supportedTargets: [PLATFORM_LINUX]}, csr: "x") { builderID }
	}`, "")

	h.mu.Lock()
	logs := h.buf.String()
	h.mu.Unlock()
	require.Contains(t, logs, "tavern graphql request")
	assert.NotContains(t, logs, "SECRET-")
	assert.Contains(t, logs, redactedValue)
}
