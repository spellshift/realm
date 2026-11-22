package http_test

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent/deviceauth"
	"realm.pub/tavern/internal/ent/enttest"
	tavernhttp "realm.pub/tavern/internal/http"
)

func TestRDACodeHandler_VerificationURIHardening(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	t.Run("configured public base URL ignores spoofed host and forwarded proto", func(t *testing.T) {
		handler := tavernhttp.NewRDACodeHandler(graph, "https://tavern.example.com")
		req := httptest.NewRequest(http.MethodPost, "/auth/rda/code", nil)
		req.Host = "evil.attacker.tld"
		req.Header.Set("X-Forwarded-Proto", "http")

		w := httptest.NewRecorder()
		handler.ServeHTTP(w, req)
		require.Equal(t, http.StatusOK, w.Code)

		var resp tavernhttp.RDACodeResponse
		require.NoError(t, json.Unmarshal(w.Body.Bytes(), &resp))
		assert.Equal(t, "https://tavern.example.com/profile", resp.VerificationURI)
		assert.Equal(t, "https://tavern.example.com/profile?device-code="+resp.UserCode, resp.VerificationURIComplete)
		assert.Equal(t, 5, resp.Interval)
	})

	t.Run("rejects malformed host header when unconfigured", func(t *testing.T) {
		handler := tavernhttp.NewRDACodeHandler(graph)
		for _, badHost := range []string{"", "evil.com/phish", "user@evil.com", "evil.com?x=1", "evil.com:99999"} {
			req := httptest.NewRequest(http.MethodPost, "/auth/rda/code", nil)
			req.Host = badHost
			w := httptest.NewRecorder()
			handler.ServeHTTP(w, req)
			assert.Equal(t, http.StatusBadRequest, w.Code, "expected 400 for host %q", badHost)
		}
	})

	t.Run("uses validated host and https forwarded proto when unconfigured", func(t *testing.T) {
		handler := tavernhttp.NewRDACodeHandler(graph)
		req := httptest.NewRequest(http.MethodPost, "/auth/rda/code", nil)
		req.Host = "127.0.0.1:8000"
		req.Header.Set("X-Forwarded-Proto", "https, http")

		w := httptest.NewRecorder()
		handler.ServeHTTP(w, req)
		require.Equal(t, http.StatusOK, w.Code)

		var resp tavernhttp.RDACodeResponse
		require.NoError(t, json.Unmarshal(w.Body.Bytes(), &resp))
		assert.Equal(t, "https://127.0.0.1:8000/profile", resp.VerificationURI)
	})
}

