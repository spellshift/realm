package auth_test

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"net/url"
	"sync"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/oauth2"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/user"
	"realm.pub/tavern/internal/logging"
)

type oauthAuditCapture struct {
	mu      sync.Mutex
	entries []logging.AuditEntry
}

func (c *oauthAuditCapture) Enqueue(e logging.AuditEntry) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.entries = append(c.entries, e)
}

func (c *oauthAuditCapture) Snapshot() []logging.AuditEntry {
	c.mu.Lock()
	defer c.mu.Unlock()
	out := make([]logging.AuditEntry, len(c.entries))
	copy(out, c.entries)
	return out
}

// TestNewOAuthLoginHandler ensures the OAuth Login Handler exhibits expected behaviour
func TestNewOAuthLoginHandler(t *testing.T) {
	// Generate keys for signing JWTs
	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	// Configure OAuth
	cfg := oauth2.Config{
		ClientID:     "12345",
		ClientSecret: "SuperSecret",
		RedirectURL:  "https://test-idp.io/",
	}

	// Serve request
	handler := auth.NewOAuthLoginHandler(cfg, privKey)
	resp := httptest.NewRecorder()
	handler.ServeHTTP(resp, httptest.NewRequest(http.MethodGet, "/oauth/login", nil))

	// Ensure client was redirected
	result := resp.Result()
	redirectURL, err := result.Location()
	require.NoError(t, err)
	require.NotNil(t, redirectURL)
	assert.Equal(t, http.StatusFound, result.StatusCode)

	// Ensure state was set properly
	state := redirectURL.Query().Get("state")
	assert.NotEmpty(t, state)
	assert.GreaterOrEqual(t, len(state), 30)

	// Ensure OAuth cookie was set properly
	var oauthCookie *http.Cookie
	for _, cookie := range result.Cookies() {
		if cookie.Name == auth.OAuthCookieName {
			oauthCookie = cookie
			break
		}
	}
	require.NotNil(t, oauthCookie)
	assert.Equal(t, "/", oauthCookie.Path)
	assert.True(t, oauthCookie.HttpOnly)
	assert.True(t, oauthCookie.Secure)
	assert.Equal(t, http.SameSiteLaxMode, oauthCookie.SameSite)

	// Parse JWT
	var claims jwt.RegisteredClaims
	token, err := jwt.ParseWithClaims(oauthCookie.Value, &claims, func(token *jwt.Token) (interface{}, error) {
		assert.IsType(t, token.Method, &jwt.SigningMethodEd25519{})
		return pubKey, nil
	})
	require.NoError(t, err)
	assert.True(t, token.Valid)

	// JWT assertions
	assert.Equal(t, state, claims.ID)
	assert.GreaterOrEqual(t, claims.IssuedAt.Time.Unix(), time.Now().Add(-60*time.Second).Unix())
	assert.Greater(t, claims.ExpiresAt.Time.Unix(), time.Now().Add(9*time.Minute).Unix())
	assert.Less(t, claims.ExpiresAt.Time.Unix(), time.Now().Add(11*time.Minute).Unix())
}

