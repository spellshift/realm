package http_test

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	tavernhttp "realm.pub/tavern/internal/http"
)

// pkceS256 computes the PKCE S256 code challenge from a verifier.
func pkceS256(verifier string) string {
	h := sha256.Sum256([]byte(verifier))
	return base64.RawURLEncoding.EncodeToString(h[:])
}

// setupMCPOAuthServer creates a test HTTP server with MCP OAuth handlers.
func setupMCPOAuthServer(t *testing.T) (*tavernhttp.Server, *ent.Client) {
	t.Helper()
	graph := enttest.OpenTempDB(t)

	routes := tavernhttp.RouteMap{
		"/.well-known/oauth-authorization-server": tavernhttp.Endpoint{
			Handler:              tavernhttp.NewOAuthServerMetadataHandler("https://tavern.example.com"),
			AllowUnauthenticated: true,
			AllowUnactivated:     true,
		},
		"/oauth/register": tavernhttp.Endpoint{
			Handler:              tavernhttp.NewOAuthDCRHandler(graph),
			AllowUnauthenticated: true,
			AllowUnactivated:     true,
		},
		"/oauth/token": tavernhttp.Endpoint{
			Handler:              tavernhttp.NewOAuthTokenHandler(graph),
			AllowUnauthenticated: true,
			AllowUnactivated:     true,
		},
		"/oauth/mcp/authorize": tavernhttp.Endpoint{
			Handler:              tavernhttp.NewOAuthMCPAuthorizeHandler(graph),
			AllowUnauthenticated: true,
			AllowUnactivated:     true,
		},
		"/": tavernhttp.Endpoint{
			Handler:              http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { w.WriteHeader(http.StatusOK) }),
			AllowUnauthenticated: true,
		},
	}

	srv := tavernhttp.NewServer(routes, tavernhttp.WithAuthentication(graph))
	return srv, graph
}

// TestOAuthServerMetadata verifies the RFC 8414 metadata endpoint.
func TestOAuthServerMetadata(t *testing.T) {
	srv, graph := setupMCPOAuthServer(t)
	defer graph.Close()

	req := httptest.NewRequest(http.MethodGet, "/.well-known/oauth-authorization-server", nil)
	w := httptest.NewRecorder()
	srv.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
	assert.Contains(t, w.Header().Get("Content-Type"), "application/json")

	var meta map[string]interface{}
	require.NoError(t, json.Unmarshal(w.Body.Bytes(), &meta))

	assert.Equal(t, "https://tavern.example.com", meta["issuer"])
	assert.Equal(t, "https://tavern.example.com/oauth/mcp/authorize", meta["authorization_endpoint"])
	assert.Equal(t, "https://tavern.example.com/oauth/token", meta["token_endpoint"])
	assert.Equal(t, "https://tavern.example.com/oauth/register", meta["registration_endpoint"])
}

// TestOAuthServerMetadataDerivesIssuerFromRequest verifies the issuer is derived from the request
// when no issuer is configured.
func TestOAuthServerMetadataDerivesIssuerFromRequest(t *testing.T) {
	handler := tavernhttp.NewOAuthServerMetadataHandler("")

	req := httptest.NewRequest(http.MethodGet, "/.well-known/oauth-authorization-server", nil)
	req.Host = "myhost.example.com"
	w := httptest.NewRecorder()
	handler.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
	var meta map[string]interface{}
	require.NoError(t, json.Unmarshal(w.Body.Bytes(), &meta))
	assert.Equal(t, "http://myhost.example.com", meta["issuer"])
}

