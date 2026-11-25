package main

import (
	"bytes"
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"encoding/json"
	"encoding/pem"
	"net/http"
	"net/http/httptest"
	"sync"
	"testing"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/redirectorenrollmenttoken"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/internal/logging"
)

type redirectorAuditRecorder struct {
	entries []logging.AuditEntry
}

func (r *redirectorAuditRecorder) Enqueue(entry logging.AuditEntry) {
	r.entries = append(r.entries, entry)
}

func TestRedirectorEnrollmentTokenExchangeAndReplay(t *testing.T) {
	audits := &redirectorAuditRecorder{}
	logging.SetAuditPersister(audits)
	t.Cleanup(func() { logging.SetAuditPersister(nil) })
	graph := enttest.OpenTempDB(t)
	defer graph.Close()
	ctx := context.Background()
	tokens, err := parseRedirectorEnrollmentTokens(" first , second ")
	require.NoError(t, err)
	require.NoError(t, seedRedirectorEnrollmentTokens(ctx, graph, tokens))
	require.NoError(t, seedRedirectorEnrollmentTokens(ctx, graph, tokens))
	count, err := graph.RedirectorEnrollmentToken.Query().Count(ctx)
	require.NoError(t, err)
	require.Equal(t, 2, count)
	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	ca, err := builder.CreateCA(caKey)
	require.NoError(t, err)
	_, key, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	csrDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, key)
	require.NoError(t, err)
	csr := string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrDER}))
	handler := tavernhttp.NewRedirectorEnrollHandler(graph, ca, caKey)
	exchange := func(req tavernhttp.RedirectorEnrollRequest) *httptest.ResponseRecorder {
		body, marshalErr := json.Marshal(req)
		require.NoError(t, marshalErr)
		recorder := httptest.NewRecorder()
		handler.ServeHTTP(recorder, httptest.NewRequest(http.MethodPost, "/auth/redirector/enroll", bytes.NewReader(body)))
		return recorder
	}
	valid := tavernhttp.RedirectorEnrollRequest{Token: "first", CSR: csr, Transport: "http1", Domains: []string{" A.Example.com ", "b.example.com", "a.example.com", "123.example.com"}}
	invalidToken := valid
	invalidToken.Token = "unknown"
	invalidToken.CSR = "bad"
	require.Equal(t, http.StatusUnauthorized, exchange(invalidToken).Code, "token preflight must precede CSR signing")
	malformed := valid
	malformed.CSR = "bad"
	require.Equal(t, http.StatusBadRequest, exchange(malformed).Code)
	malformed = valid
	malformed.Domains = []string{"a.example.com", "bad/domain"}
	require.Equal(t, http.StatusBadRequest, exchange(malformed).Code)
	for _, domain := range []string{"127.0.0.1", "1.2.3.999"} {
		malformed = valid
		malformed.Domains = []string{domain}
		require.Equal(t, http.StatusBadRequest, exchange(malformed).Code, domain)
	}
	used, err := graph.RedirectorEnrollmentToken.Query().Where(redirectorenrollmenttoken.TokenHash(tokenHash("first"))).Only(ctx)
	require.NoError(t, err)
	require.False(t, used.Used)
	response := exchange(valid)
	require.Equal(t, http.StatusOK, response.Code, response.Body.String())
	var enrolled tavernhttp.RedirectorEnrollResponse
	require.NoError(t, json.Unmarshal(response.Body.Bytes(), &enrolled))
	require.NotEmpty(t, enrolled.RedirectorID)
	block, _ := pem.Decode([]byte(enrolled.MTLSCert))
	require.NotNil(t, block)
	cert, err := x509.ParseCertificate(block.Bytes)
	require.NoError(t, err)
	require.Equal(t, "redirector-"+enrolled.RedirectorID, cert.Subject.CommonName)
	require.NoError(t, cert.CheckSignatureFrom(ca))
	saved, err := graph.Redirector.Query().Only(ctx)
	require.NoError(t, err)
	require.Equal(t, enrolled.RedirectorID, saved.Identifier)
	require.Equal(t, []string{"a.example.com", "b.example.com", "123.example.com"}, saved.Domains)
	require.True(t, saved.Enabled)
	require.Equal(t, http.StatusUnauthorized, exchange(valid).Code)
	require.NoError(t, seedRedirectorEnrollmentTokens(ctx, graph, tokens))
	require.Equal(t, http.StatusUnauthorized, exchange(valid).Code)
	count, err = graph.Redirector.Query().Count(ctx)
	require.NoError(t, err)
	require.Equal(t, 1, count)
	var enrolledAudit, deniedAudit *logging.AuditEntry
	for i := range audits.entries {
		event := &audits.entries[i]
		switch event.Action {
		case "redirector.enrolled":
			enrolledAudit = event
		case "redirector.enroll_denied":
			if event.Details["reason"] == "invalid_or_used_token" {
				deniedAudit = event
			}
		}
	}
	require.NotNil(t, enrolledAudit)
	require.Equal(t, logging.OutcomeSuccess, enrolledAudit.Outcome)
	require.Equal(t, logging.ActorRedirector, enrolledAudit.Actor.Kind)
	require.Equal(t, enrolled.RedirectorID, enrolledAudit.Actor.Name)
	require.Equal(t, "redirector", enrolledAudit.TargetType)
	require.NotNil(t, deniedAudit)
	require.Equal(t, logging.OutcomeDenied, deniedAudit.Outcome)
}

func TestParseRedirectorEnrollmentTokens(t *testing.T) {
	tokens, err := parseRedirectorEnrollmentTokens(" one ,two, one ")
	require.NoError(t, err)
	require.Equal(t, []string{"one", "two", "one"}, tokens)
	for _, value := range []string{" ", ",one", "one,", "one,,two"} {
		_, err := parseRedirectorEnrollmentTokens(value)
		require.ErrorContains(t, err, EnvRedirectorEnrollmentTokens.Key)
	}
}

func TestRedirectorEnrollmentConcurrentRedemption(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()
	ctx := context.Background()
	require.NoError(t, seedRedirectorEnrollmentTokens(ctx, graph, []string{"shared"}))
	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	ca, err := builder.CreateCA(caKey)
	require.NoError(t, err)
	_, key, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	csrDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, key)
	require.NoError(t, err)
	body, err := json.Marshal(tavernhttp.RedirectorEnrollRequest{
		Token: "shared", CSR: string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrDER})),
		Transport: "grpc", Domains: []string{"c2.example.com"},
	})
	require.NoError(t, err)
	handler := tavernhttp.NewRedirectorEnrollHandler(graph, ca, caKey)
	const attempts = 8
	statuses := make(chan int, attempts)
	var wg sync.WaitGroup
	for range attempts {
		wg.Add(1)
		go func() {
			defer wg.Done()
			recorder := httptest.NewRecorder()
			handler.ServeHTTP(recorder, httptest.NewRequest(http.MethodPost, "/auth/redirector/enroll", bytes.NewReader(body)))
			statuses <- recorder.Code
		}()
	}
	wg.Wait()
	close(statuses)
	successes := 0
	for status := range statuses {
		if status == http.StatusOK {
			successes++
		}
	}
	require.Equal(t, 1, successes)
	count, err := graph.Redirector.Query().Count(ctx)
	require.NoError(t, err)
	require.Equal(t, 1, count)
}
