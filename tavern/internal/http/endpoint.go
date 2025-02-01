package http

import (
	"log/slog"
	"net/http"

	"realm.pub/tavern/internal/auth"
)

// An Endpoint wraps an HTTP handler with configuration options.
type Endpoint struct {
	// LoginRedirectURI defines the path to redirect unauthenticated requests to.
	// If unset, no redirect will be performed.
	LoginRedirectURI string

	// AllowUnauthenticated requests, if false unauthenticated requests will error.
	// AllowUnactivated allows unactivated user accounts to access the endpoint.
	AllowUnauthenticated bool
	AllowUnactivated     bool

	http.Handler
}

// ServeHTTP traffic based on the configured handler.
func (endpoint Endpoint) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	// Redirect request to login (if configured)
	if endpoint.LoginRedirectURI != "" && !auth.IsAuthenticatedContext(ctx) {
		http.Redirect(w, r, endpoint.LoginRedirectURI, http.StatusFound)
		return
	}

	// Require Authentication
	if !endpoint.AllowUnauthenticated && !auth.IsAuthenticatedContext(ctx) {
		slog.WarnContext(r.Context(), "http unauthenticated request forbidden", "http_method", r.Method, "http_url", r.URL.String())
		http.Error(w, "must authenticate", http.StatusUnauthorized)
		return
	}

	// Require Activation
	if !endpoint.AllowUnactivated && !auth.IsActivatedContext(ctx) {
		slog.WarnContext(ctx, "http unactivated user request forbidden", "http_method", r.Method, "http_url", r.URL.String())
		http.Error(w, "must be activated", http.StatusForbidden)
		return
	}

	endpoint.Handler.ServeHTTP(w, r)
}
