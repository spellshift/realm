package http1

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"time"

	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/redirectors"
)

const (
	streamingTimeout = 60 * time.Second
	unaryTimeout     = 10 * time.Second
	bufferCapacity   = 64 * 1024 // 64KB
	readChunkSize    = 32 * 1024 // 32KB
)

// requirePOST validates that the request method is POST
func requirePOST(w http.ResponseWriter, r *http.Request) bool {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return false
	}
	return true
}

// readRequestBody reads the entire request body and handles errors
func readRequestBody(w http.ResponseWriter, r *http.Request) ([]byte, bool) {
	defer r.Body.Close()

	requestBody, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, fmt.Sprintf("Failed to read request body: %v", err), http.StatusBadRequest)
		return nil, false
	}
	return requestBody, true
}

// setGRPCResponseHeaders sets headers for gRPC-framed HTTP/1.1 responses.
func setGRPCResponseHeaders(w http.ResponseWriter) {
	w.Header().Set("Content-Type", "application/octet-stream")
	w.Header().Set("Connection", "close")
	w.WriteHeader(http.StatusOK)
}

// getFlusher attempts to get an http.Flusher from the ResponseWriter
func getFlusher(w http.ResponseWriter) (http.Flusher, bool) {
	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "Streaming not supported", http.StatusInternalServerError)
		return nil, false
	}
	return flusher, true
}

// handleStreamError logs and returns an HTTP error for gRPC stream errors
func handleStreamError(w http.ResponseWriter, message string, err error) {
	log.Error("gRPC stream error", "message", message, "error", err)
	http.Error(w, fmt.Sprintf("%s: %v", message, err), http.StatusInternalServerError)
}

// createRequestContext creates a context with timeout
func createRequestContext(timeout time.Duration) (context.Context, context.CancelFunc) {
	return context.WithTimeout(context.Background(), timeout)
}

// prepareUpstreamContext extracts or generates HTTP trace IDs, sets x-redirected-for,
// and injects W3C traceparent / x-trace-id / x-request-id into outgoing gRPC metadata.
func prepareUpstreamContext(r *http.Request, timeout time.Duration, clientIP string) (context.Context, context.CancelFunc) {
	var headers http.Header
	if r != nil {
		headers = r.Header
	}
	traceID, reqID := logging.ExtractHTTPTrace(headers)
	baseCtx := logging.WithTrace(context.Background(), traceID, reqID)
	ctx, cancel := context.WithTimeout(baseCtx, timeout)
	ctx = redirectors.SetRedirectedForHeader(ctx, clientIP)
	ctx, _, _ = logging.InjectOutgoingGRPCTrace(ctx)
	return ctx, cancel
}
