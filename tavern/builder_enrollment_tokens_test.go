package main

import (
	"bytes"
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/sha256"
	"crypto/x509"
	"encoding/hex"
	"encoding/json"
	"encoding/pem"
	"net/http"
	"net/http/httptest"
	"sync"
	"testing"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/ent/builderenrollmenttoken"
	"realm.pub/tavern/internal/ent/enttest"
	tavernhttp "realm.pub/tavern/internal/http"
)

func tokenHash(token string) string {
	hash := sha256.Sum256([]byte(token))
	return hex.EncodeToString(hash[:])
}

func TestParseBuilderEnrollmentTokens(t *testing.T) {
	tokens, err := parseBuilderEnrollmentTokens("")
	require.NoError(t, err)
	require.Empty(t, tokens)
	tokens, err = parseBuilderEnrollmentTokens(" first ,second, first ")
	require.NoError(t, err)
	require.Equal(t, []string{"first", "second", "first"}, tokens)

	for _, value := range []string{" ", "\t\n", ",first", "first,", "first,,second", "first, \t,second"} {
		tokens, err := parseBuilderEnrollmentTokens(value)
		require.ErrorContains(t, err, EnvBuilderEnrollmentTokens.Key)
		require.NotContains(t, err.Error(), "first")
		require.Nil(t, tokens)
	}
}

func TestNewServerRejectsMalformedBuilderTokensBeforeDatabaseWork(t *testing.T) {
	for _, value := range []string{" \t ", "valid, ,invalid"} {
		t.Run(value, func(t *testing.T) {
			t.Setenv(EnvBuilderEnrollmentTokens.Key, value)
			server, err := NewServer(context.Background())
			require.Nil(t, server)
			require.ErrorContains(t, err, EnvBuilderEnrollmentTokens.Key)
			require.NotContains(t, err.Error(), "valid")
		})
	}
}

func TestSeedBuilderEnrollmentTokens(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()
	ctx := context.Background()

	require.NoError(t, seedBuilderEnrollmentTokens(ctx, graph, nil))
	require.NoError(t, seedBuilderEnrollmentTokens(ctx, graph, []string{"one", "two", "one"}))
	require.NoError(t, seedBuilderEnrollmentTokens(ctx, graph, []string{"one", "two"}))
	count, err := graph.BuilderEnrollmentToken.Query().Count(ctx)
	require.NoError(t, err)
	require.Equal(t, 2, count)

	_, err = graph.BuilderEnrollmentToken.Update().Where(builderenrollmenttoken.TokenHash(tokenHash("one"))).SetUsed(true).Save(ctx)
	require.NoError(t, err)
	require.NoError(t, seedBuilderEnrollmentTokens(ctx, graph, []string{"one", "three"}))
	one, err := graph.BuilderEnrollmentToken.Query().Where(builderenrollmenttoken.TokenHash(tokenHash("one"))).Only(ctx)
	require.NoError(t, err)
	require.True(t, one.Used)
	count, err = graph.BuilderEnrollmentToken.Query().Count(ctx)
	require.NoError(t, err)
	require.Equal(t, 3, count)

	// Removing a token from the environment does not remove it from the DB.
	require.NoError(t, seedBuilderEnrollmentTokens(ctx, graph, []string{"three"}))
	count, err = graph.BuilderEnrollmentToken.Query().Count(ctx)
	require.NoError(t, err)
	require.Equal(t, 3, count)
}

func TestSeedBuilderEnrollmentTokensConcurrent(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()
	ctx := context.Background()
	var wg sync.WaitGroup
	errs := make(chan error, 12)
	for i := 0; i < cap(errs); i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			errs <- seedBuilderEnrollmentTokens(ctx, graph, []string{"shared"})
		}()
	}
	wg.Wait()
	close(errs)
	for err := range errs {
		require.NoError(t, err)
	}
	count, err := graph.BuilderEnrollmentToken.Query().Count(ctx)
	require.NoError(t, err)
	require.Equal(t, 1, count)
}

func TestPresetBuilderEnrollmentTokenExchangeAndReplay(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()
	ctx := context.Background()
	tokens, err := parseBuilderEnrollmentTokens(" preset-token ")
	require.NoError(t, err)
	require.NoError(t, seedBuilderEnrollmentTokens(ctx, graph, tokens))
	_, caKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	ca, err := builder.CreateCA(caKey)
	require.NoError(t, err)
	_, key, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	csrDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, key)
	require.NoError(t, err)
	request := tavernhttp.BuilderEnrollRequest{
		Token: "preset-token", CSR: string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrDER})), SupportedTargets: []string{"linux"},
		Upstream: "http://127.0.0.1:8000", Executor: "local",
	}
	body, err := json.Marshal(request)
	require.NoError(t, err)
	handler := tavernhttp.NewBuilderEnrollHandler(graph, ca, caKey)
	for _, wantStatus := range []int{http.StatusOK, http.StatusUnauthorized} {
		recorder := httptest.NewRecorder()
		handler.ServeHTTP(recorder, httptest.NewRequest(http.MethodPost, "/auth/builder/enroll", bytes.NewReader(body)))
		require.Equal(t, wantStatus, recorder.Code, recorder.Body.String())
	}
	require.NoError(t, seedBuilderEnrollmentTokens(ctx, graph, tokens))
	recorder := httptest.NewRecorder()
	handler.ServeHTTP(recorder, httptest.NewRequest(http.MethodPost, "/auth/builder/enroll", bytes.NewReader(body)))
	require.Equal(t, http.StatusUnauthorized, recorder.Code)
}
