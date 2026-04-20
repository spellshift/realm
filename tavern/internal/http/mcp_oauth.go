package http

import (
	"crypto/rand"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/oauthclient"
	"realm.pub/tavern/internal/ent/oauthcode"
)

// newSecureID generates a cryptographically random, URL-safe token.
func newSecureID() string {
	buf := make([]byte, 32)
	if _, err := io.ReadFull(rand.Reader, buf); err != nil {
		panic(fmt.Errorf("failed to generate secure ID: %w", err))
	}
	return base64.RawURLEncoding.EncodeToString(buf)
}

// oauthServerMetadata is the RFC 8414 authorization server metadata response.
type oauthServerMetadata struct {
	Issuer                            string   `json:"issuer"`
	AuthorizationEndpoint             string   `json:"authorization_endpoint"`
	TokenEndpoint                     string   `json:"token_endpoint"`
	RegistrationEndpoint              string   `json:"registration_endpoint"`
	ResponseTypesSupported            []string `json:"response_types_supported"`
	GrantTypesSupported               []string `json:"grant_types_supported"`
	CodeChallengeMethodsSupported     []string `json:"code_challenge_methods_supported"`
	TokenEndpointAuthMethodsSupported []string `json:"token_endpoint_auth_methods_supported"`
}

// NewOAuthServerMetadataHandler returns an http.Handler that serves the RFC 8414
// OAuth 2.0 Authorization Server Metadata at /.well-known/oauth-authorization-server.
func NewOAuthServerMetadataHandler(issuer string) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		// Derive issuer from request if not configured
		effectiveIssuer := issuer
		if effectiveIssuer == "" {
			scheme := "http"
			if r.TLS != nil || r.Header.Get("X-Forwarded-Proto") == "https" {
				scheme = "https"
			}
			effectiveIssuer = fmt.Sprintf("%s://%s", scheme, r.Host)
		}
		effectiveIssuer = strings.TrimRight(effectiveIssuer, "/")

		metadata := oauthServerMetadata{
			Issuer:                effectiveIssuer,
			AuthorizationEndpoint: effectiveIssuer + "/oauth/mcp/authorize",
			TokenEndpoint:         effectiveIssuer + "/oauth/token",
			RegistrationEndpoint:  effectiveIssuer + "/oauth/register",
			ResponseTypesSupported:            []string{"code"},
			GrantTypesSupported:               []string{"authorization_code"},
			CodeChallengeMethodsSupported:     []string{"S256"},
			TokenEndpointAuthMethodsSupported: []string{"none"},
		}

		w.Header().Set("Content-Type", "application/json")
		w.Header().Set("Cache-Control", "no-store")
		json.NewEncoder(w).Encode(metadata)
	})
}

// dcrRequest is the RFC 7591 Dynamic Client Registration request body.
type dcrRequest struct {
	ClientName    string   `json:"client_name"`
	RedirectURIs  []string `json:"redirect_uris"`
	GrantTypes    []string `json:"grant_types"`
	ResponseTypes []string `json:"response_types"`
}

// dcrResponse is the RFC 7591 Dynamic Client Registration response body.
type dcrResponse struct {
	ClientID              string   `json:"client_id"`
	ClientName            string   `json:"client_name,omitempty"`
	RedirectURIs          []string `json:"redirect_uris"`
	GrantTypes            []string `json:"grant_types"`
	ResponseTypes         []string `json:"response_types"`
	TokenEndpointAuthMethod string `json:"token_endpoint_auth_method"`
}

