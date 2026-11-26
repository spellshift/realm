package http_test

import (
	"bufio"
	"bytes"
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/sha256"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/hex"
	"encoding/json"
	"encoding/pem"
	"net"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/ent/enttest"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/internal/logging"
)

type capturePersister struct {
	mu      sync.Mutex
	entries []logging.AuditEntry
}

func (c *capturePersister) Enqueue(e logging.AuditEntry) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.entries = append(c.entries, e)
}

func (c *capturePersister) Snapshot() []logging.AuditEntry {
	c.mu.Lock()
	defer c.mu.Unlock()
	out := make([]logging.AuditEntry, len(c.entries))
	copy(out, c.entries)
	return out
}

type mockHijackResponseWriter struct {
	*httptest.ResponseRecorder
	flushed  bool
	hijacked bool
}

func (m *mockHijackResponseWriter) Flush() {
	m.flushed = true
	m.ResponseRecorder.Flush()
}

func (m *mockHijackResponseWriter) Hijack() (net.Conn, *bufio.ReadWriter, error) {
	m.hijacked = true
	c1, c2 := net.Pipe()
	_ = c2.Close()
	return c1, bufio.NewReadWriter(bufio.NewReader(c1), bufio.NewWriter(c1)), nil
}

func parseJSONLines(t *testing.T, buf *bytes.Buffer) []map[string]any {
	t.Helper()
	var out []map[string]any
	for _, line := range strings.Split(strings.TrimSpace(buf.String()), "\n") {
		if strings.TrimSpace(line) == "" {
			continue
		}
		var rec map[string]any
		require.NoError(t, json.Unmarshal([]byte(line), &rec))
		out = append(out, rec)
	}
	return out
}

func TestHTTPTraceCorrelationAndResponseRecorder(t *testing.T) {
	var buf bytes.Buffer
	require.NoError(t, logging.Configure(logging.Options{
		Spec:   "http.access=debug",
		Format: logging.FormatJSON,
		Output: &buf,
	}))
	t.Cleanup(func() {
		_ = logging.Configure(logging.Options{Spec: "dev"})
	})

	var (
		capturedTrace      string
		capturedReq        string
		capturedClientIP   string
		capturedEntrypoint string
		hasFlusher         bool
		hasHijacker        bool
		unwrappedMatches   bool
	)

	underlying := &mockHijackResponseWriter{ResponseRecorder: httptest.NewRecorder()}

	srv := tavernhttp.NewServer(tavernhttp.RouteMap{
		"/ws": http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			capturedTrace = logging.TraceIDFromContext(r.Context())
			capturedReq = logging.RequestIDFromContext(r.Context())
			capturedClientIP = logging.ClientIPFromContext(r.Context())
			capturedEntrypoint = logging.EntrypointFromContext(r.Context())

			if f, ok := w.(http.Flusher); ok {
				hasFlusher = true
				f.Flush()
			}
			if _, ok := w.(http.Hijacker); ok {
				hasHijacker = true
			}
			if u, ok := w.(interface{ Unwrap() http.ResponseWriter }); ok {
				unwrappedMatches = (u.Unwrap() == underlying)
			}
			w.WriteHeader(http.StatusCreated)
			_, _ = w.Write([]byte("hello-world"))
		}),
	})

	incomingTrace := "4bf92f3577b34da6a3ce929d0e0e4736"
	req := httptest.NewRequest(http.MethodGet, "/ws", nil)
	req.Header.Set("traceparent", "00-"+incomingTrace+"-00f067aa0ba902b7-01")
	req.Header.Set("X-Redirected-For", "198.51.100.42:54321")
	req.Header.Set("Upgrade", "websocket")
	req.Header.Set("User-Agent", "realm-test-agent/1.0")

	srv.ServeHTTP(underlying, req)

	assert.Equal(t, incomingTrace, underlying.Header().Get(logging.HeaderTraceID))
	assert.NotEmpty(t, underlying.Header().Get(logging.HeaderRequestID))
	assert.Equal(t, incomingTrace, capturedTrace)
	assert.Equal(t, underlying.Header().Get(logging.HeaderRequestID), capturedReq)
	assert.Equal(t, "198.51.100.42", capturedClientIP)
	assert.Equal(t, "websocket", capturedEntrypoint)
	assert.True(t, hasFlusher)
	assert.True(t, underlying.flushed)
	assert.True(t, hasHijacker)
	assert.True(t, unwrappedMatches)

	records := parseJSONLines(t, &buf)
	require.Len(t, records, 1)
	assert.Equal(t, "http.access", records[0]["subsystem"])
	assert.Equal(t, "INFO", records[0]["severity"])
	assert.Equal(t, float64(http.StatusCreated), records[0]["status_code"])
	assert.Equal(t, float64(len("hello-world")), records[0]["bytes_written"])
	assert.Equal(t, "198.51.100.42", records[0]["client_ip"])
	assert.Equal(t, "realm-test-agent/1.0", records[0]["user_agent"])
}

