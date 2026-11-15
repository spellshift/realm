package logging

import (
	"context"
	"crypto/rand"
	"encoding/binary"
	"encoding/hex"
	"net/http"
	"strings"
	"sync/atomic"
	"time"

	"google.golang.org/grpc/metadata"
)

const (
	// HTTP & gRPC header names for trace and request correlation.
	HeaderTraceID      = "X-Trace-ID"
	HeaderRequestID    = "X-Request-ID"
	HeaderTraceparent  = "traceparent"
	HeaderCloudTrace   = "X-Cloud-Trace-Context"
	MetadataTraceIDKey = "x-trace-id"
	MetadataReqIDKey   = "x-request-id"
)

var fallbackSeq atomic.Uint64

// NewTraceID generates a 128-bit (32 lowercase hex character) trace identifier.
func NewTraceID() string {
	var buf [16]byte
	if _, err := rand.Read(buf[:]); err != nil {
		binary.BigEndian.PutUint64(buf[:8], uint64(time.Now().UnixNano()))
		binary.BigEndian.PutUint64(buf[8:], fallbackSeq.Add(1))
	}
	return hex.EncodeToString(buf[:])
}

// NewRequestID generates a 64-bit (16 lowercase hex character) request/span identifier.
func NewRequestID() string {
	var buf [8]byte
	if _, err := rand.Read(buf[:]); err != nil {
		binary.BigEndian.PutUint64(buf[:], fallbackSeq.Add(1)^uint64(time.Now().UnixNano()))
	}
	return hex.EncodeToString(buf[:])
}

// FormatTraceparent formats a W3C Trace Context traceparent header ("00-<traceID>-<spanID>-01").
func FormatTraceparent(traceID, spanID string) string {
	if !isValidHexNonZero(traceID, 32) {
		traceID = NewTraceID()
	}
	if !isValidHexNonZero(spanID, 16) {
		spanID = NewRequestID()
	}
	return "00-" + strings.ToLower(traceID) + "-" + strings.ToLower(spanID) + "-01"
}

// ExtractHTTPTrace extracts or generates (traceID, requestID) from HTTP request headers.
func ExtractHTTPTrace(h http.Header) (traceID, requestID string) {
	if h != nil {
		traceID = extractTraceIDFromValues(
			h.Get(HeaderTraceparent),
			h.Get(HeaderCloudTrace),
			h.Get(HeaderTraceID),
		)
		requestID = sanitizeIdentifier(h.Get(HeaderRequestID), 64)
	}
	if traceID == "" {
		traceID = NewTraceID()
	}
	if requestID == "" {
		requestID = NewRequestID()
	}
	return traceID, requestID
}

// ExtractGRPCTrace extracts or generates (traceID, requestID) from incoming gRPC context/metadata,
// preserving any trace already attached to ctx (such as when served via grpc.Server.ServeHTTP).
func ExtractGRPCTrace(ctx context.Context) (context.Context, string, string) {
	existingTrace := TraceIDFromContext(ctx)
	existingReq := RequestIDFromContext(ctx)
	if existingTrace != "" && existingReq != "" {
		return ctx, existingTrace, existingReq
	}

	var traceID, requestID string
	if md, ok := metadata.FromIncomingContext(ctx); ok {
		traceID = extractTraceIDFromValues(
			firstMD(md, "traceparent"),
			firstMD(md, "x-cloud-trace-context"),
			firstMD(md, MetadataTraceIDKey),
		)
		requestID = sanitizeIdentifier(firstMD(md, MetadataReqIDKey), 64)
	}

	if existingTrace != "" {
		traceID = existingTrace
	}
	if traceID == "" {
		traceID = NewTraceID()
	}
	if existingReq != "" {
		requestID = existingReq
	}
	if requestID == "" {
		requestID = NewRequestID()
	}

	return WithTrace(ctx, traceID, requestID), traceID, requestID
}

// InjectOutgoingGRPCTrace ensures trace_id and request_id exist on ctx and appends W3C `traceparent`,
// `x-trace-id`, and `x-request-id` to outgoing gRPC metadata (used by Redirectors -> Tavern).
func InjectOutgoingGRPCTrace(ctx context.Context) (context.Context, string, string) {
	ctx, traceID, reqID := EnsureTrace(ctx)
	ctx = metadata.AppendToOutgoingContext(
		ctx,
		HeaderTraceparent, FormatTraceparent(traceID, reqID),
		MetadataTraceIDKey, traceID,
		MetadataReqIDKey, reqID,
	)
	return ctx, traceID, reqID
}

func firstMD(md metadata.MD, key string) string {
	vals := md.Get(key)
	if len(vals) == 0 {
		return ""
	}
	return vals[0]
}

func extractTraceIDFromValues(traceparent, cloudTrace, xTraceID string) string {
	if tid := parseTraceparent(traceparent); tid != "" {
		return tid
	}
	if tid := parseCloudTraceContext(cloudTrace); tid != "" {
		return tid
	}
	if isValidHexNonZero(xTraceID, 32) {
		return strings.ToLower(xTraceID)
	}
	return sanitizeIdentifier(xTraceID, 64)
}

func parseTraceparent(raw string) string {
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return ""
	}
	parts := strings.Split(raw, "-")
	if len(parts) < 4 {
		return ""
	}
	if len(parts[0]) != 2 || !isValidHex(parts[0]) || strings.EqualFold(parts[0], "ff") {
		return ""
	}
	if !isValidHexNonZero(parts[1], 32) || !isValidHexNonZero(parts[2], 16) {
		return ""
	}
	return strings.ToLower(parts[1])
}

func parseCloudTraceContext(raw string) string {
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return ""
	}
	tracePart := raw
	if slash := strings.IndexByte(tracePart, '/'); slash >= 0 {
		tracePart = tracePart[:slash]
	}
	if semi := strings.IndexByte(tracePart, ';'); semi >= 0 {
		tracePart = tracePart[:semi]
	}
	if !isValidHexNonZero(tracePart, 32) {
		return ""
	}
	return strings.ToLower(tracePart)
}

func isValidHexNonZero(s string, wantLen int) bool {
	if len(s) != wantLen || !isValidHex(s) {
		return false
	}
	for i := 0; i < len(s); i++ {
		if s[i] != '0' {
			return true
		}
	}
	return false
}

func isValidHex(s string) bool {
	if s == "" {
		return false
	}
	for i := 0; i < len(s); i++ {
		c := s[i]
		isDigit := c >= '0' && c <= '9'
		isLower := c >= 'a' && c <= 'f'
		isUpper := c >= 'A' && c <= 'F'
		if !isDigit && !isLower && !isUpper {
			return false
		}
	}
	return true
}

func sanitizeIdentifier(raw string, maxLen int) string {
	s := strings.TrimSpace(raw)
	if s == "" || len(s) > maxLen {
		return ""
	}
	for i := 0; i < len(s); i++ {
		c := s[i]
		ok := (c >= 'a' && c <= 'z') ||
			(c >= 'A' && c <= 'Z') ||
			(c >= '0' && c <= '9') ||
			c == '-' || c == '_' || c == '.'
		if !ok {
			return ""
		}
	}
	return s
}
