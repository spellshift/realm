package http

import (
	"net/http"

	"realm.pub/tavern/internal/auth"
)

// An Endpoint wraps an HTTP handler with configuration options.
type Endpoint struct {
	// LoginRedirectURI defines the path to redirect unauthenticated requests to.
	// If unset, no redirect will be performed.
	LoginRedirectURI string

	http.Handler
}

// ServeHTTP traffic based on the configured handler.
func (endpoint Endpoint) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	// Redirect request to login (if configured)
	if endpoint.LoginRedirectURI != "" && !auth.IsAuthenticatedContext(r.Context()) {
		http.Redirect(w, r, endpoint.LoginRedirectURI, http.StatusFound)
		return
	}

	endpoint.Handler.ServeHTTP(w, r)
}