// NewOAuthDCRHandler returns an http.Handler implementing RFC 7591 Dynamic Client Registration
// at POST /oauth/register.
func NewOAuthDCRHandler(client *ent.Client) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req dcrRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}

		if len(req.RedirectURIs) == 0 {
			http.Error(w, "redirect_uris is required", http.StatusBadRequest)
			return
		}

		// Validate redirect URIs are absolute https (or http for localhost)
		for _, uri := range req.RedirectURIs {
			u, err := url.Parse(uri)
			if err != nil || !u.IsAbs() {
				http.Error(w, fmt.Sprintf("invalid redirect_uri: %q", uri), http.StatusBadRequest)
				return
			}
		}

		clientID := newSecureID()
		oauthClient, err := client.OAuthClient.Create().
			SetClientID(clientID).
			SetClientName(req.ClientName).
			SetRedirectUris(req.RedirectURIs).
			Save(r.Context())
		if err != nil {
			http.Error(w, "failed to register client", http.StatusInternalServerError)
			return
		}

		resp := dcrResponse{
			ClientID:              oauthClient.ClientID,
			ClientName:            oauthClient.ClientName,
			RedirectURIs:          oauthClient.RedirectUris,
			GrantTypes:            []string{"authorization_code"},
			ResponseTypes:         []string{"code"},
			TokenEndpointAuthMethod: "none",
		}

		w.Header().Set("Content-Type", "application/json")
		w.Header().Set("Cache-Control", "no-store")
		w.WriteHeader(http.StatusCreated)
		json.NewEncoder(w).Encode(resp)
	})
}

// authorizeConsentPage is the HTML template for the MCP authorization consent page.
const authorizeConsentPage = `<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Authorize Access - Tavern</title>
<style>
body{font-family:sans-serif;max-width:480px;margin:80px auto;padding:0 16px;color:#222}
h1{font-size:1.4rem;margin-bottom:8px}
.client{font-weight:bold;color:#1a56db}
.card{background:#f9fafb;border:1px solid #e5e7eb;border-radius:8px;padding:20px;margin:24px 0}
.actions{display:flex;gap:12px;margin-top:24px}
button{flex:1;padding:10px 16px;border-radius:6px;border:none;cursor:pointer;font-size:1rem}
.allow{background:#1a56db;color:#fff}
.allow:hover{background:#1e40af}
.deny{background:#f3f4f6;color:#374151;border:1px solid #d1d5db}
.deny:hover{background:#e5e7eb}
</style>
</head>
<body>
<h1>Authorize Access</h1>
<div class="card">
<p><span class="client">%s</span> is requesting access to your Tavern account.</p>
<p>This will allow the application to interact with Tavern on your behalf, including running quests and querying data.</p>
</div>
<form method="POST" action="/oauth/mcp/authorize">
<input type="hidden" name="client_id" value="%s">
<input type="hidden" name="redirect_uri" value="%s">
<input type="hidden" name="state" value="%s">
<input type="hidden" name="code_challenge" value="%s">
<input type="hidden" name="code_challenge_method" value="%s">
<div class="actions">
<button type="submit" name="action" value="allow" class="allow">Allow</button>
<button type="submit" name="action" value="deny" class="deny">Deny</button>
</div>
</form>
</body>
</html>`

// NewOAuthMCPAuthorizeHandler returns an http.Handler for the authorization endpoint at
// GET+POST /oauth/mcp/authorize. GET shows the consent page; POST processes the user's decision.
func NewOAuthMCPAuthorizeHandler(client *ent.Client) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			handleMCPAuthorizeGET(w, r, client)
		case http.MethodPost:
			handleMCPAuthorizePOST(w, r, client)
		default:
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		}
	})
}

