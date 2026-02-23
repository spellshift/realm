package www

import (
	"embed"
	"fmt"
	"log"
	"net/http"
	"strings"
)

// Handler is a custom handler for the single page react app - if the path doesn't exist the react app is returned.
type Handler struct {
	logger *log.Logger
}

// Content embedded from the application's build directory, includes the latest build of the UI.
//
//go:embed build/*.png build/*.html build/*.json build/*.txt build/*.ico
//go:embed build/static/*
//go:embed build/wasm/*
var Content embed.FS

// ServeHTTP provides the Tavern UI, if the requested file does not exist it will serve index.html
func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {

	// Serve the requested file
	path := fmt.Sprintf("build/%s", strings.TrimPrefix(r.URL.Path, "/"))
	content, err := Content.ReadFile(path)
	if err == nil {
		if strings.HasSuffix(path, ".css") {
			w.Header().Add("Content-Type", "text/css")
		} else if strings.HasSuffix(path, ".js") {
			w.Header().Add("Content-Type", "application/javascript")
		} else if strings.HasSuffix(path, ".wasm") {
			w.Header().Add("Content-Type", "application/wasm")
		}
		w.Write(content)
		return
	}

	// Otherwise serve index.html
	index, err := Content.ReadFile("build/index.html")
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