// TestNewOAuthAuthorizationHandler ensures the OAuth Authorization Handler exhibits expected behaviour
func TestNewOAuthAuthorizationHandler(t *testing.T) {
	var (
		expectedClientID     = "12345"
		expectedClientSecret = "SuperSecret"
		expectedRedirect     = "REDIRECT_URL"
		expectedCode         = `SuperSecretAuthorization`
		expectedAccessToken  = `90d64460d14870c08c81352a05dedd3465940a7c`
		profileResp          = []byte(`{"sub":"goofygoober","picture":"photos.com/goofygoober","email":"goofygoober@google.com","name":"Mr. Goober","email_verified":true}`)
	)
	persister := &oauthAuditCapture{}
	logging.SetAuditPersister(persister)
	t.Cleanup(func() { logging.SetAuditPersister(nil) })

	// Setup Test DB
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	// Setup Mock IDP
	idp := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		body, err := io.ReadAll(r.Body)
		require.NoError(t, err)
		expected := fmt.Sprintf("client_id=%s&client_secret=%s&code=%s&grant_type=authorization_code&redirect_uri=%s", expectedClientID, expectedClientSecret, expectedCode, expectedRedirect)
		assert.Equal(t, expected, string(body))
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(fmt.Sprintf(`{"access_token": "%s", "scope": "user", "token_type": "bearer", "expires_in": 86400}`, expectedAccessToken)))
	}))
	defer idp.Close()

	// Setup Mock Resource Server
	rsrv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write(profileResp)
	}))
	defer rsrv.Close()

	// Generate keys for signing JWTs
	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	// Configure OAuth
	cfg := oauth2.Config{
		ClientID:     expectedClientID,
		ClientSecret: expectedClientSecret,
		RedirectURL:  expectedRedirect,
		Endpoint: oauth2.Endpoint{
			TokenURL:  idp.URL,
			AuthStyle: oauth2.AuthStyleInParams,
		},
	}

	// Assert no user already exists
	assert.Equal(t, 0, graph.User.Query().CountX(context.Background()))

	// Serve first request
	handler := auth.NewOAuthAuthorizationHandler(cfg, pubKey, graph, rsrv.URL)
	resp := httptest.NewRecorder()
	req := getOAuthAuthorizationRequest(t, privKey, expectedCode)
	handler.ServeHTTP(resp, req)

	// Result Assertions
	result := resp.Result()
	require.Equal(t, http.StatusFound, result.StatusCode)
	usr := graph.User.Query().
		Where(user.OauthID("goofygoober")).
		OnlyX(context.Background())
	assert.NotEmpty(t, usr.SessionToken)
	assert.True(t, usr.IsActivated)
	assert.True(t, usr.IsAdmin)
	assert.Equal(t, "Mr. Goober", usr.Name)
	assert.Equal(t, "photos.com/goofygoober", usr.PhotoURL)
	firstLoginCookie := getSessionCookie(result.Cookies())
	assert.Equal(t, usr.SessionToken, firstLoginCookie.Value)

	// Login again with the same user
	resp = httptest.NewRecorder()
	req = getOAuthAuthorizationRequest(t, privKey, expectedCode)
	handler.ServeHTTP(resp, req)
	result = resp.Result()
	require.Equal(t, http.StatusFound, result.StatusCode)

	// User assertions
	assert.Equal(t, 1, graph.User.Query().CountX(context.Background()))
	secondLoginCookie := getSessionCookie(result.Cookies())
	assert.Equal(t, usr.SessionToken, secondLoginCookie.Value)

	// Login with a new user
	profileResp = []byte(`{"sub":"n00b","picture":"photos.com/bestcatphoto","email":"noob@google.com","name":"1337 hax0r","email_verified":true}`)
	resp = httptest.NewRecorder()
	req = getOAuthAuthorizationRequest(t, privKey, expectedCode)
	handler.ServeHTTP(resp, req)
	result = resp.Result()
	require.Equal(t, http.StatusFound, result.StatusCode)

	// Assert new user was created
	assert.Equal(t, 2, graph.User.Query().CountX(context.Background()))

	// Second User assertions
	secondUsr := graph.User.Query().
		Where(user.OauthID("n00b")).
		OnlyX(context.Background())
	assert.NotEqual(t, usr.ID, secondUsr.ID)
	assert.NotEqual(t, usr.SessionToken, secondUsr.SessionToken)
	assert.False(t, secondUsr.IsAdmin)
	assert.False(t, secondUsr.IsActivated)

	// Verify semantic audit records
	entries := persister.Snapshot()
	require.Len(t, entries, 3)
	assert.Equal(t, "audit.auth", entries[0].Subsystem)
	assert.Equal(t, "oauth.login_succeeded", entries[0].Action)
	assert.Equal(t, true, entries[0].Details["new_user"])
	assert.Equal(t, true, entries[0].Details["is_admin"])
	assert.Equal(t, "oauth", entries[0].Actor.AuthMethod)

	assert.Equal(t, "oauth.login_succeeded", entries[1].Action)
	assert.Equal(t, false, entries[1].Details["new_user"])

	assert.Equal(t, "oauth.login_succeeded", entries[2].Action)
	assert.Equal(t, true, entries[2].Details["new_user"])
	assert.Equal(t, false, entries[2].Details["is_admin"])
}

func getSessionCookie(cookies []*http.Cookie) *http.Cookie {
	for _, cookie := range cookies {
		if cookie.Name == auth.SessionCookieName {
			return cookie
		}
	}
	panic(fmt.Errorf("did not find session cookie"))
}

