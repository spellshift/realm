package auth_test

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"
	"time"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/user"
	"realm.pub/tavern/internal/oauthproxy"
)

func TestProxyLoginHandlerSetsNonceCookieAndPassesNonce(t *testing.T) {
	returnTo := "https://tenant.run.app/oauth/complete"
	h := auth.NewProxyLoginHandler("https://proxy.run.app/", returnTo)

	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/oauth/login", nil))

	require.Equal(t, http.StatusFound, rec.Code)
	loc, err := url.Parse(rec.Header().Get("Location"))
	require.NoError(t, err)
	assert.Equal(t, "proxy.run.app", loc.Host)
	assert.Equal(t, "/oauth/start", loc.Path)
	assert.Equal(t, returnTo, loc.Query().Get("return_to"))

	cookie := findCookie(rec.Result().Cookies(), auth.OAuthProxyNonceCookieName)
	require.NotNil(t, cookie, "nonce cookie must be set")
	assert.NotEmpty(t, cookie.Value)
	assert.Equal(t, cookie.Value, loc.Query().Get("nonce"))
	assert.True(t, cookie.HttpOnly)
	assert.True(t, cookie.Secure)
	assert.Equal(t, http.SameSiteLaxMode, cookie.SameSite)
}

func TestProxyAssertionHandlerRejectsMissingNonceCookie(t *testing.T) {
	// A valid-looking request with no login cookie must be rejected before any key fetch or DB use.
	h := auth.NewProxyAssertionHandler(nil, nil, "https://tenant.run.app/oauth/complete")

	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/oauth/complete?assertion=abc", nil))

	assert.Equal(t, http.StatusBadRequest, rec.Code)
}

func newTestKeySource(t *testing.T) (*oauthproxy.KeySource, ed25519.PrivateKey) {
	t.Helper()
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	jwksData, err := oauthproxy.EncodeJWKS(pub)
	require.NoError(t, err)
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(jwksData)
	}))
	t.Cleanup(srv.Close)
	return &oauthproxy.KeySource{JWKSURL: srv.URL}, priv
}

func findCookie(cookies []*http.Cookie, name string) *http.Cookie {
	for _, c := range cookies {
		if c.Name == name {
			return c
		}
	}
	return nil
}

func TestProxyAssertionHandlerHappyPath(t *testing.T) {
	returnTo := "https://tenant.run.app/oauth/complete"
	ks, priv := newTestKeySource(t)
	graph := enttest.OpenTempDB(t)
	t.Cleanup(func() { require.NoError(t, graph.Close()) })

	h := auth.NewProxyAssertionHandler(ks, graph, returnTo)

	// 1. First user registers via TOFU as admin + activated.
	nonce1 := "browser-nonce-1"
	assertion1, err := oauthproxy.SignAssertion(priv, oauthproxy.Profile{
		OAuthID:       "sub-alice",
		Email:         "alice@example.com",
		Name:          "Alice",
		PhotoURL:      "https://example.com/alice.png",
		EmailVerified: true,
	}, returnTo, nonce1, time.Minute)
	require.NoError(t, err)

	req := httptest.NewRequest(http.MethodGet, "/oauth/complete?assertion="+url.QueryEscape(assertion1), nil)
	req.AddCookie(&http.Cookie{Name: auth.OAuthProxyNonceCookieName, Value: nonce1})
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)

	require.Equal(t, http.StatusFound, rec.Code)
	assert.Equal(t, "/", rec.Header().Get("Location"))

	// Verify nonce cookie was cleared with matching attributes.
	cleared := findCookie(rec.Result().Cookies(), auth.OAuthProxyNonceCookieName)
	require.NotNil(t, cleared, "nonce cookie must be cleared after successful verification")
	assert.Equal(t, "", cleared.Value)
	assert.Equal(t, -1, cleared.MaxAge)
	assert.True(t, cleared.Secure)
	assert.True(t, cleared.HttpOnly)
	assert.Equal(t, http.SameSiteLaxMode, cleared.SameSite)
	assert.True(t, cleared.Expires.Equal(time.Unix(0, 0).UTC()))

	// Verify TOFU user + session cookie.
	alice := graph.User.Query().Where(user.OauthID("sub-alice")).OnlyX(context.Background())
	assert.Equal(t, "Alice", alice.Name)
	assert.Equal(t, "https://example.com/alice.png", alice.PhotoURL)
	assert.True(t, alice.IsAdmin)
	assert.True(t, alice.IsActivated)
	sess := findCookie(rec.Result().Cookies(), auth.SessionCookieName)
	require.NotNil(t, sess)
	assert.Equal(t, alice.SessionToken, sess.Value)

	// 2. Second distinct user registers as non-admin, unactivated.
	nonce2 := "browser-nonce-2"
	assertion2, err := oauthproxy.SignAssertion(priv, oauthproxy.Profile{
		OAuthID: "sub-bob",
		Name:    "Bob",
	}, returnTo, nonce2, time.Minute)
	require.NoError(t, err)

	req2 := httptest.NewRequest(http.MethodGet, "/oauth/complete?assertion="+url.QueryEscape(assertion2), nil)
	req2.AddCookie(&http.Cookie{Name: auth.OAuthProxyNonceCookieName, Value: nonce2})
	rec2 := httptest.NewRecorder()
	h.ServeHTTP(rec2, req2)

	require.Equal(t, http.StatusFound, rec2.Code)
	bob := graph.User.Query().Where(user.OauthID("sub-bob")).OnlyX(context.Background())
	assert.False(t, bob.IsAdmin)
	assert.False(t, bob.IsActivated)
}

