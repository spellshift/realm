package auth_test

import (
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/auth"
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

	var cookie *http.Cookie
	for _, c := range rec.Result().Cookies() {
		if c.Name == auth.OAuthProxyNonceCookieName {
			cookie = c
		}
	}
	require.NotNil(t, cookie, "nonce cookie must be set")
	assert.NotEmpty(t, cookie.Value)
	assert.Equal(t, cookie.Value, loc.Query().Get("nonce"))
	assert.True(t, cookie.HttpOnly)
	assert.True(t, cookie.Secure)
}

func TestProxyAssertionHandlerRejectsMissingNonceCookie(t *testing.T) {
	// A valid-looking request with no login cookie must be rejected before any key fetch or DB use.
	h := auth.NewProxyAssertionHandler(nil, nil, "https://tenant.run.app/oauth/complete")

	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/oauth/complete?assertion=abc", nil))

	assert.Equal(t, http.StatusBadRequest, rec.Code)
}