func getOAuthAuthorizationRequest(t *testing.T, privKey ed25519.PrivateKey, code string) *http.Request {
	// Create a JWT
	expiresAt := time.Now().Add(10 * time.Minute)
	claims := jwt.RegisteredClaims{
		ID:        "ABCDEFG",
		IssuedAt:  jwt.NewNumericDate(time.Now()),
		ExpiresAt: jwt.NewNumericDate(expiresAt),
	}
	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	tokenStr, err := token.SignedString(privKey)
	require.NoError(t, err)

	req := httptest.NewRequest(http.MethodGet, "/oauth/authorize", nil)
	params := url.Values{
		"state": []string{claims.ID},
		"code":  []string{code},
	}
	req.AddCookie(&http.Cookie{
		Name:     auth.OAuthCookieName,
		Value:    tokenStr,
		Secure:   true,
		HttpOnly: true,
		Expires:  expiresAt,
	})
	req.URL.RawQuery = params.Encode()
	return req
}

// TestNewOAuthAuthorizationHandler_DBError ensures the OAuth Authorization Handler handles DB errors gracefully
func TestNewOAuthAuthorizationHandler_DBError(t *testing.T) {
	var (
		expectedClientID     = "12345"
		expectedClientSecret = "SuperSecret"
		expectedRedirect     = "REDIRECT_URL"
		expectedCode         = `SuperSecretAuthorization`
		expectedAccessToken  = `90d64460d14870c08c81352a05dedd3465940a7c`
		profileResp          = []byte(`{"sub":"goofygoober","picture":"photos.com/goofygoober","email":"goofygoober@google.com","name":"Mr. Goober","email_verified":true}`)
	)
	persister := &oauthAuditCapture{}
	logging.SetAuditPersister(persister)
	t.Cleanup(func() { logging.SetAuditPersister(nil) })

	// Setup Test DB
	graph := enttest.OpenTempDB(t)
	// We deliberately close the graph to trigger a DB error during query
	graph.Close()

	// Setup Mock IDP
	idp := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(fmt.Sprintf(`{"access_token": "%s", "scope": "user", "token_type": "bearer", "expires_in": 86400}`, expectedAccessToken)))
	}))
	defer idp.Close()

	// Setup Mock Resource Server
	rsrv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write(profileResp)
	}))
	defer rsrv.Close()

	// Generate keys for signing JWTs
	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	// Configure OAuth
	cfg := oauth2.Config{
		ClientID:     expectedClientID,
		ClientSecret: expectedClientSecret,
		RedirectURL:  expectedRedirect,
		Endpoint: oauth2.Endpoint{
			TokenURL:  idp.URL,
			AuthStyle: oauth2.AuthStyleInParams,
		},
	}

	handler := auth.NewOAuthAuthorizationHandler(cfg, pubKey, graph, rsrv.URL)
	resp := httptest.NewRecorder()
	req := getOAuthAuthorizationRequest(t, privKey, expectedCode)

	handler.ServeHTTP(resp, req)

	result := resp.Result()
	// We expect 500 Internal Server Error, and no panic
	assert.Equal(t, http.StatusInternalServerError, result.StatusCode)

	entries := persister.Snapshot()
	require.Len(t, entries, 1)
	assert.Equal(t, "oauth.login_failed", entries[0].Action)
	assert.Equal(t, logging.OutcomeFailure, entries[0].Outcome)

	// State cookie should still be cleared after callback verification
	clearedState := findCookie(result.Cookies(), auth.OAuthCookieName)
	require.NotNil(t, clearedState)
	assert.Equal(t, -1, clearedState.MaxAge)
	assert.Empty(t, clearedState.Value)
}

func findCookie(cookies []*http.Cookie, name string) *http.Cookie {
	for _, cookie := range cookies {
		if cookie.Name == name {
			return cookie
		}
	}
	return nil
}

