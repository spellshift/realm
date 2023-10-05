package www

import (
	"log"
	"net/http"
	"strings"

	"github.com/kcarretto/realm/tavern/internal/www/build"
)

// Handler is a custom handler for the single page react app - if the path doesn't exist the react app is returned.
type Handler struct {
	logger *log.Logger
}

// ServeHTTP provides the Tavern UI, if the requested file does not exist it will serve index.html
func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {

	// Serve the requested file
	path := strings.TrimPrefix(r.URL.Path, "/")
	content, err := build.Content.ReadFile(path)
	if err == nil {
		if strings.HasSuffix(path, ".css") {
			w.Header().Add("Content-Type", "text/css")
		}
		w.Write(content)
		return
	}

	// Otherwise serve index.html
	index, err := build.Content.ReadFile("index.html")
	if err != nil {
		h.logger.Printf("fatal error: failed to read index.html: %v", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}
	w.Write(index)
}

// NewHandler creates and returns a handler for the Tavern UI.
func NewHandler(logger *log.Logger) *Handler {
	return &Handler{logger}
}
