package http

import (
	"net/http"
	"strings"
	"time"

	"realm.pub/tavern/internal/logging"
)

// A Server for Tavern HTTP traffic.
type Server struct {
	Authenticator
	Logger
	http.Handler
}

func (srv *Server) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	start := time.Now()

	traceID, reqID := logging.ExtractHTTPTrace(r.Header)
	w.Header().Set(logging.HeaderTraceID, traceID)
	w.Header().Set(logging.HeaderRequestID, reqID)

	clientIP := logging.NormalizeClientIP(
		r.RemoteAddr,
		r.Header.Get("x-redirected-for"),
		r.Header.Get("x-forwarded-for"),
	)
	entrypoint := "http"
	if strings.EqualFold(r.Header.Get("Upgrade"), "websocket") {
		entrypoint = "websocket"
	}

	ctx := r.Context()
	ctx = logging.WithTrace(ctx, traceID, reqID)
	ctx = logging.WithClientIP(ctx, clientIP)
	ctx = logging.WithEntrypoint(ctx, entrypoint)
	r = r.WithContext(ctx)

	rec := newResponseRecorder(w)

	defer func() {
		duration := time.Since(start)
		metricHTTPRequests.WithLabelValues(r.RequestURI, r.Method).Inc()
		metricHTTPLatency.WithLabelValues(r.RequestURI, r.Method).Observe(duration.Seconds())

		if srv.Logger != nil {
			if _, isDefault := srv.Logger.(defaultRequestLogger); !isDefault {
				srv.Logger.Log(r)
			}
			logHTTPAccess(r, rec.statusCode, rec.bytesWritten, duration, clientIP)
		}
	}()

	// Authenticate Request (if possible)
	if srv.Authenticator != nil {
		authCtx, err := srv.Authenticate(rec, r)
		if err != nil {
			switch err {
			case ErrInvalidAccessToken:
				http.Error(rec, "invalid access token", http.StatusUnauthorized)
				return
			case ErrInvalidAuthCookie:
				resetAuthCookie(rec)
				http.Error(rec, "invalid auth cookie", http.StatusUnauthorized)
				return
			case ErrReadingAuthCookie:
				resetAuthCookie(rec)
				http.Error(rec, "failed to read auth cookie", http.StatusBadRequest)
				return
			default:
				http.Error(rec, "unexpected error occurred", http.StatusInternalServerError)
				return
			}
		}
		r = r.WithContext(authCtx)
	}

	// Handle Request
	srv.Handler.ServeHTTP(rec, r)
}
