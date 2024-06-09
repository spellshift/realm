package http

import (
	"log"
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
		log.Printf("[HTTP] Unauthenticated Request Forbidden: %s %q", r.Method, r.URL.String())
		http.Error(w, "must authenticate", http.StatusUnauthorized)
		return
	}

	// Require Activation
	if !endpoint.AllowUnactivated && !auth.IsActivatedContext(ctx) {
		log.Printf("[HTTP] Unactivated User Request Forbidden: %s %q", r.Method, r.URL.String())
		http.Error(w, "must be activated", http.StatusForbidden)
		return
	}

	endpoint.Handler.ServeHTTP(w, r)
}

// An Endpoint wraps an HTTP handler with configuration options.
type EncryptedEndpoint struct {
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
func (endpoint EncryptedEndpoint) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	// Redirect request to login (if configured)
	if endpoint.LoginRedirectURI != "" && !auth.IsAuthenticatedContext(ctx) {
		http.Redirect(w, r, endpoint.LoginRedirectURI, http.StatusFound)
		return
	}

	// Require Authentication
	if !endpoint.AllowUnauthenticated && !auth.IsAuthenticatedContext(ctx) {
		log.Printf("[HTTP] Unauthenticated Request Forbidden: %s %q", r.Method, r.URL.String())
		http.Error(w, "must authenticate", http.StatusUnauthorized)
		return
	}

	// Require Activation
	if !endpoint.AllowUnactivated && !auth.IsActivatedContext(ctx) {
		log.Printf("[HTTP] Unactivated User Request Forbidden: %s %q", r.Method, r.URL.String())
		http.Error(w, "must be activated", http.StatusForbidden)
		return
	}

	// csvc := crypto.NewCryptoSvc([]byte("helloworld"))

	// // Decrypt grpc message
	// r.Body = crypto.RequestBodyWrapper{Csvc: csvc, Body: r.Body}

	// // Encrypt grpc response
	// rww := crypto.NewResponseWriterWrapper(csvc, w)

	endpoint.Handler.ServeHTTP(w, r)
}
