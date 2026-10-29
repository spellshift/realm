package main

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent/enttest"
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

	git := tomes.NewGitImporter(graph)
	gqlHandler := newGraphQLHandler(graph, git)
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
	payload, err := json.Marshal(map[string]any{"query": query})
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
