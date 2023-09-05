package www

import (
	"net/http"

	"github.com/kcarretto/realm/tavern/internal/www/build"
)

// NewAppHandler returns a new handler to serve UI embedded UI files that strips the provided prefix from the route.
func NewAppHandler(prefix string) http.Handler {
	return http.FileServer(http.FS(build.Content))
}
