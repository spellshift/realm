package www

import (
	"net/http"

	"github.com/kcarretto/realm/tavern/internal/www/build"
)

// NewAppHandler returns a new handler to serve embedded UI files.
func NewAppHandler() http.Handler {
	return http.FileServer(http.FS(build.Content))
}
