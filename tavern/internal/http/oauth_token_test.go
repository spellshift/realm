package http

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestOAuthTokenAuthorizationCodeGrant(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	usr := graph.User.Create().
		SetName("token-user").
		SetOauthID("token-user").
		SetPhotoURL("https://example.test/pfp.png").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	form := url.Values{}
	form.Set("grant_type", "authorization_code")
	form.Set("code", usr.AccessToken)
	form.Set("scope", "mcp:read mcp:tools:invoke")

	req := httptest.NewRequest(http.MethodPost, "http://example.test/oauth/token", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	resp := httptest.NewRecorder()

	NewOAuthTokenHandler(graph).ServeHTTP(resp, req)

	result := resp.Result()
	defer result.Body.Close()
	require.Equal(t, http.StatusOK, result.StatusCode)

	var payload map[string]any
	require.NoError(t, json.NewDecoder(result.Body).Decode(&payload))
	assert.Equal(t, usr.AccessToken, payload["access_token"])
	assert.Equal(t, "Bearer", payload["token_type"])
	assert.Equal(t, "mcp:read mcp:tools:invoke", payload["scope"])
	assert.Equal(t, usr.AccessToken, payload["refresh_token"])
}

func TestOAuthTokenRefreshGrant(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	usr := graph.User.Create().
		SetName("refresh-user").
		SetOauthID("refresh-user").
		SetPhotoURL("https://example.test/pfp.png").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	form := url.Values{}
	form.Set("grant_type", "refresh_token")
	form.Set("refresh_token", usr.AccessToken)

	req := httptest.NewRequest(http.MethodPost, "http://example.test/oauth/token", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	resp := httptest.NewRecorder()

	NewOAuthTokenHandler(graph).ServeHTTP(resp, req)

	result := resp.Result()
	defer result.Body.Close()
	require.Equal(t, http.StatusOK, result.StatusCode)

	var payload map[string]any
	require.NoError(t, json.NewDecoder(result.Body).Decode(&payload))
	assert.Equal(t, usr.AccessToken, payload["access_token"])
	assert.Equal(t, "Bearer", payload["token_type"])
}

func TestOAuthTokenInvalidCode(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	form := url.Values{}
	form.Set("grant_type", "authorization_code")
	form.Set("code", "not-a-code")

	req := httptest.NewRequest(http.MethodPost, "http://example.test/oauth/token", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	resp := httptest.NewRecorder()

	NewOAuthTokenHandler(graph).ServeHTTP(resp, req)

	assert.Equal(t, http.StatusUnauthorized, resp.Result().StatusCode)
}
