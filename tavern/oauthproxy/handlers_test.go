package main

import (
	"crypto/ed25519"
	"crypto/rand"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/oauth2"
	"realm.pub/tavern/internal/oauthproxy"
)

func mustTestKeyPair(t *testing.T) (ed25519.PublicKey, ed25519.PrivateKey) {
	t.Helper()
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	return pub, priv
}

func findRespCookie(cookies []*http.Cookie, name string) *http.Cookie {
	for _, c := range cookies {
		if c.Name == name {
			return c
		}
	}
	return nil
}

func TestStatusHandler(t *testing.T) {
	rec := httptest.NewRecorder()
	newStatusHandler().ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/status", nil))
	assert.Equal(t, http.StatusOK, rec.Code)
}

func TestJWKSHandler(t *testing.T) {
	pub, _ := mustTestKeyPair(t)
	srv := httptest.NewServer(newJWKSHandler(pub))
	defer srv.Close()

	ks := &oauthproxy.KeySource{JWKSURL: srv.URL}
	got, err := ks.PublicKey(t.Context())
	require.NoError(t, err)
	assert.True(t, got.Equal(pub))
}

func TestStartHandler(t *testing.T) {
	pub, priv := mustTestKeyPair(t)

	newCfg := func(proxyDomain string) *Config {
		return &Config{
			OAuth: oauth2.Config{
				ClientID:     "cid",
				ClientSecret: "csec",
				RedirectURL:  proxyDomain + "/oauth/callback",
				Endpoint: oauth2.Endpoint{
					AuthURL: "https://accounts.example.com/auth",
				},
			},
			AllowedSuffixes: []string{".run.app", "localhost"},
			ProxyDomain:     proxyDomain,
		}
	}

	t.Run("https sets secure state cookie and redirects", func(t *testing.T) {
		h := newStartHandler(newCfg("https://proxy.run.app"), priv)
		returnTo := "https://tenant.run.app/oauth/complete"
		req := httptest.NewRequest(http.MethodGet, "/oauth/start?return_to="+url.QueryEscape(returnTo)+"&nonce=client-nonce-1", nil)
		rec := httptest.NewRecorder()
		h.ServeHTTP(rec, req)

		require.Equal(t, http.StatusFound, rec.Code)
		loc, err := url.Parse(rec.Header().Get("Location"))
		require.NoError(t, err)
		assert.Equal(t, "accounts.example.com", loc.Host)

		cookie := findRespCookie(rec.Result().Cookies(), stateCookieName)
		require.NotNil(t, cookie)
		assert.True(t, cookie.Secure)
		assert.True(t, cookie.HttpOnly)
		assert.Equal(t, http.SameSiteLaxMode, cookie.SameSite)

		gotReturnTo, gotClientNonce, err := oauthproxy.VerifyState(pub, loc.Query().Get("state"), cookie.Value)
		require.NoError(t, err)
		assert.Equal(t, returnTo, gotReturnTo)
		assert.Equal(t, "client-nonce-1", gotClientNonce)
	})

	t.Run("loopback http sets non-secure state cookie", func(t *testing.T) {
		h := newStartHandler(newCfg("http://localhost:8080"), priv)
		returnTo := "http://localhost:8000/oauth/complete"
		req := httptest.NewRequest(http.MethodGet, "/oauth/start?return_to="+url.QueryEscape(returnTo)+"&nonce=local-nonce", nil)
		rec := httptest.NewRecorder()
		h.ServeHTTP(rec, req)

		require.Equal(t, http.StatusFound, rec.Code)
		cookie := findRespCookie(rec.Result().Cookies(), stateCookieName)
		require.NotNil(t, cookie)
		assert.False(t, cookie.Secure)
		assert.True(t, cookie.HttpOnly)
		assert.Equal(t, http.SameSiteLaxMode, cookie.SameSite)
	})

	t.Run("rejects bad parameters", func(t *testing.T) {
		h := newStartHandler(newCfg("https://proxy.run.app"), priv)
		badURLs := []string{
			"/oauth/start",
			"/oauth/start?return_to=https://evil.com/oauth/complete&nonce=n1",
			"/oauth/start?return_to=https://tenant.run.app/oauth/complete",
			"/oauth/start?return_to=https://tenant.run.app/oauth/complete&nonce=" + strings.Repeat("a", maxClientNonceLen+1),
		}
		for _, u := range badURLs {
			rec := httptest.NewRecorder()
			h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, u, nil))
			assert.Equal(t, http.StatusBadRequest, rec.Code, "url=%s", u)
		}
	})
}

