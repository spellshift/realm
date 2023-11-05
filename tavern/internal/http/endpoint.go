package http

import "net/http"

type Endpoint struct {
	authenticator

	Logger  http.Handler
	Handler http.Handler
}

// ServeHTTP by wrapping the configured handler with middleware.
func (endpoint *Endpoint) ServeHTTP(w http.ResponseWriter, req *http.Request) {
	// // Authenticate requests (if configured)
	// if endpoint.authenticator != nil {
	req, err := endpoint.Authenticate(req)
	if err != nil {
		if err.Code == http.StatusUnauthorized {
			resetAuthCookie(w)
		}
		http.Error(w, err.Message, err.Code)
		return
	}

	// Log requests (if configured)
	if endpoint.Logger != nil {
		endpoint.Logger.ServeHTTP(w, req)
	}

	// Handle the request
	endpoint.Handler.ServeHTTP(w, req)
}
