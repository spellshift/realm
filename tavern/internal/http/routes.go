package http

import "net/http"

// ServeMux used for endpoint registration.
type ServeMux interface {
	Handle(pattern string, handler http.Handler)
}

// A RouteMap contains a mapping of route patterns to http handlers.
type RouteMap map[string]http.Handler

// HandleFunc registers the handler function for the given pattern.
func (routes RouteMap) HandleFunc(pattern string, handler func(http.ResponseWriter, *http.Request)) {
	routes[pattern] = http.HandlerFunc(handler)
}

// Handle registers the handler for the given pattern.
// If a handler already exists for pattern, Handle panics.
func (routes RouteMap) Handle(pattern string, handler http.Handler) {
	routes[pattern] = handler
}
