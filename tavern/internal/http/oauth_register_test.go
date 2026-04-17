package http_test

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	tavernhttp "realm.pub/tavern/internal/http"
)

func TestOAuthDynamicClientRegistrationHandler(t *testing.T) {
	h := tavernhttp.NewOAuthDynamicClientRegistrationHandler()

	body := map[string]any{
		"client_name":   "chatgpt-mcp-test",
		"redirect_uris": []string{"https://chat.openai.com/aip/callback"},
	}
	raw, err := json.Marshal(body)
	assert.NoError(t, err)

	req := httptest.NewRequest(http.MethodPost, "http://example.test/oauth/register", bytes.NewReader(raw))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	h.ServeHTTP(w, req)

	resp := w.Result()
	defer resp.Body.Close()
	assert.Equal(t, http.StatusCreated, resp.StatusCode)
	assert.Equal(t, "application/json", resp.Header.Get("Content-Type"))

	var data map[string]any
	err = json.NewDecoder(resp.Body).Decode(&data)
	assert.NoError(t, err)
	assert.NotEmpty(t, data["client_id"])
	assert.NotEmpty(t, data["client_secret"])
	assert.Equal(t, "chatgpt-mcp-test", data["client_name"])
}

func TestOAuthDynamicClientRegistrationHandlerMethodNotAllowed(t *testing.T) {
	h := tavernhttp.NewOAuthDynamicClientRegistrationHandler()

	req := httptest.NewRequest(http.MethodGet, "http://example.test/oauth/register", nil)
	w := httptest.NewRecorder()

	h.ServeHTTP(w, req)

	resp := w.Result()
	defer resp.Body.Close()
	assert.Equal(t, http.StatusMethodNotAllowed, resp.StatusCode)
}