func TestCallbackHandlerEndToEnd(t *testing.T) {
	pub, priv := mustTestKeyPair(t)

	tokenSrv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(map[string]any{
			"access_token": "upstream-token",
			"token_type":   "bearer",
			"expires_in":   3600,
		})
	}))
	defer tokenSrv.Close()

	profileStatus := http.StatusOK
	profileBody := `{"sub":"sub-42","picture":"https://example.com/p.png","email":"op@example.com","name":"Operator","email_verified":true}`
	profileSrv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(profileStatus)
		_, _ = w.Write([]byte(profileBody))
	}))
	defer profileSrv.Close()

	cfg := &Config{
		OAuth: oauth2.Config{
			ClientID:     "cid",
			ClientSecret: "csec",
			RedirectURL:  "https://proxy.run.app/oauth/callback",
			Endpoint: oauth2.Endpoint{
				AuthURL:   "https://accounts.example.com/auth",
				TokenURL:  tokenSrv.URL,
				AuthStyle: oauth2.AuthStyleInParams,
			},
		},
		ProfileURL:      profileSrv.URL,
		AllowedSuffixes: []string{".run.app"},
		ProxyDomain:     "https://proxy.run.app",
	}

	returnTo := "https://tenant.run.app/oauth/complete"
	clientNonce := "tenant-nonce-xyz"
	stateNonce := "proxy-state-nonce"
	state, err := oauthproxy.SignState(priv, returnTo, clientNonce, stateNonce, time.Minute)
	require.NoError(t, err)

	h := newCallbackHandler(cfg, priv)

	t.Run("happy path mints verifiable assertion and clears state cookie", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/oauth/callback?state="+url.QueryEscape(state)+"&code=auth-code", nil)
		req.AddCookie(&http.Cookie{Name: stateCookieName, Value: stateNonce})
		rec := httptest.NewRecorder()
		h.ServeHTTP(rec, req)

		require.Equal(t, http.StatusFound, rec.Code)

		cleared := findRespCookie(rec.Result().Cookies(), stateCookieName)
		require.NotNil(t, cleared)
		assert.Equal(t, "", cleared.Value)
		assert.Equal(t, -1, cleared.MaxAge)
		assert.True(t, cleared.Secure)
		assert.True(t, cleared.HttpOnly)
		assert.Equal(t, http.SameSiteLaxMode, cleared.SameSite)
		assert.True(t, cleared.Expires.Equal(time.Unix(0, 0).UTC()))

		loc, err := url.Parse(rec.Header().Get("Location"))
		require.NoError(t, err)
		assert.Equal(t, "tenant.run.app", loc.Host)
		assert.Equal(t, "/oauth/complete", loc.Path)

		assertion := loc.Query().Get("assertion")
		require.NotEmpty(t, assertion)
		claims, err := oauthproxy.VerifyAssertion(pub, assertion, returnTo)
		require.NoError(t, err)
		assert.Equal(t, "sub-42", claims.OAuthID)
		assert.Equal(t, "Operator", claims.Name)
		assert.Equal(t, "op@example.com", claims.Email)
		assert.True(t, claims.EmailVerified)
		assert.Equal(t, clientNonce, claims.Nonce)
	})

	t.Run("invalid state does not clear state cookie", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/oauth/callback?state=forged&code=auth-code", nil)
		req.AddCookie(&http.Cookie{Name: stateCookieName, Value: stateNonce})
		rec := httptest.NewRecorder()
		h.ServeHTTP(rec, req)

		assert.Equal(t, http.StatusBadRequest, rec.Code)
		assert.Nil(t, findRespCookie(rec.Result().Cookies(), stateCookieName),
			"forged callback must not clear in-flight state cookie")
	})

	t.Run("mismatched state cookie does not clear state cookie", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/oauth/callback?state="+url.QueryEscape(state)+"&code=auth-code", nil)
		req.AddCookie(&http.Cookie{Name: stateCookieName, Value: "wrong-cookie-nonce"})
		rec := httptest.NewRecorder()
		h.ServeHTTP(rec, req)

		assert.Equal(t, http.StatusBadRequest, rec.Code)
		assert.Nil(t, findRespCookie(rec.Result().Cookies(), stateCookieName))
	})

	t.Run("missing state or cookie rejected", func(t *testing.T) {
		rec := httptest.NewRecorder()
		h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/oauth/callback?code=c", nil))
		assert.Equal(t, http.StatusBadRequest, rec.Code)

		rec = httptest.NewRecorder()
		h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/oauth/callback?state="+url.QueryEscape(state)+"&code=c", nil))
		assert.Equal(t, http.StatusBadRequest, rec.Code)
	})

	t.Run("missing code rejected", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/oauth/callback?state="+url.QueryEscape(state), nil)
		req.AddCookie(&http.Cookie{Name: stateCookieName, Value: stateNonce})
		rec := httptest.NewRecorder()
		h.ServeHTTP(rec, req)
		assert.Equal(t, http.StatusBadRequest, rec.Code)
	})

	t.Run("narrowed allowlist rejects return_to on callback", func(t *testing.T) {
		narrowCfg := *cfg
		narrowCfg.AllowedSuffixes = []string{"other.example.com"}
		nh := newCallbackHandler(&narrowCfg, priv)

		req := httptest.NewRequest(http.MethodGet, "/oauth/callback?state="+url.QueryEscape(state)+"&code=auth-code", nil)
		req.AddCookie(&http.Cookie{Name: stateCookieName, Value: stateNonce})
		rec := httptest.NewRecorder()
		nh.ServeHTTP(rec, req)
		assert.Equal(t, http.StatusBadRequest, rec.Code)
	})

	t.Run("upstream profile non-200 status returns 500", func(t *testing.T) {
		profileStatus = http.StatusBadGateway
		t.Cleanup(func() { profileStatus = http.StatusOK })

		req := httptest.NewRequest(http.MethodGet, "/oauth/callback?state="+url.QueryEscape(state)+"&code=auth-code", nil)
		req.AddCookie(&http.Cookie{Name: stateCookieName, Value: stateNonce})
		rec := httptest.NewRecorder()
		h.ServeHTTP(rec, req)
		assert.Equal(t, http.StatusInternalServerError, rec.Code)
	})

	t.Run("upstream profile missing sub returns 500", func(t *testing.T) {
		orig := profileBody
		profileBody = `{"name":"NoSub"}`
		t.Cleanup(func() { profileBody = orig })

		req := httptest.NewRequest(http.MethodGet, "/oauth/callback?state="+url.QueryEscape(state)+"&code=auth-code", nil)
		req.AddCookie(&http.Cookie{Name: stateCookieName, Value: stateNonce})
		rec := httptest.NewRecorder()
		h.ServeHTTP(rec, req)
		assert.Equal(t, http.StatusInternalServerError, rec.Code)
	})
}
