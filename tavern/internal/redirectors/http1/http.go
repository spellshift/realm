package http1

import (
	"context"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"time"
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

// setGRPCResponseHeaders sets standard gRPC response headers
func setGRPCResponseHeaders(w http.ResponseWriter) {
	w.Header().Set("Content-Type", "application/grpc")
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
	slog.Error(fmt.Sprintf("[gRPC Stream Error] %s: %v\n", message, err))
	http.Error(w, fmt.Sprintf("%s: %v", message, err), http.StatusInternalServerError)
}

// createRequestContext creates a context with timeout
func createRequestContext(timeout time.Duration) (context.Context, context.CancelFunc) {
	return context.WithTimeout(context.Background(), timeout)
}