func TestHTTPAccessRoutingAndGRPCSkip(t *testing.T) {
	var buf bytes.Buffer
	require.NoError(t, logging.Configure(logging.Options{
		Spec:   "http.access=debug",
		Format: logging.FormatJSON,
		Output: &buf,
	}))
	t.Cleanup(func() {
		_ = logging.Configure(logging.Options{Spec: "dev"})
	})

	srv := tavernhttp.NewServer(tavernhttp.RouteMap{
		"/status":          http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) { w.WriteHeader(http.StatusOK) }),
		"/static/app.js":   http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) { w.WriteHeader(http.StatusOK) }),
		"/api/ok":          http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) { w.WriteHeader(http.StatusOK) }),
		"/api/forbidden":   http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) { w.WriteHeader(http.StatusForbidden) }),
		"/api/servererror": http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) { w.WriteHeader(http.StatusInternalServerError) }),
		"/c2.C2/ClaimTasks": http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
			w.WriteHeader(http.StatusOK)
		}),
	})

	paths := []string{"/status", "/static/app.js", "/api/ok", "/api/forbidden", "/api/servererror", "/c2.C2/ClaimTasks"}
	for _, p := range paths {
		w := httptest.NewRecorder()
		srv.ServeHTTP(w, httptest.NewRequest(http.MethodGet, p, nil))
		assert.NotEmpty(t, w.Header().Get(logging.HeaderTraceID))
		assert.NotEmpty(t, w.Header().Get(logging.HeaderRequestID))
	}

	// Also verify Content-Type: application/grpc is skipped regardless of path
	grpcReq := httptest.NewRequest(http.MethodPost, "/api/ok", nil)
	grpcReq.Header.Set("Content-Type", "application/grpc+proto")
	srv.ServeHTTP(httptest.NewRecorder(), grpcReq)

	records := parseJSONLines(t, &buf)
	require.Len(t, records, 5)

	assert.Equal(t, "/status", records[0]["path"])
	assert.Equal(t, "DEBUG", records[0]["severity"])

	assert.Equal(t, "/static/app.js", records[1]["path"])
	assert.Equal(t, "DEBUG", records[1]["severity"])

	assert.Equal(t, "/api/ok", records[2]["path"])
	assert.Equal(t, "INFO", records[2]["severity"])

	assert.Equal(t, "/api/forbidden", records[3]["path"])
	assert.Equal(t, "WARNING", records[3]["severity"])

	assert.Equal(t, "/api/servererror", records[4]["path"])
	assert.Equal(t, "ERROR", records[4]["severity"])
}