func handleMCPAuthorizeGET(w http.ResponseWriter, r *http.Request, client *ent.Client) {
	q := r.URL.Query()
	clientID := q.Get("client_id")
	redirectURI := q.Get("redirect_uri")
	state := q.Get("state")
	codeChallenge := q.Get("code_challenge")
	codeChallengeMethod := q.Get("code_challenge_method")
	if codeChallengeMethod == "" {
		codeChallengeMethod = "S256"
	}

	if clientID == "" || redirectURI == "" || codeChallenge == "" {
		http.Error(w, "missing required parameters: client_id, redirect_uri, code_challenge", http.StatusBadRequest)
		return
	}

	// If user is not authenticated, redirect to login with ?next= so they return here
	if !auth.IsAuthenticatedContext(r.Context()) {
		loginURL := "/oauth/login?next=" + url.QueryEscape(r.URL.RequestURI())
		http.Redirect(w, r, loginURL, http.StatusFound)
		return
	}

	// Validate the client exists and redirect_uri is registered
	oauthCli, err := client.OAuthClient.Query().Where(oauthclient.ClientID(clientID)).Only(r.Context())
	if err != nil {
		http.Error(w, "unknown client_id", http.StatusBadRequest)
		return
	}
	if !containsString(oauthCli.RedirectUris, redirectURI) {
		http.Error(w, "redirect_uri not registered for this client", http.StatusBadRequest)
		return
	}

	clientName := oauthCli.ClientName
	if clientName == "" {
		clientName = clientID
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	w.Header().Set("Cache-Control", "no-store")
	fmt.Fprintf(w, authorizeConsentPage,
		htmlEscape(clientName),
		htmlEscape(clientID),
		htmlEscape(redirectURI),
		htmlEscape(state),
		htmlEscape(codeChallenge),
		htmlEscape(codeChallengeMethod),
	)
}

func handleMCPAuthorizePOST(w http.ResponseWriter, r *http.Request, client *ent.Client) {
	if err := r.ParseForm(); err != nil {
		http.Error(w, "failed to parse form", http.StatusBadRequest)
		return
	}

	action := r.FormValue("action")
	clientID := r.FormValue("client_id")
	redirectURI := r.FormValue("redirect_uri")
	state := r.FormValue("state")
	codeChallenge := r.FormValue("code_challenge")
	codeChallengeMethod := r.FormValue("code_challenge_method")

	if clientID == "" || redirectURI == "" {
		http.Error(w, "missing required form fields", http.StatusBadRequest)
		return
	}

	// Build redirect URL helper
	redirectWithParams := func(params url.Values) {
		u, _ := url.Parse(redirectURI)
		u.RawQuery = params.Encode()
		http.Redirect(w, r, u.String(), http.StatusFound)
	}

	if action == "deny" {
		redirectWithParams(url.Values{
			"error":             {"access_denied"},
			"error_description": {"The user denied the authorization request"},
			"state":             {state},
		})
		return
	}

	// Require authenticated, activated user
	authUser := auth.UserFromContext(r.Context())
	if authUser == nil {
		http.Error(w, "must be authenticated", http.StatusUnauthorized)
		return
	}

	// Validate client
	oauthCli, err := client.OAuthClient.Query().Where(oauthclient.ClientID(clientID)).Only(r.Context())
	if err != nil {
		http.Error(w, "unknown client_id", http.StatusBadRequest)
		return
	}
	if !containsString(oauthCli.RedirectUris, redirectURI) {
		http.Error(w, "redirect_uri not registered for this client", http.StatusBadRequest)
		return
	}

	if codeChallenge == "" {
		http.Error(w, "code_challenge is required", http.StatusBadRequest)
		return
	}

	if codeChallengeMethod == "" {
		codeChallengeMethod = "S256"
	}

	// Generate authorization code
	code := newSecureID()
	_, err = client.OAuthCode.Create().
		SetCode(code).
		SetRedirectURI(redirectURI).
		SetCodeChallenge(codeChallenge).
		SetCodeChallengeMethod(codeChallengeMethod).
		SetExpiresAt(time.Now().Add(10 * time.Minute)).
		SetClient(oauthCli).
		SetUser(authUser).
		Save(r.Context())
	if err != nil {
		http.Error(w, "failed to create authorization code", http.StatusInternalServerError)
		return
	}

	redirectWithParams(url.Values{
		"code":  {code},
		"state": {state},
	})
}

// tokenRequest is the OAuth 2.0 token request body.
type tokenRequest struct {
	GrantType    string `json:"grant_type"`
	Code         string `json:"code"`
	RedirectURI  string `json:"redirect_uri"`
	ClientID     string `json:"client_id"`
	CodeVerifier string `json:"code_verifier"`
}

// tokenResponse is the OAuth 2.0 token response body.
type tokenResponse struct {
	AccessToken string `json:"access_token"`
	TokenType   string `json:"token_type"`
}

// tokenErrorResponse is the OAuth 2.0 error response body.
type tokenErrorResponse struct {
	Error            string `json:"error"`
	ErrorDescription string `json:"error_description,omitempty"`
}

// NewOAuthTokenHandler returns an http.Handler for the token endpoint at POST /oauth/token.
func NewOAuthTokenHandler(client *ent.Client) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		writeError := func(status int, errCode, desc string) {
			w.Header().Set("Content-Type", "application/json")
			w.Header().Set("Cache-Control", "no-store")
			w.WriteHeader(status)
			json.NewEncoder(w).Encode(tokenErrorResponse{Error: errCode, ErrorDescription: desc})
		}

		// Parse either JSON or form-encoded body
		var req tokenRequest
		contentType := r.Header.Get("Content-Type")
		if strings.Contains(contentType, "application/x-www-form-urlencoded") {
			if err := r.ParseForm(); err != nil {
				writeError(http.StatusBadRequest, "invalid_request", "failed to parse form body")
				return
			}
			req.GrantType = r.FormValue("grant_type")
			req.Code = r.FormValue("code")
			req.RedirectURI = r.FormValue("redirect_uri")
			req.ClientID = r.FormValue("client_id")
			req.CodeVerifier = r.FormValue("code_verifier")
		} else {
			if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
				writeError(http.StatusBadRequest, "invalid_request", "failed to parse JSON body")
				return
			}
		}

		if req.GrantType != "authorization_code" {
			writeError(http.StatusBadRequest, "unsupported_grant_type", "only authorization_code grant type is supported")
			return
		}
		if req.Code == "" || req.RedirectURI == "" || req.ClientID == "" || req.CodeVerifier == "" {
			writeError(http.StatusBadRequest, "invalid_request", "missing required parameters")
			return
		}

		// Look up the authorization code
		authCode, err := client.OAuthCode.Query().
			Where(oauthcode.Code(req.Code)).
			WithClient().
			WithUser().
			Only(r.Context())
		if err != nil {
			writeError(http.StatusBadRequest, "invalid_grant", "authorization code not found")
			return
		}

		// Check expiry
		if time.Now().After(authCode.ExpiresAt) {
			writeError(http.StatusBadRequest, "invalid_grant", "authorization code has expired")
			return
		}

		// Check already claimed
		if authCode.Claimed {
			writeError(http.StatusBadRequest, "invalid_grant", "authorization code has already been used")
			return
		}

		// Check client_id matches
		if authCode.Edges.Client == nil || authCode.Edges.Client.ClientID != req.ClientID {
			writeError(http.StatusBadRequest, "invalid_grant", "client_id does not match authorization code")
			return
		}

		// Check redirect_uri matches
		if authCode.RedirectURI != req.RedirectURI {
			writeError(http.StatusBadRequest, "invalid_grant", "redirect_uri does not match authorization code")
			return
		}

		// Verify PKCE code_verifier
		if !verifyPKCE(authCode.CodeChallengeMethod, authCode.CodeChallenge, req.CodeVerifier) {
			writeError(http.StatusBadRequest, "invalid_grant", "PKCE code_verifier verification failed")
			return
		}

		// Mark as claimed
		_, err = client.OAuthCode.UpdateOne(authCode).SetClaimed(true).Save(r.Context())
		if err != nil {
			writeError(http.StatusInternalServerError, "server_error", "failed to claim authorization code")
			return
		}

		if authCode.Edges.User == nil {
			writeError(http.StatusInternalServerError, "server_error", "authorization code has no associated user")
			return
		}

		w.Header().Set("Content-Type", "application/json")
		w.Header().Set("Cache-Control", "no-store")
		json.NewEncoder(w).Encode(tokenResponse{
			AccessToken: authCode.Edges.User.AccessToken,
			TokenType:   "Bearer",
		})
	})
}

// verifyPKCE verifies the PKCE code_verifier against the stored code_challenge.
func verifyPKCE(method, challenge, verifier string) bool {
	switch strings.ToUpper(method) {
	case "S256":
		h := sha256.Sum256([]byte(verifier))
		computed := base64.RawURLEncoding.EncodeToString(h[:])
		return computed == challenge
	case "PLAIN":
		return verifier == challenge
	default:
		return false
	}
}

// containsString returns true if s is in the slice ss.
func containsString(ss []string, s string) bool {
	for _, v := range ss {
		if v == s {
			return true
		}
	}
	return false
}

// htmlEscape escapes special HTML characters to prevent XSS in the consent page.
func htmlEscape(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	s = strings.ReplaceAll(s, `"`, "&#34;")
	s = strings.ReplaceAll(s, "'", "&#39;")
	return s
}
