package http

import (
	"net/http"
)

// A Server for Tavern HTTP traffic.
type Server struct {
	Authenticator
	Logger
	http.Handler
}

func (srv *Server) ServeHTTP(w http.ResponseWriter, r *http.Request) {
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

	// Log Request
	if srv.Logger != nil {
		srv.Log(r)
	}

	// Handle Request
	srv.Handler.ServeHTTP(w, r)
}