func TestAuthBypassAndDenialAudits(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	persister := &capturePersister{}
	logging.SetAuditPersister(persister)
	t.Cleanup(func() { logging.SetAuditPersister(nil) })

	// 1. Verify bypass authenticator tags AuthMethod="bypass"
	var bypassActor logging.Actor
	bypassSrv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/check": http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				bypassActor = logging.ActorFromContext(r.Context())
				w.WriteHeader(http.StatusOK)
			}),
		},
		tavernhttp.WithAuthenticationBypass(graph),
	)
	bypassSrv.ServeHTTP(httptest.NewRecorder(), httptest.NewRequest(http.MethodGet, "/check", nil))
	assert.Equal(t, logging.ActorUser, bypassActor.Kind)
	assert.Equal(t, "bypass", bypassActor.AuthMethod)

	// 2. Verify invalid access token, invalid cookie, unauthenticated, and unactivated audits
	unactivatedUser := graph.User.Create().
		SetName("pending_operator").
		SetOauthID("pending_oauth").
		SetPhotoURL("https://example.com/p.png").
		SetIsActivated(false).
		SetIsAdmin(false).
		SaveX(ctx)

	authSrv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/protected": tavernhttp.Endpoint{
				Handler: http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
					w.WriteHeader(http.StatusOK)
				}),
			},
		},
		tavernhttp.WithAuthentication(graph),
	)

	// Invalid access token
	reqBadToken := httptest.NewRequest(http.MethodGet, "/protected", nil)
	reqBadToken.Header.Set(auth.HeaderAPIAccessToken, "bad-secret-token")
	wBadToken := httptest.NewRecorder()
	authSrv.ServeHTTP(wBadToken, reqBadToken)
	assert.Equal(t, http.StatusUnauthorized, wBadToken.Code)

	// Invalid session cookie
	reqBadCookie := httptest.NewRequest(http.MethodGet, "/protected", nil)
	reqBadCookie.AddCookie(&http.Cookie{Name: auth.SessionCookieName, Value: "bad-session-cookie"})
	wBadCookie := httptest.NewRecorder()
	authSrv.ServeHTTP(wBadCookie, reqBadCookie)
	assert.Equal(t, http.StatusUnauthorized, wBadCookie.Code)

	// Unauthenticated request to protected endpoint
	wUnauth := httptest.NewRecorder()
	authSrv.ServeHTTP(wUnauth, httptest.NewRequest(http.MethodGet, "/protected", nil))
	assert.Equal(t, http.StatusUnauthorized, wUnauth.Code)

	// Unactivated user request to protected endpoint
	reqUnactivated := httptest.NewRequest(http.MethodGet, "/protected", nil)
	reqUnactivated.Header.Set(auth.HeaderAPIAccessToken, unactivatedUser.AccessToken)
	wUnactivated := httptest.NewRecorder()
	authSrv.ServeHTTP(wUnactivated, reqUnactivated)
	assert.Equal(t, http.StatusForbidden, wUnactivated.Code)

	entries := persister.Snapshot()
	require.Len(t, entries, 4)
	assert.Equal(t, "auth.access_token_denied", entries[0].Action)
	assert.Equal(t, logging.OutcomeDenied, entries[0].Outcome)
	assert.Equal(t, "auth.cookie_denied", entries[1].Action)
	assert.Equal(t, logging.OutcomeDenied, entries[1].Outcome)
	assert.Equal(t, "auth.unauthenticated_denied", entries[2].Action)
	assert.Equal(t, logging.OutcomeDenied, entries[2].Outcome)
	assert.Equal(t, "auth.unactivated_denied", entries[3].Action)
	assert.Equal(t, logging.OutcomeDenied, entries[3].Outcome)
	assert.Equal(t, "pending_operator", entries[3].Actor.Name)
	assert.Equal(t, "access_token", entries[3].Actor.AuthMethod)
}