// TestOAuthDCR verifies the Dynamic Client Registration endpoint.
func TestOAuthDCR(t *testing.T) {
	srv, graph := setupMCPOAuthServer(t)
	defer graph.Close()

	body := `{"client_name":"Test MCP Client","redirect_uris":["https://ai.example.com/callback"]}`
	req := httptest.NewRequest(http.MethodPost, "/oauth/register", strings.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	srv.ServeHTTP(w, req)

	assert.Equal(t, http.StatusCreated, w.Code)

	var resp map[string]interface{}
	require.NoError(t, json.Unmarshal(w.Body.Bytes(), &resp))
	assert.NotEmpty(t, resp["client_id"])
	assert.Equal(t, "Test MCP Client", resp["client_name"])
}

// TestOAuthDCRMissingRedirectURIs verifies that DCR rejects requests without redirect_uris.
func TestOAuthDCRMissingRedirectURIs(t *testing.T) {
	srv, graph := setupMCPOAuthServer(t)
	defer graph.Close()

	body := `{"client_name":"Test"}`
	req := httptest.NewRequest(http.MethodPost, "/oauth/register", strings.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	srv.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
}

// TestOAuthMCPAuthorizeRedirectsWhenUnauthenticated verifies that the authorize endpoint
// redirects unauthenticated users to the login page.
func TestOAuthMCPAuthorizeRedirectsWhenUnauthenticated(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	// Register a client first
	ctx := context.Background()
	oauthClient := graph.OAuthClient.Create().
		SetClientID("test-client-id").
		SetClientName("Test Client").
		SetRedirectUris([]string{"https://ai.example.com/callback"}).
		SaveX(ctx)

	handler := tavernhttp.NewOAuthMCPAuthorizeHandler(graph)

	q := url.Values{
		"client_id":             {oauthClient.ClientID},
		"redirect_uri":          {"https://ai.example.com/callback"},
		"state":                 {"mystate"},
		"code_challenge":        {"ABCDE12345"},
		"code_challenge_method": {"S256"},
	}
	req := httptest.NewRequest(http.MethodGet, "/oauth/mcp/authorize?"+q.Encode(), nil)
	w := httptest.NewRecorder()
	handler.ServeHTTP(w, req)

	// Must redirect to login
	assert.Equal(t, http.StatusFound, w.Code)
	assert.Contains(t, w.Header().Get("Location"), "/oauth/login")
	assert.Contains(t, w.Header().Get("Location"), "next=")
}

// TestOAuthMCPAuthorizeShowsConsentPage verifies authenticated users see the consent page.
func TestOAuthMCPAuthorizeShowsConsentPage(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	ctx := context.Background()
	// Create a user
	user := graph.User.Create().
		SetName("testuser").
		SetOauthID("oauth-test").
		SetPhotoURL("https://example.com/photo.jpg").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	// Register a client
	oauthClient := graph.OAuthClient.Create().
		SetClientID("test-client-id").
		SetClientName("Test Client").
		SetRedirectUris([]string{"https://ai.example.com/callback"}).
		SaveX(ctx)

	handler := tavernhttp.NewOAuthMCPAuthorizeHandler(graph)

	q := url.Values{
		"client_id":             {oauthClient.ClientID},
		"redirect_uri":          {"https://ai.example.com/callback"},
		"state":                 {"mystate"},
		"code_challenge":        {"ABCDE12345"},
		"code_challenge_method": {"S256"},
	}
	req := httptest.NewRequest(http.MethodGet, "/oauth/mcp/authorize?"+q.Encode(), nil)
	// Inject auth context
	authCtx, err := auth.ContextFromSessionToken(ctx, graph, user.SessionToken)
	require.NoError(t, err)
	req = req.WithContext(authCtx)

	w := httptest.NewRecorder()
	handler.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
	assert.Contains(t, w.Header().Get("Content-Type"), "text/html")
	assert.Contains(t, w.Body.String(), "Test Client")
	assert.Contains(t, w.Body.String(), "Authorize Access")
}

// TestOAuthMCPAuthorizePOSTAllow verifies the consent form submission creates an auth code.
func TestOAuthMCPAuthorizePOSTAllow(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	ctx := context.Background()
	user := graph.User.Create().
		SetName("testuser").
		SetOauthID("oauth-test").
		SetPhotoURL("https://example.com/photo.jpg").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	oauthClient := graph.OAuthClient.Create().
		SetClientID("test-client-id").
		SetClientName("Test Client").
		SetRedirectUris([]string{"https://ai.example.com/callback"}).
		SaveX(ctx)

	handler := tavernhttp.NewOAuthMCPAuthorizeHandler(graph)

	formData := url.Values{
		"action":                {"allow"},
		"client_id":             {oauthClient.ClientID},
		"redirect_uri":          {"https://ai.example.com/callback"},
		"state":                 {"mystate"},
		"code_challenge":        {pkceS256("myverifier")},
		"code_challenge_method": {"S256"},
	}
	req := httptest.NewRequest(http.MethodPost, "/oauth/mcp/authorize", strings.NewReader(formData.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	authCtx, err := auth.ContextFromSessionToken(ctx, graph, user.SessionToken)
	require.NoError(t, err)
	req = req.WithContext(authCtx)

	w := httptest.NewRecorder()
	handler.ServeHTTP(w, req)

	assert.Equal(t, http.StatusFound, w.Code)
	loc := w.Header().Get("Location")
	assert.Contains(t, loc, "https://ai.example.com/callback")
	assert.Contains(t, loc, "code=")
	assert.Contains(t, loc, "state=mystate")
}

// TestOAuthMCPAuthorizePOSTDeny verifies the deny action redirects with access_denied error.
func TestOAuthMCPAuthorizePOSTDeny(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	ctx := context.Background()
	user := graph.User.Create().
		SetName("testuser").
		SetOauthID("oauth-test").
		SetPhotoURL("https://example.com/photo.jpg").
		SetIsActivated(true).
		SaveX(ctx)

	oauthClient := graph.OAuthClient.Create().
		SetClientID("test-client-id").
		SetClientName("Test Client").
		SetRedirectUris([]string{"https://ai.example.com/callback"}).
		SaveX(ctx)

	handler := tavernhttp.NewOAuthMCPAuthorizeHandler(graph)

	formData := url.Values{
		"action":       {"deny"},
		"client_id":    {oauthClient.ClientID},
		"redirect_uri": {"https://ai.example.com/callback"},
		"state":        {"mystate"},
	}
	req := httptest.NewRequest(http.MethodPost, "/oauth/mcp/authorize", strings.NewReader(formData.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	authCtx, err := auth.ContextFromSessionToken(ctx, graph, user.SessionToken)
	require.NoError(t, err)
	req = req.WithContext(authCtx)

	w := httptest.NewRecorder()
	handler.ServeHTTP(w, req)

	assert.Equal(t, http.StatusFound, w.Code)
	loc := w.Header().Get("Location")
	assert.Contains(t, loc, "error=access_denied")
	assert.Contains(t, loc, "state=mystate")
}

// TestOAuthFullFlow verifies the complete DCR OAuth flow: register → authorize → token.
func TestOAuthFullFlow(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	ctx := context.Background()
	user := graph.User.Create().
		SetName("testuser").
		SetOauthID("oauth-test").
		SetPhotoURL("https://example.com/photo.jpg").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	authorizeHandler := tavernhttp.NewOAuthMCPAuthorizeHandler(graph)
	tokenHandler := tavernhttp.NewOAuthTokenHandler(graph)

	codeVerifier := "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"
	codeChallenge := pkceS256(codeVerifier)

	// Register a client
	oauthClient := graph.OAuthClient.Create().
		SetClientID("test-client-id").
		SetClientName("AI Assistant").
		SetRedirectUris([]string{"https://ai.example.com/callback"}).
		SaveX(ctx)

	// Step 1: POST to authorize (user allows)
	formData := url.Values{
		"action":                {"allow"},
		"client_id":             {oauthClient.ClientID},
		"redirect_uri":          {"https://ai.example.com/callback"},
		"state":                 {"random-state"},
		"code_challenge":        {codeChallenge},
		"code_challenge_method": {"S256"},
	}
	authReq := httptest.NewRequest(http.MethodPost, "/oauth/mcp/authorize", strings.NewReader(formData.Encode()))
	authReq.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	authCtx, err := auth.ContextFromSessionToken(ctx, graph, user.SessionToken)
	require.NoError(t, err)
	authReq = authReq.WithContext(authCtx)

	authW := httptest.NewRecorder()
	authorizeHandler.ServeHTTP(authW, authReq)
	require.Equal(t, http.StatusFound, authW.Code)

	// Extract code from redirect
	loc, err := url.Parse(authW.Header().Get("Location"))
	require.NoError(t, err)
	code := loc.Query().Get("code")
	require.NotEmpty(t, code)

	// Step 2: POST to token
	tokenBody := url.Values{
		"grant_type":    {"authorization_code"},
		"code":          {code},
		"redirect_uri":  {"https://ai.example.com/callback"},
		"client_id":     {oauthClient.ClientID},
		"code_verifier": {codeVerifier},
	}
	tokenReq := httptest.NewRequest(http.MethodPost, "/oauth/token", bytes.NewBufferString(tokenBody.Encode()))
	tokenReq.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	tokenW := httptest.NewRecorder()
	tokenHandler.ServeHTTP(tokenW, tokenReq)
	require.Equal(t, http.StatusOK, tokenW.Code)

	var tokenResp map[string]interface{}
	require.NoError(t, json.Unmarshal(tokenW.Body.Bytes(), &tokenResp))
	assert.Equal(t, user.AccessToken, tokenResp["access_token"])
	assert.Equal(t, "Bearer", tokenResp["token_type"])
}

// TestOAuthTokenRejectsReusedCode verifies authorization codes cannot be reused.
func TestOAuthTokenRejectsReusedCode(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	ctx := context.Background()
	user := graph.User.Create().
		SetName("testuser").
		SetOauthID("oauth-test").
		SetPhotoURL("https://example.com/photo.jpg").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	authorizeHandler := tavernhttp.NewOAuthMCPAuthorizeHandler(graph)
	tokenHandler := tavernhttp.NewOAuthTokenHandler(graph)

	codeVerifier := "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"
	codeChallenge := pkceS256(codeVerifier)

	oauthClient := graph.OAuthClient.Create().
		SetClientID("test-client-id").
		SetClientName("AI Assistant").
		SetRedirectUris([]string{"https://ai.example.com/callback"}).
		SaveX(ctx)

	// Issue code
	formData := url.Values{
		"action":                {"allow"},
		"client_id":             {oauthClient.ClientID},
		"redirect_uri":          {"https://ai.example.com/callback"},
		"state":                 {"state"},
		"code_challenge":        {codeChallenge},
		"code_challenge_method": {"S256"},
	}
	authReq := httptest.NewRequest(http.MethodPost, "/oauth/mcp/authorize", strings.NewReader(formData.Encode()))
	authReq.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	authCtx, _ := auth.ContextFromSessionToken(ctx, graph, user.SessionToken)
	authReq = authReq.WithContext(authCtx)

	authW := httptest.NewRecorder()
	authorizeHandler.ServeHTTP(authW, authReq)
	loc, _ := url.Parse(authW.Header().Get("Location"))
	code := loc.Query().Get("code")

	tokenBody := url.Values{
		"grant_type":    {"authorization_code"},
		"code":          {code},
		"redirect_uri":  {"https://ai.example.com/callback"},
		"client_id":     {oauthClient.ClientID},
		"code_verifier": {codeVerifier},
	}

	// First use: success
	r1 := httptest.NewRequest(http.MethodPost, "/oauth/token", bytes.NewBufferString(tokenBody.Encode()))
	r1.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	w1 := httptest.NewRecorder()
	tokenHandler.ServeHTTP(w1, r1)
	assert.Equal(t, http.StatusOK, w1.Code)

	// Second use: rejected
	r2 := httptest.NewRequest(http.MethodPost, "/oauth/token", bytes.NewBufferString(tokenBody.Encode()))
	r2.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	w2 := httptest.NewRecorder()
	tokenHandler.ServeHTTP(w2, r2)
	assert.Equal(t, http.StatusBadRequest, w2.Code)
}

// TestBearerTokenAuthentication verifies that Authorization: Bearer <token> authenticates requests.
func TestBearerTokenAuthentication(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	existingUser := graph.User.Create().
		SetName("testuser").
		SetOauthID("oauth-bearer-test").
		SetPhotoURL("https://example.com/photo.jpg").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/": tavernhttp.Endpoint{
				Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { w.WriteHeader(http.StatusOK) }),
			},
		},
		tavernhttp.WithAuthentication(graph),
	)

	tests := []struct {
		name     string
		header   string
		wantCode int
	}{
		{
			name:     "ValidBearerToken",
			header:   "Bearer " + existingUser.AccessToken,
			wantCode: http.StatusOK,
		},
		{
			name:     "InvalidBearerToken",
			header:   "Bearer invalid-token-value",
			wantCode: http.StatusUnauthorized,
		},
		{
			name:     "EmptyBearerToken",
			header:   "Bearer ",
			wantCode: http.StatusUnauthorized,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, "/", nil)
			req.Header.Set("Authorization", tc.header)
			w := httptest.NewRecorder()
			srv.ServeHTTP(w, req)
			assert.Equal(t, tc.wantCode, w.Code)
		})
	}
}

// TestWWWAuthenticateHeader verifies the WWW-Authenticate header is sent on 401 responses.
func TestWWWAuthenticateHeader(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/mcp/": tavernhttp.Endpoint{
				Handler:         http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { w.WriteHeader(http.StatusOK) }),
				WWWAuthenticate: `Bearer realm="Tavern"`,
			},
		},
		tavernhttp.WithAuthentication(graph),
	)

	req := httptest.NewRequest(http.MethodGet, "/mcp/", nil)
	w := httptest.NewRecorder()
	srv.ServeHTTP(w, req)

	assert.Equal(t, http.StatusUnauthorized, w.Code)
	assert.Equal(t, `Bearer realm="Tavern"`, w.Header().Get("WWW-Authenticate"))
}
