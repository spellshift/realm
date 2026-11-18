package http1

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"realm.pub/tavern/internal/redirectors"
)

func TestIngressGate(t *testing.T) {
	gate := redirectors.NewGate()
	ctx := redirectors.WithGate(context.Background(), gate)
	forwarded := 0
	handler := ingressGate(ctx, http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		forwarded++
		if forwarded == 1 {
			gate.SetEnabled(false)
		}
		w.WriteHeader(http.StatusOK)
	}))
	request := httptest.NewRequest(http.MethodPost, "/c2.C2/ClaimTasks", nil)
	first := httptest.NewRecorder()
	handler.ServeHTTP(first, request)
	if first.Code != http.StatusOK {
		t.Fatalf("in-flight request was interrupted: %d", first.Code)
	}
	response := httptest.NewRecorder()
	handler.ServeHTTP(response, request)
	if response.Code != http.StatusServiceUnavailable || forwarded != 1 {
		t.Fatalf("disabled request: status=%d forwarded=%d", response.Code, forwarded)
	}
	gate.SetEnabled(true)
	handler.ServeHTTP(httptest.NewRecorder(), request)
	if forwarded != 2 {
		t.Fatalf("re-enabled request was not forwarded: %d", forwarded)
	}
}