func TestRDAAuditLifecycleAndSignout(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	var logBuf bytes.Buffer
	require.NoError(t, logging.Configure(logging.Options{
		Spec:   "auth=debug,audit=info",
		Format: logging.FormatJSON,
		Output: &logBuf,
	}))
	persister := &capturePersister{}
	logging.SetAuditPersister(persister)
	t.Cleanup(func() {
		logging.SetAuditPersister(nil)
		_ = logging.Configure(logging.Options{Spec: "dev"})
	})

	operator := graph.User.Create().
		SetName("rda_operator").
		SetOauthID("rda_oauth").
		SetPhotoURL("https://example.com/p.png").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/auth/rda/code": tavernhttp.Endpoint{
				Handler:              tavernhttp.NewRDACodeHandler(graph),
				AllowUnauthenticated: true,
				AllowUnactivated:     true,
			},
			"/auth/rda/token": tavernhttp.Endpoint{
				Handler:              tavernhttp.NewRDATokenHandler(graph),
				AllowUnauthenticated: true,
				AllowUnactivated:     true,
			},
			"/auth/rda/approve": tavernhttp.Endpoint{
				Handler: tavernhttp.NewRDAApproveHandler(graph),
			},
			"/auth/rda/revoke": tavernhttp.Endpoint{
				Handler: tavernhttp.NewRDARevokeHandler(graph),
			},
			"/api/auth/signout": tavernhttp.Endpoint{
				Handler:              tavernhttp.NewSignoutHandler(),
				AllowUnauthenticated: true,
				AllowUnactivated:     true,
			},
		},
		tavernhttp.WithAuthentication(graph),
	)

	// 1. Create code
	wCode := httptest.NewRecorder()
	srv.ServeHTTP(wCode, httptest.NewRequest(http.MethodPost, "/auth/rda/code", nil))
	require.Equal(t, http.StatusOK, wCode.Code)
	var codeResp tavernhttp.RDACodeResponse
	require.NoError(t, json.Unmarshal(wCode.Body.Bytes(), &codeResp))

	// 2. Poll while PENDING -> should NOT emit an audit log
	wPollPending := httptest.NewRecorder()
	srv.ServeHTTP(wPollPending, httptest.NewRequest(http.MethodGet, "/auth/rda/token?device_code="+codeResp.DeviceCode, nil))
	require.Equal(t, http.StatusOK, wPollPending.Code)
	require.Len(t, persister.Snapshot(), 1) // still only rda.code_created

	// 3. Approve device
	approveReq := httptest.NewRequest(http.MethodPost, "/auth/rda/approve", strings.NewReader(`{"user_code":"`+codeResp.UserCode+`"}`))
	approveReq.Header.Set("Content-Type", "application/json")
	approveReq.Header.Set(auth.HeaderAPIAccessToken, operator.AccessToken)
	wApprove := httptest.NewRecorder()
	srv.ServeHTTP(wApprove, approveReq)
	require.Equal(t, http.StatusOK, wApprove.Code)

	// 4. Poll APPROVED -> issues token & emits rda.token_issued
	wPollApproved := httptest.NewRecorder()
	srv.ServeHTTP(wPollApproved, httptest.NewRequest(http.MethodGet, "/auth/rda/token?device_code="+codeResp.DeviceCode, nil))
	require.Equal(t, http.StatusOK, wPollApproved.Code)

	// 5. Revoke device
	revokeReq := httptest.NewRequest(http.MethodPost, "/auth/rda/revoke", strings.NewReader(`{"user_code":"`+codeResp.UserCode+`"}`))
	revokeReq.Header.Set("Content-Type", "application/json")
	revokeReq.Header.Set(auth.HeaderAPIAccessToken, operator.AccessToken)
	wRevoke := httptest.NewRecorder()
	srv.ServeHTTP(wRevoke, revokeReq)
	require.Equal(t, http.StatusOK, wRevoke.Code)

	// 6. Signout
	signoutReq := httptest.NewRequest(http.MethodPost, "/api/auth/signout", nil)
	signoutReq.AddCookie(&http.Cookie{Name: auth.SessionCookieName, Value: operator.SessionToken})
	wSignout := httptest.NewRecorder()
	srv.ServeHTTP(wSignout, signoutReq)
	require.Equal(t, http.StatusOK, wSignout.Code)

	entries := persister.Snapshot()
	require.Len(t, entries, 5)
	assert.Equal(t, "rda.code_created", entries[0].Action)
	assert.Equal(t, "rda.approved", entries[1].Action)
	assert.Equal(t, "rda.token_issued", entries[2].Action)
	assert.Equal(t, "rda", entries[2].Actor.AuthMethod)
	assert.Equal(t, "rda.revoked", entries[3].Action)
	assert.Equal(t, "auth.signout", entries[4].Action)

	// Ensure neither raw device_code nor raw access_token leaked into logs
	logOutput := logBuf.String()
	assert.NotContains(t, logOutput, codeResp.DeviceCode)
	assert.NotContains(t, logOutput, operator.AccessToken)
}