func TestRDATokenHandler_PollingAndSingleUseConsumption(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	operator := graph.User.Create().
		SetName("rda_test_op").
		SetOauthID("rda_test_oauth").
		SetPhotoURL("https://example.com/p.png").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	tokenHandler := tavernhttp.NewRDATokenHandler(graph)

	// Create approved device auth code
	approved := graph.DeviceAuth.Create().
		SetUserCode("aaaa1111").
		SetDeviceCode("device-code-approved-1").
		SetStatus(deviceauth.StatusAPPROVED).
		SetExpiresAt(time.Now().Add(5 * time.Minute)).
		SetUser(operator).
		SaveX(ctx)

	// 1. POST JSON body exchanges approved code once
	w1 := httptest.NewRecorder()
	req1 := httptest.NewRequest(http.MethodPost, "/auth/rda/token", strings.NewReader(`{"device_code":"`+approved.DeviceCode+`"}`))
	req1.Header.Set("Content-Type", "application/json")
	tokenHandler.ServeHTTP(w1, req1)
	require.Equal(t, http.StatusOK, w1.Code)

	var resp1 tavernhttp.RDATokenResponse
	require.NoError(t, json.Unmarshal(w1.Body.Bytes(), &resp1))
	assert.Equal(t, "APPROVED", resp1.Status)
	assert.Equal(t, operator.AccessToken, resp1.AccessToken)

	// 2. Second poll with the same approved code must be rejected (consumed)
	w2 := httptest.NewRecorder()
	req2 := httptest.NewRequest(http.MethodPost, "/auth/rda/token", strings.NewReader(`{"device_code":"`+approved.DeviceCode+`"}`))
	req2.Header.Set("Content-Type", "application/json")
	tokenHandler.ServeHTTP(w2, req2)
	assert.Equal(t, http.StatusGone, w2.Code)

	var resp2 tavernhttp.RDATokenResponse
	require.NoError(t, json.Unmarshal(w2.Body.Bytes(), &resp2))
	assert.Empty(t, resp2.AccessToken)
	assert.Equal(t, "expired_token", resp2.Error)

	// 3. Denied device returns 403 Forbidden with access_denied
	denied := graph.DeviceAuth.Create().
		SetUserCode("bbbb2222").
		SetDeviceCode("device-code-denied-1").
		SetStatus(deviceauth.StatusDENIED).
		SetExpiresAt(time.Now().Add(5 * time.Minute)).
		SaveX(ctx)

	wDenied := httptest.NewRecorder()
	reqDenied := httptest.NewRequest(http.MethodPost, "/auth/rda/token", strings.NewReader(`{"device_code":"`+denied.DeviceCode+`"}`))
	reqDenied.Header.Set("Content-Type", "application/json")
	tokenHandler.ServeHTTP(wDenied, reqDenied)
	require.Equal(t, http.StatusForbidden, wDenied.Code)

	var respDenied tavernhttp.RDATokenResponse
	require.NoError(t, json.Unmarshal(wDenied.Body.Bytes(), &respDenied))
	assert.Equal(t, "DENIED", respDenied.Status)
	assert.Equal(t, "access_denied", respDenied.Error)

	// 4. Expired device returns 410 Gone with expired_token
	expired := graph.DeviceAuth.Create().
		SetUserCode("cccc3333").
		SetDeviceCode("device-code-expired-1").
		SetStatus(deviceauth.StatusPENDING).
		SetExpiresAt(time.Now().Add(-1 * time.Minute)).
		SaveX(ctx)

	wExp := httptest.NewRecorder()
	reqExp := httptest.NewRequest(http.MethodPost, "/auth/rda/token", strings.NewReader(`{"device_code":"`+expired.DeviceCode+`"}`))
	reqExp.Header.Set("Content-Type", "application/json")
	tokenHandler.ServeHTTP(wExp, reqExp)
	require.Equal(t, http.StatusGone, wExp.Code)

	var respExp tavernhttp.RDATokenResponse
	require.NoError(t, json.Unmarshal(wExp.Body.Bytes(), &respExp))
	assert.Equal(t, "expired_token", respExp.Error)

	// 5. Backwards-compatible GET with X-Device-Code header works for PENDING
	pending := graph.DeviceAuth.Create().
		SetUserCode("dddd4444").
		SetDeviceCode("device-code-pending-1").
		SetStatus(deviceauth.StatusPENDING).
		SetExpiresAt(time.Now().Add(5 * time.Minute)).
		SaveX(ctx)

	wPending := httptest.NewRecorder()
	reqPending := httptest.NewRequest(http.MethodGet, "/auth/rda/token", nil)
	reqPending.Header.Set("X-Device-Code", pending.DeviceCode)
	tokenHandler.ServeHTTP(wPending, reqPending)
	require.Equal(t, http.StatusOK, wPending.Code)

	var respPending tavernhttp.RDATokenResponse
	require.NoError(t, json.Unmarshal(wPending.Body.Bytes(), &respPending))
	assert.Equal(t, "PENDING", respPending.Status)
	assert.Empty(t, respPending.AccessToken)
}

