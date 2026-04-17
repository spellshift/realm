package http_test

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	tavernhttp "realm.pub/tavern/internal/http"
)

func TestOAuthProtectedResourceMetadataHandler(t *testing.T) {
	h := tavernhttp.NewOAuthProtectedResourceMetadataHandler()

	req := httptest.NewRequest(http.MethodGet, "http://example.test/.well-known/oauth-protected-resource", nil)
	w := httptest.NewRecorder()

	h.ServeHTTP(w, req)

	resp := w.Result()
	defer resp.Body.Close()
	assert.Equal(t, http.StatusOK, resp.StatusCode)
	assert.Equal(t, "application/json", resp.Header.Get("Content-Type"))

	var data map[string]any
	err := json.NewDecoder(resp.Body).Decode(&data)
	assert.NoError(t, err)
	assert.Equal(t, "http://example.test/mcp", data["resource"])
	assert.NotNil(t, data["authorization_servers"])
}

func TestOAuthAuthorizationServerMetadataHandler(t *testing.T) {
	h := tavernhttp.NewOAuthAuthorizationServerMetadataHandler()

	req := httptest.NewRequest(http.MethodGet, "http://example.test/.well-known/oauth-authorization-server", nil)
	w := httptest.NewRecorder()

	h.ServeHTTP(w, req)

	resp := w.Result()
	defer resp.Body.Close()
	assert.Equal(t, http.StatusOK, resp.StatusCode)
	assert.Equal(t, "application/json", resp.Header.Get("Content-Type"))

	var data map[string]any
	err := json.NewDecoder(resp.Body).Decode(&data)
	assert.NoError(t, err)
	assert.Equal(t, "http://example.test", data["issuer"])
	assert.Equal(t, "http://example.test/oauth/login", data["authorization_endpoint"])
}

func TestOpenIDConfigurationHandler(t *testing.T) {
	h := tavernhttp.NewOpenIDConfigurationHandler()

	req := httptest.NewRequest(http.MethodGet, "http://example.test/.well-known/openid-configuration", nil)
	w := httptest.NewRecorder()

	h.ServeHTTP(w, req)

	resp := w.Result()
	defer resp.Body.Close()
	assert.Equal(t, http.StatusOK, resp.StatusCode)
	assert.Equal(t, "application/json", resp.Header.Get("Content-Type"))

	var data map[string]any
	err := json.NewDecoder(resp.Body).Decode(&data)
	assert.NoError(t, err)
	assert.Equal(t, "http://example.test", data["issuer"])
	assert.Equal(t, "http://example.test/oauth/login", data["authorization_endpoint"])
}