func TestBuilderEnrollAuditing(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	var logBuf bytes.Buffer
	require.NoError(t, logging.Configure(logging.Options{
		Spec:   "builder=debug,audit=info",
		Format: logging.FormatJSON,
		Output: &logBuf,
	}))
	persister := &capturePersister{}
	logging.SetAuditPersister(persister)
	t.Cleanup(func() {
		logging.SetAuditPersister(nil)
		_ = logging.Configure(logging.Options{Spec: "dev"})
	})

	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	caCert, err := builder.CreateCA(caKey)
	require.NoError(t, err)

	rawToken := "top-secret-builder-enrollment-token"
	tokenHash := sha256.Sum256([]byte(rawToken))
	graph.BuilderEnrollmentToken.Create().
		SetTokenHash(hex.EncodeToString(tokenHash[:])).
		SaveX(ctx)

	_, builderKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	csrDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{
		Subject: pkix.Name{CommonName: "builder-test"},
	}, builderKey)
	require.NoError(t, err)
	csrPEM := string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrDER}))

	handler := tavernhttp.NewBuilderEnrollHandler(graph, caCert, caKey)

	// 1. Denied on invalid token
	badPayload, _ := json.Marshal(tavernhttp.BuilderEnrollRequest{
		Token:            "wrong-token",
		CSR:              csrPEM,
		SupportedTargets: []string{"linux"},
		Upstream:         "https://tavern.example.com",
		Executor:         "docker",
	})
	wDenied := httptest.NewRecorder()
	handler.ServeHTTP(wDenied, httptest.NewRequest(http.MethodPost, "/auth/builder/enroll", bytes.NewReader(badPayload)))
	require.Equal(t, http.StatusUnauthorized, wDenied.Code)

	// 2. Enrolled on valid token
	goodPayload, _ := json.Marshal(tavernhttp.BuilderEnrollRequest{
		Token:            rawToken,
		CSR:              csrPEM,
		SupportedTargets: []string{"linux", "windows"},
		Upstream:         "https://tavern.example.com",
		Executor:         "docker",
	})
	wOK := httptest.NewRecorder()
	handler.ServeHTTP(wOK, httptest.NewRequest(http.MethodPost, "/auth/builder/enroll", bytes.NewReader(goodPayload)))
	require.Equal(t, http.StatusOK, wOK.Code)

	var enrollResp tavernhttp.BuilderEnrollResponse
	require.NoError(t, json.Unmarshal(wOK.Body.Bytes(), &enrollResp))
	require.NotEmpty(t, enrollResp.BuilderID)

	entries := persister.Snapshot()
	require.Len(t, entries, 2)
	assert.Equal(t, "builder.enroll_denied", entries[0].Action)
	assert.Equal(t, logging.OutcomeDenied, entries[0].Outcome)

	assert.Equal(t, "builder.enrolled", entries[1].Action)
	assert.Equal(t, logging.OutcomeSuccess, entries[1].Outcome)
	assert.Equal(t, "builder", entries[1].TargetType)
	assert.Equal(t, enrollResp.BuilderID, entries[1].TargetName)
	assert.Equal(t, "docker", entries[1].Details["executor"])

	// Raw enrollment token and CSR must never appear in logs
	assert.NotContains(t, logBuf.String(), rawToken)
	assert.NotContains(t, logBuf.String(), "BEGIN CERTIFICATE REQUEST")
}