func TestRDAApproveAndRevoke_CSRFProtection(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	operator := graph.User.Create().
		SetName("csrf_op").
		SetOauthID("csrf_oauth").
		SetPhotoURL("https://example.com/p.png").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/auth/rda/approve": tavernhttp.Endpoint{
				Handler: tavernhttp.NewRDAApproveHandler(graph),
			},
			"/auth/rda/revoke": tavernhttp.Endpoint{
				Handler: tavernhttp.NewRDARevokeHandler(graph),
			},
		},
		tavernhttp.WithAuthentication(graph),
	)

	da := graph.DeviceAuth.Create().
		SetUserCode("eeee5555").
		SetDeviceCode("device-code-csrf-1").
		SetStatus(deviceauth.StatusPENDING).
		SetExpiresAt(time.Now().Add(5 * time.Minute)).
		SaveX(ctx)

	for _, path := range []string{"/auth/rda/approve", "/auth/rda/revoke"} {
		t.Run(path+" rejects non-json content-type", func(t *testing.T) {
			req := httptest.NewRequest(http.MethodPost, path, strings.NewReader(`{"user_code":"`+da.UserCode+`"}`))
			req.Header.Set("Content-Type", "text/plain")
			req.AddCookie(&http.Cookie{Name: auth.SessionCookieName, Value: operator.SessionToken})
			w := httptest.NewRecorder()
			srv.ServeHTTP(w, req)
			assert.Equal(t, http.StatusUnsupportedMediaType, w.Code)
		})

		t.Run(path+" rejects cross-site Sec-Fetch-Site", func(t *testing.T) {
			req := httptest.NewRequest(http.MethodPost, path, strings.NewReader(`{"user_code":"`+da.UserCode+`"}`))
			req.Header.Set("Content-Type", "application/json")
			req.Header.Set("Sec-Fetch-Site", "cross-site")
			req.AddCookie(&http.Cookie{Name: auth.SessionCookieName, Value: operator.SessionToken})
			w := httptest.NewRecorder()
			srv.ServeHTTP(w, req)
			assert.Equal(t, http.StatusForbidden, w.Code)
		})

		t.Run(path+" rejects mismatched Origin", func(t *testing.T) {
			req := httptest.NewRequest(http.MethodPost, path, strings.NewReader(`{"user_code":"`+da.UserCode+`"}`))
			req.Host = "tavern.example.com"
			req.Header.Set("Content-Type", "application/json")
			req.Header.Set("Origin", "https://evil.example.net")
			req.AddCookie(&http.Cookie{Name: auth.SessionCookieName, Value: operator.SessionToken})
			w := httptest.NewRecorder()
			srv.ServeHTTP(w, req)
			assert.Equal(t, http.StatusForbidden, w.Code)
		})
	}

	// Same-origin JSON POST succeeds
	approveReq := httptest.NewRequest(http.MethodPost, "/auth/rda/approve", strings.NewReader(`{"user_code":"`+da.UserCode+`"}`))
	approveReq.Host = "tavern.example.com"
	approveReq.Header.Set("Content-Type", "application/json")
	approveReq.Header.Set("Sec-Fetch-Site", "same-origin")
	approveReq.Header.Set("Origin", "https://tavern.example.com")
	approveReq.AddCookie(&http.Cookie{Name: auth.SessionCookieName, Value: operator.SessionToken})
	wApprove := httptest.NewRecorder()
	srv.ServeHTTP(wApprove, approveReq)
	require.Equal(t, http.StatusOK, wApprove.Code)

	revokeReq := httptest.NewRequest(http.MethodPost, "/auth/rda/revoke", strings.NewReader(`{"user_code":"`+da.UserCode+`"}`))
	revokeReq.Host = "tavern.example.com"
	revokeReq.Header.Set("Content-Type", "application/json")
	revokeReq.Header.Set("Sec-Fetch-Site", "same-origin")
	revokeReq.Header.Set("Origin", "https://tavern.example.com")
	revokeReq.AddCookie(&http.Cookie{Name: auth.SessionCookieName, Value: operator.SessionToken})
	wRevoke := httptest.NewRecorder()
	srv.ServeHTTP(wRevoke, revokeReq)
	require.Equal(t, http.StatusOK, wRevoke.Code)
}