func TestNewOAuthAuthorizationHandler_ClearsStateCookie(t *testing.T) {
	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	idp := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"access_token": "tok", "scope": "user", "token_type": "bearer", "expires_in": 86400}`))
	}))
	defer idp.Close()

	rsrv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte(`{"sub":"state-clear-user","picture":"photos.com/u","email":"u@example.com","name":"State User","email_verified":true}`))
	}))
	defer rsrv.Close()

	cfg := oauth2.Config{
		ClientID:     "12345",
		ClientSecret: "SuperSecret",
		RedirectURL:  "REDIRECT_URL",
		Endpoint: oauth2.Endpoint{
			TokenURL:  idp.URL,
			AuthStyle: oauth2.AuthStyleInParams,
		},
	}

	handler := auth.NewOAuthAuthorizationHandler(cfg, pubKey, graph, rsrv.URL)

	t.Run("cleared_after_valid_callback", func(t *testing.T) {
		resp := httptest.NewRecorder()
		req := getOAuthAuthorizationRequest(t, privKey, "valid-code")
		handler.ServeHTTP(resp, req)

		result := resp.Result()
		require.Equal(t, http.StatusFound, result.StatusCode)

		cleared := findCookie(result.Cookies(), auth.OAuthCookieName)
		require.NotNil(t, cleared)
		assert.Equal(t, -1, cleared.MaxAge)
		assert.Empty(t, cleared.Value)
		assert.Equal(t, "/", cleared.Path)
		assert.True(t, cleared.HttpOnly)
		assert.Equal(t, http.SameSiteLaxMode, cleared.SameSite)
	})

	t.Run("not_cleared_on_state_mismatch", func(t *testing.T) {
		resp := httptest.NewRecorder()
		req := getOAuthAuthorizationRequest(t, privKey, "valid-code")
		q := req.URL.Query()
		q.Set("state", "wrong-state")
		req.URL.RawQuery = q.Encode()
		handler.ServeHTTP(resp, req)

		result := resp.Result()
		require.Equal(t, http.StatusUnauthorized, result.StatusCode)
		assert.Nil(t, findCookie(result.Cookies(), auth.OAuthCookieName))
	})
}

func TestNewOAuthAuthorizationHandler_SessionCookieFlags(t *testing.T) {
	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	idp := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"access_token": "tok", "scope": "user", "token_type": "bearer", "expires_in": 86400}`))
	}))
	defer idp.Close()

	rsrv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte(`{"sub":"cookie-flags-user","picture":"photos.com/u","email":"u@example.com","name":"Cookie User","email_verified":true}`))
	}))
	defer rsrv.Close()

	cfg := oauth2.Config{
		ClientID:     "12345",
		ClientSecret: "SuperSecret",
		RedirectURL:  "REDIRECT_URL",
		Endpoint: oauth2.Endpoint{
			TokenURL:  idp.URL,
			AuthStyle: oauth2.AuthStyleInParams,
		},
	}

	handler := auth.NewOAuthAuthorizationHandler(cfg, pubKey, graph, rsrv.URL)

	tests := []struct {
		name           string
		target         string
		forwardedProto string
		wantSecure     bool
	}{
		{
			name:       "plain_http_localhost",
			target:     "http://localhost:8000/oauth/authorize",
			wantSecure: false,
		},
		{
			name:       "direct_https",
			target:     "https://tavern.example/oauth/authorize",
			wantSecure: true,
		},
		{
			name:           "x_forwarded_proto_https",
			target:         "http://localhost:8000/oauth/authorize",
			forwardedProto: "https",
			wantSecure:     true,
		},
		{
			name:           "x_forwarded_proto_chain_https_first",
			target:         "http://localhost:8000/oauth/authorize",
			forwardedProto: "https, http",
			wantSecure:     true,
		},
		{
			name:           "x_forwarded_proto_http",
			target:         "http://localhost:8000/oauth/authorize",
			forwardedProto: "http",
			wantSecure:     false,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			req := getOAuthAuthorizationRequest(t, privKey, "code")
			httpsReq := httptest.NewRequest(http.MethodGet, tc.target, nil)
			httpsReq.URL.RawQuery = req.URL.RawQuery
			for _, c := range req.Cookies() {
				httpsReq.AddCookie(c)
			}
			if tc.forwardedProto != "" {
				httpsReq.Header.Set("X-Forwarded-Proto", tc.forwardedProto)
			}

			resp := httptest.NewRecorder()
			handler.ServeHTTP(resp, httpsReq)

			result := resp.Result()
			require.Equal(t, http.StatusFound, result.StatusCode)

			sessionCookie := getSessionCookie(result.Cookies())
			assert.Equal(t, "/", sessionCookie.Path)
			assert.True(t, sessionCookie.HttpOnly)
			assert.Equal(t, http.SameSiteLaxMode, sessionCookie.SameSite)
			assert.Equal(t, tc.wantSecure, sessionCookie.Secure)
		})
	}
}

