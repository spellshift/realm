package hostcheck

import (
	"crypto/ed25519"
	"crypto/rand"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent/enttest"

	_ "github.com/mattn/go-sqlite3"
)

func postHostCheck(t *testing.T, endpoint, token string) *http.Response {
	t.Helper()
	req, err := http.NewRequest(http.MethodPost, endpoint, nil)
	require.NoError(t, err)
	if token != "" {
		req.Header.Set("Authorization", "Bearer "+token)
	}
	resp, err := http.DefaultClient.Do(req)
	require.NoError(t, err)
	return resp
}

func TestHandlerMissingToken(t *testing.T) {
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer client.Close()

	pubKey, _, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	handler := NewHandler(client, pubKey)
	ts := httptest.NewServer(handler)
	defer ts.Close()

	resp := postHostCheck(t, ts.URL, "")
	defer resp.Body.Close()
	assert.Equal(t, http.StatusUnauthorized, resp.StatusCode)
}

func TestHandlerRejectsQueryToken(t *testing.T) {
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer client.Close()

	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	tokenStr, err := NewToken(privKey)
	require.NoError(t, err)

	ts := httptest.NewServer(NewHandler(client, pubKey))
	defer ts.Close()

	resp, err := http.Post(ts.URL+"?token="+tokenStr, "application/json", nil)
	require.NoError(t, err)
	defer resp.Body.Close()
	assert.Equal(t, http.StatusUnauthorized, resp.StatusCode)
}

func TestHandlerInvalidToken(t *testing.T) {
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer client.Close()

	pubKey, _, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	handler := NewHandler(client, pubKey)
	ts := httptest.NewServer(handler)
	defer ts.Close()

	resp := postHostCheck(t, ts.URL, "bad-token")
	defer resp.Body.Close()
	assert.Equal(t, http.StatusUnauthorized, resp.StatusCode)
}

func TestHandlerWrongKeyToken(t *testing.T) {
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer client.Close()

	pubKey, _, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	_, otherPrivKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	// Sign with wrong key
	tokenStr, err := NewToken(otherPrivKey)
	require.NoError(t, err)

	handler := NewHandler(client, pubKey)
	ts := httptest.NewServer(handler)
	defer ts.Close()

	resp := postHostCheck(t, ts.URL, tokenStr)
	defer resp.Body.Close()
	assert.Equal(t, http.StatusUnauthorized, resp.StatusCode)
}

func TestHandlerValidToken(t *testing.T) {
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer client.Close()

	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)

	tokenStr, err := NewToken(privKey)
	require.NoError(t, err)

	handler := NewHandler(client, pubKey)
	ts := httptest.NewServer(handler)
	defer ts.Close()

	resp := postHostCheck(t, ts.URL, tokenStr)
	defer resp.Body.Close()
	assert.Equal(t, http.StatusOK, resp.StatusCode)
}
