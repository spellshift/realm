package http

import (
	"log"
	"net/http"

	"realm.pub/tavern/internal/ent"
)

// An Option to configure a Tavern HTTP Server.
type Option func(*Server)

// NewServer configures a new server for serving HTTP traffic.
func NewServer(routes RouteMap, options ...Option) *Server {
	// Register routes
	router := http.NewServeMux()
	for route, handler := range routes {
		router.Handle(route, handler)
	}

	// Apply Options
	server := &Server{Handler: router}
	for _, opt := range options {
		opt(server)
	}

	return server
}

// WithAuthentication enables http request authentication for the server.
func WithAuthentication(graph *ent.Client) Option {
	return Option(func(server *Server) {
		server.Authenticator = &requestAuthenticator{graph}
	})
}

// WithAuthenticationBypass enables requests to bypass authentication for the server.
func WithAuthenticationBypass(graph *ent.Client) Option {
	return Option(func(server *Server) {
		server.Authenticator = &bypassAuthenticator{graph}
	})
}

// WithRequestLogging configures HTTP request logging for the server.
func WithRequestLogging(logger *log.Logger) Option {
	return Option(func(server *Server) {
		server.Logger = defaultRequestLogger{logger}
	})
}