func TestNewOAuthAuthorizationHandler_UserCreationError(t *testing.T) {
	persister := &oauthAuditCapture{}
	logging.SetAuditPersister(persister)
	t.Cleanup(func() { logging.SetAuditPersister(nil) })

	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	idp := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"access_token": "tok", "scope": "user", "token_type": "bearer", "expires_in": 86400}`))
	}))
	defer idp.Close()

	// Name "ab" fails User.Name MinLen(3) validation during Save, previously panicking in SaveX.
	rsrv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte(`{"sub":"short-name-user","picture":"photos.com/u","email":"u@example.com","name":"ab","email_verified":true}`))
	}))
	defer rsrv.Close()

	cfg := oauth2.Config{
		ClientID:     "12345",
		ClientSecret: "SuperSecret",
		RedirectURL:  "REDIRECT_URL",
		Endpoint: oauth2.Endpoint{
			TokenURL:  idp.URL,
			AuthStyle: oauth2.AuthStyleInParams,
		},
	}

	handler := auth.NewOAuthAuthorizationHandler(cfg, pubKey, graph, rsrv.URL)
	resp := httptest.NewRecorder()
	req := getOAuthAuthorizationRequest(t, privKey, "code")

	assert.NotPanics(t, func() {
		handler.ServeHTTP(resp, req)
	})

	result := resp.Result()
	assert.Equal(t, http.StatusInternalServerError, result.StatusCode)
	assert.Equal(t, 0, graph.User.Query().CountX(context.Background()))

	entries := persister.Snapshot()
	require.Len(t, entries, 1)
	assert.Equal(t, "oauth.login_failed", entries[0].Action)
	assert.Equal(t, logging.OutcomeFailure, entries[0].Outcome)
	assert.Equal(t, "user_creation_failed", entries[0].Details["reason"])
}

func TestNewOAuthAuthorizationHandler_ConcurrentTOFU(t *testing.T) {
	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	idp := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = r.ParseForm()
		code := r.FormValue("code")
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(fmt.Sprintf(`{"access_token": "%s", "scope": "user", "token_type": "bearer", "expires_in": 86400}`, code)))
	}))
	defer idp.Close()

	rsrv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		authHeader := r.Header.Get("Authorization")
		sub := authHeader
		_, _ = w.Write([]byte(fmt.Sprintf(`{"sub":"%s","picture":"photos.com/u","email":"u@example.com","name":"User %s","email_verified":true}`, sub, sub[len(sub)-4:])))
	}))
	defer rsrv.Close()

	cfg := oauth2.Config{
		ClientID:     "12345",
		ClientSecret: "SuperSecret",
		RedirectURL:  "REDIRECT_URL",
		Endpoint: oauth2.Endpoint{
			TokenURL:  idp.URL,
			AuthStyle: oauth2.AuthStyleInParams,
		},
	}

	handler := auth.NewOAuthAuthorizationHandler(cfg, pubKey, graph, rsrv.URL)

	const numUsers = 6
	start := make(chan struct{})
	var wg sync.WaitGroup
	codes := make([]int, numUsers)

	for i := 0; i < numUsers; i++ {
		wg.Add(1)
		go func(idx int) {
			defer wg.Done()
			req := getOAuthAuthorizationRequest(t, privKey, fmt.Sprintf("user-%04d", idx))
			resp := httptest.NewRecorder()
			<-start
			handler.ServeHTTP(resp, req)
			codes[idx] = resp.Result().StatusCode
		}(i)
	}
	close(start)
	wg.Wait()

	for i, status := range codes {
		assert.Equal(t, http.StatusFound, status, "request %d failed", i)
	}

	users := graph.User.Query().AllX(context.Background())
	require.Len(t, users, numUsers)

	adminCount := 0
	activatedCount := 0
	for _, u := range users {
		if u.IsAdmin {
			adminCount++
		}
		if u.IsActivated {
			activatedCount++
		}
	}
	assert.Equal(t, 1, adminCount, "expected exactly one TOFU admin")
	assert.Equal(t, 1, activatedCount, "expected exactly one TOFU activated user")

	// Also verify concurrent first-time logins for the same OAuthID do not fail.
	const sameUserReqs = 4
	startSame := make(chan struct{})
	sameCodes := make([]int, sameUserReqs)
	for i := 0; i < sameUserReqs; i++ {
		wg.Add(1)
		go func(idx int) {
			defer wg.Done()
			req := getOAuthAuthorizationRequest(t, privKey, "same-9999")
			resp := httptest.NewRecorder()
			<-startSame
			handler.ServeHTTP(resp, req)
			sameCodes[idx] = resp.Result().StatusCode
		}(i)
	}
	close(startSame)
	wg.Wait()

	for i, status := range sameCodes {
		assert.Equal(t, http.StatusFound, status, "same-user request %d failed", i)
	}
	assert.Equal(t, numUsers+1, graph.User.Query().CountX(context.Background()))
}
