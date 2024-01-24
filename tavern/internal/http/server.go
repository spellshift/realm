package http

import (
	"net/http"
	"time"
)

// A Server for Tavern HTTP traffic.
type Server struct {
	Authenticator
	Logger
	http.Handler
}

func (srv *Server) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	start := time.Now()

	// Authenticate Request (if possible)
	ctx, err := srv.Authenticate(r)
	if err != nil {
		switch err {
		case ErrInvalidAuthCookie:
			resetAuthCookie(w)
			http.Error(w, "invalid auth cookie", http.StatusUnauthorized)
			return
		case ErrReadingAuthCookie:
			resetAuthCookie(w)
			http.Error(w, "failed to read auth cookie", http.StatusBadRequest)
			return
		default:
			resetAuthCookie(w)
			http.Error(w, "unexpected error occurred", http.StatusInternalServerError)
			return
		}
	}
	r = r.WithContext(ctx)

	// Metrics
	defer func() {
		// Increment total requests counter
		metricHTTPRequests.WithLabelValues(r.RequestURI, r.Method).Inc()

		// Record the latency
		metricHTTPLatency.WithLabelValues(r.RequestURI, r.Method).Observe(time.Since(start).Seconds())

		// Record if there was an error
		if err != nil {
			metricHTTPErrors.WithLabelValues(r.RequestURI, r.Method).Inc()
		}
	}()

	// Log Request
	if srv.Logger != nil {
		srv.Log(r)
	}

	// Handle Request
	srv.Handler.ServeHTTP(w, r)
}
