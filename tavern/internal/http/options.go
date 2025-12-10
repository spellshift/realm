package http

import (
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
	server := &Server{
		Handler: router,
		Logger:  defaultRequestLogger{},
	}
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

// WithRequestLogging configures specialized HTTP request logging for the server, overriding the default logger.
func WithRequestLogging(logger Logger) Option {
	return Option(func(server *Server) {
		server.Logger = logger
	})
}
