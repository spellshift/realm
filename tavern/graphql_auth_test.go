package main

import (
	"bytes"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"encoding/json"
	"encoding/pem"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/tomes"
)

func newGraphQLAuthTestServer(t *testing.T) *httptest.Server {
	t.Helper()
	graph := enttest.OpenTempDB(t)
	t.Cleanup(func() { graph.Close() })
	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	caCert, err := builder.CreateCA(caKey)
	require.NoError(t, err)
	git := tomes.NewGitImporter(graph)
	srv := tavernhttp.NewServer(tavernhttp.RouteMap{
		"/graphql":             tavernhttp.Endpoint{Handler: newGraphQLHandler(graph, git, graphql.WithBuilderCA(caCert), graphql.WithBuilderCAKey(caKey))},
		"/auth/builder/enroll": tavernhttp.Endpoint{Handler: tavernhttp.NewBuilderEnrollHandler(graph, caCert, caKey), AllowUnauthenticated: true, AllowUnactivated: true},
	}, tavernhttp.WithAuthentication(graph))
	httpSrv := httptest.NewServer(srv)
	t.Cleanup(httpSrv.Close)
	graph.User.Create().SetOauthID("test_oauth_id").SetPhotoURL("https://photos.com").SetName("test").SetSessionToken("admin-session-token").SetAccessToken("access-token").SetIsActivated(true).SetIsAdmin(true).SaveX(t.Context())
	return httpSrv
}

func doGraphQLRequest(t *testing.T, srv *httptest.Server, query, sessionToken string) (int, string) {
	t.Helper()
	payload, err := json.Marshal(map[string]any{"query": query})
	require.NoError(t, err)
	req, err := http.NewRequest(http.MethodPost, srv.URL+"/graphql", bytes.NewReader(payload))
	require.NoError(t, err)
	req.Header.Set("Content-Type", "application/json")
	if sessionToken != "" {
		req.AddCookie(&http.Cookie{Name: auth.SessionCookieName, Value: sessionToken, Expires: time.Now().Add(time.Hour)})
	}
	resp, err := http.DefaultClient.Do(req)
	require.NoError(t, err)
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	require.NoError(t, err)
	return resp.StatusCode, string(body)
}

func newTestCSR(t *testing.T) string {
	t.Helper()
	_, key, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	der, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, key)
	require.NoError(t, err)
	return string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: der}))
}

func postEnrollment(t *testing.T, srv *httptest.Server, payload any) (int, string) {
	t.Helper()
	body, err := json.Marshal(payload)
	require.NoError(t, err)
	resp, err := http.Post(srv.URL+"/auth/builder/enroll", "application/json", bytes.NewReader(body))
	require.NoError(t, err)
	defer resp.Body.Close()
	data, err := io.ReadAll(resp.Body)
	require.NoError(t, err)
	return resp.StatusCode, string(data)
}

func TestBuilderEnrollmentAuthRoutes(t *testing.T) {
	srv := newGraphQLAuthTestServer(t)
	for _, query := range []string{`mutation { createBuilderEnrollmentToken }`, `query { __schema { queryType { name } } }`} {
		status, _ := doGraphQLRequest(t, srv, query, "")
		assert.Equal(t, http.StatusUnauthorized, status)
	}
	status, body := doGraphQLRequest(t, srv, `mutation { createBuilderEnrollmentToken }`, "admin-session-token")
	require.Equal(t, http.StatusOK, status)
	var created struct {
		Data struct {
			Token string `json:"createBuilderEnrollmentToken"`
		} `json:"data"`
	}
	require.NoError(t, json.Unmarshal([]byte(body), &created))
	require.NotEmpty(t, created.Data.Token)
	req := tavernhttp.BuilderEnrollRequest{Token: created.Data.Token, CSR: newTestCSR(t), SupportedTargets: []string{"linux"}, Upstream: "https://tavern.example.com", PollInterval: 5, Executor: "local"}
	status, body = postEnrollment(t, srv, req)
	require.Equal(t, http.StatusOK, status, body)
	var enrolled tavernhttp.BuilderEnrollResponse
	require.NoError(t, json.Unmarshal([]byte(body), &enrolled))
	assert.NotEmpty(t, enrolled.BuilderID)
	assert.Contains(t, enrolled.MTLSCert, "BEGIN CERTIFICATE")
	assert.NotContains(t, enrolled.MTLSCert, "PRIVATE KEY")
	status, body = postEnrollment(t, srv, req)
	assert.Equal(t, http.StatusUnauthorized, status)
	assert.NotContains(t, body, req.Token)
	req.Token = "bogus"
	invalidStatus, invalidBody := postEnrollment(t, srv, req)
	assert.Equal(t, http.StatusUnauthorized, invalidStatus)
	assert.Equal(t, body, invalidBody)
	status, body = doGraphQLRequest(t, srv, `query { __type(name: "BuilderEnrollmentToken") { name } }`, "admin-session-token")
	assert.Equal(t, http.StatusOK, status)
	assert.Contains(t, body, `"__type":null`)
}

func TestBuilderEnrollmentRejectsMalformedRequests(t *testing.T) {
	srv := newGraphQLAuthTestServer(t)
	for _, tc := range []struct {
		name, method, body string
		want               int
	}{
		{"method", http.MethodGet, "", http.StatusMethodNotAllowed},
		{"malformed JSON", http.MethodPost, `{`, http.StatusBadRequest},
		{"unknown field", http.MethodPost, `{"token":"secret","csr":"secret","unexpected":true}`, http.StatusBadRequest},
		{"oversized body", http.MethodPost, `{"padding":"` + string(bytes.Repeat([]byte("x"), 1<<20)) + `"}`, http.StatusBadRequest},
	} {
		t.Run(tc.name, func(t *testing.T) {
			req, err := http.NewRequest(tc.method, srv.URL+"/auth/builder/enroll", bytes.NewBufferString(tc.body))
			require.NoError(t, err)
			resp, err := http.DefaultClient.Do(req)
			require.NoError(t, err)
			defer resp.Body.Close()
			assert.Equal(t, tc.want, resp.StatusCode)
		})
	}
}