func TestProxyAssertionHandlerLocalHTTPClearsInsecureCookie(t *testing.T) {
	returnTo := "http://localhost:8000/oauth/complete"
	ks, priv := newTestKeySource(t)
	graph := enttest.OpenTempDB(t)
	t.Cleanup(func() { require.NoError(t, graph.Close()) })

	h := auth.NewProxyAssertionHandler(ks, graph, returnTo)

	nonce := "local-nonce"
	assertion, err := oauthproxy.SignAssertion(priv, oauthproxy.Profile{
		OAuthID: "sub-local",
		Name:    "Local",
	}, returnTo, nonce, time.Minute)
	require.NoError(t, err)

	req := httptest.NewRequest(http.MethodGet, "/oauth/complete?assertion="+url.QueryEscape(assertion), nil)
	req.AddCookie(&http.Cookie{Name: auth.OAuthProxyNonceCookieName, Value: nonce})
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)

	require.Equal(t, http.StatusFound, rec.Code)
	cleared := findCookie(rec.Result().Cookies(), auth.OAuthProxyNonceCookieName)
	require.NotNil(t, cleared)
	assert.False(t, cleared.Secure)
	assert.True(t, cleared.HttpOnly)
	assert.Equal(t, http.SameSiteLaxMode, cleared.SameSite)
	assert.Equal(t, -1, cleared.MaxAge)
}

func TestProxyAssertionHandlerErrorsPreserveNonceCookie(t *testing.T) {
	returnTo := "https://tenant.run.app/oauth/complete"
	ks, priv := newTestKeySource(t)
	_, otherPriv, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	graph := enttest.OpenTempDB(t)
	t.Cleanup(func() { require.NoError(t, graph.Close()) })

	h := auth.NewProxyAssertionHandler(ks, graph, returnTo)

	validNonce := "expected-nonce"
	expiredAssertion, err := oauthproxy.SignAssertion(priv, oauthproxy.Profile{OAuthID: "sub-1"}, returnTo, validNonce, -time.Minute)
	require.NoError(t, err)
	wrongAudAssertion, err := oauthproxy.SignAssertion(priv, oauthproxy.Profile{OAuthID: "sub-1"}, "https://other.run.app/oauth/complete", validNonce, time.Minute)
	require.NoError(t, err)
	wrongNonceAssertion, err := oauthproxy.SignAssertion(priv, oauthproxy.Profile{OAuthID: "sub-1"}, returnTo, "different-nonce", time.Minute)
	require.NoError(t, err)
	wrongKeyAssertion, err := oauthproxy.SignAssertion(otherPriv, oauthproxy.Profile{OAuthID: "sub-1"}, returnTo, validNonce, time.Minute)
	require.NoError(t, err)

	cases := []struct {
		name      string
		assertion string
		wantCode  int
	}{
		{"missing assertion", "", http.StatusBadRequest},
		{"malformed assertion", "not-a-jwt", http.StatusUnauthorized},
		{"expired assertion", expiredAssertion, http.StatusUnauthorized},
		{"wrong audience", wrongAudAssertion, http.StatusUnauthorized},
		{"nonce mismatch", wrongNonceAssertion, http.StatusUnauthorized},
		{"wrong signing key", wrongKeyAssertion, http.StatusUnauthorized},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			target := "/oauth/complete"
			if tc.assertion != "" {
				target += "?assertion=" + url.QueryEscape(tc.assertion)
			}
			req := httptest.NewRequest(http.MethodGet, target, nil)
			req.AddCookie(&http.Cookie{Name: auth.OAuthProxyNonceCookieName, Value: validNonce})
			rec := httptest.NewRecorder()
			h.ServeHTTP(rec, req)

			assert.Equal(t, tc.wantCode, rec.Code)
			assert.Nil(t, findCookie(rec.Result().Cookies(), auth.OAuthProxyNonceCookieName),
				"failed verification must not clear in-flight login nonce cookie")
			assert.Nil(t, findCookie(rec.Result().Cookies(), auth.SessionCookieName))
		})
	}
}
