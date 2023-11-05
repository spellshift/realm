package http

import "net/http"

// ServeMux used for endpoint registration.
type ServeMux interface {
	Handle(pattern string, handler http.Handler)
}

// A RouteMap contains a mapping of route patterns to Endpoints.
type RouteMap map[string]*Endpoint

// Register all endpoints with the configured patterns on the provided router.
func (routes RouteMap) Register(router ServeMux) {
	for route, handler := range routes {
		router.Handle(route, handler)
	}
}
