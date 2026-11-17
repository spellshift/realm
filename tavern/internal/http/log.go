package http

import (
	"bufio"
	"fmt"
	"net"
	"net/http"
	"strings"
	"time"

	"realm.pub/tavern/internal/logging"
)

var (
	httpAccessLog  = logging.For("http.access")
	authSessionLog = logging.For("auth.session")
)

// A Logger to display information about the incoming request.
type Logger interface {
	Log(r *http.Request)
}

// defaultRequestLogger enables post-execution structured http.access logging on Server.
type defaultRequestLogger struct{}

func (logger defaultRequestLogger) Log(r *http.Request) {
	clientIP := logging.ClientIPFromContext(r.Context())
	if clientIP == "" {
		clientIP = logging.NormalizeClientIP(
			r.RemoteAddr,
			r.Header.Get("x-redirected-for"),
			r.Header.Get("x-forwarded-for"),
		)
	}
	logHTTPAccess(r, http.StatusOK, 0, 0, clientIP)
}

type responseRecorder struct {
	http.ResponseWriter
	statusCode   int
	bytesWritten int64
	wroteHeader  bool
}

func newResponseRecorder(w http.ResponseWriter) *responseRecorder {
	return &responseRecorder{
		ResponseWriter: w,
		statusCode:     http.StatusOK,
	}
}

func (r *responseRecorder) WriteHeader(code int) {
	if !r.wroteHeader {
		r.statusCode = code
		r.wroteHeader = true
	}
	r.ResponseWriter.WriteHeader(code)
}

func (r *responseRecorder) Write(b []byte) (int, error) {
	if !r.wroteHeader {
		r.statusCode = http.StatusOK
		r.wroteHeader = true
	}
	n, err := r.ResponseWriter.Write(b)
	r.bytesWritten += int64(n)
	return n, err
}

func (r *responseRecorder) Flush() {
	if flusher, ok := r.ResponseWriter.(http.Flusher); ok {
		flusher.Flush()
	}
}

func (r *responseRecorder) Hijack() (net.Conn, *bufio.ReadWriter, error) {
	hijacker, ok := r.ResponseWriter.(http.Hijacker)
	if !ok {
		return nil, nil, fmt.Errorf("%w: underlying ResponseWriter does not implement http.Hijacker", http.ErrNotSupported)
	}
	conn, rw, err := hijacker.Hijack()
	if err == nil && !r.wroteHeader {
		r.statusCode = http.StatusSwitchingProtocols
		r.wroteHeader = true
	}
	return conn, rw, err
}

func (r *responseRecorder) Unwrap() http.ResponseWriter {
	return r.ResponseWriter
}

func isGRPCRequest(r *http.Request) bool {
	if strings.HasPrefix(strings.ToLower(r.Header.Get("Content-Type")), "application/grpc") {
		return true
	}
	path := r.URL.Path
	return strings.HasPrefix(path, "/c2.C2/") ||
		strings.HasPrefix(path, "/portal.Portal/") ||
		strings.HasPrefix(path, "/builder.Builder/")
}

func isLowNoiseHTTPPath(path string) bool {
	if path == "/status" || path == "/internal/host-check" || path == "/favicon.ico" || path == "/manifest.json" {
		return true
	}
	return strings.HasPrefix(path, "/static/") || strings.HasPrefix(path, "/logo")
}

func logHTTPAccess(r *http.Request, statusCode int, bytesWritten int64, duration time.Duration, clientIP string) {
	if isGRPCRequest(r) {
		return
	}

	ctx := r.Context()
	path := r.URL.Path
	attrs := []any{
		"method", r.Method,
		"path", path,
		"status_code", statusCode,
		"bytes_written", bytesWritten,
		"duration_ms", duration.Milliseconds(),
		"client_ip", clientIP,
		"user_agent", r.UserAgent(),
	}

	switch {
	case statusCode >= http.StatusInternalServerError:
		httpAccessLog.ErrorContext(ctx, "http request failed", attrs...)
	case statusCode == http.StatusUnauthorized || statusCode == http.StatusForbidden:
		httpAccessLog.WarnContext(ctx, "http request unauthorized", attrs...)
	case statusCode < http.StatusBadRequest && isLowNoiseHTTPPath(path):
		httpAccessLog.DebugContext(ctx, "http request completed", attrs...)
	default:
		httpAccessLog.InfoContext(ctx, "http request completed", attrs...)
	}
}
