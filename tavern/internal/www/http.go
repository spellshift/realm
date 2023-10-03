package www

import (
	"net/http"
	"strings"

	"github.com/kcarretto/realm/tavern/internal/www/build"
)

// FallbackAppHandler is a custom handler for the single page react app - if the path doesn't exist the react app is returned.
type FallbackAppHandler struct{}

func (h *FallbackAppHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	// Extract the path from the http request
	path := r.URL.Path[1:]
	// Try to read the file requested
	resp, err := build.Content.ReadFile(path)
	if err != nil {
		// If the file doesn't exist
		if strings.Contains(err.Error(), "file does not exist") {
			// Return our react apps main page and let it handle the route using react router.
			resp, err := build.Content.ReadFile("index.html")
			if err != nil {
				println("Read error")
			}
			// Our index page will always be html
			w.Header().Add("Content-Type", "text/html")
			w.Write(resp)
		} else {
			println("Real error") // Should probably use the logging system.
		}
	} else {
		// If the file does exist then it's a real embeded file and we'll write it's contents back.
		// Since `w.Write` isn't aware of the file type we need to manually add the MIME types for files we'll serve.
		// These MIME types only need to account for the files we'll be embedding through the `./build/` directory.
		if strings.HasSuffix(path, ".css") {
			w.Header().Add("Content-Type", "text/css")
		} else if strings.HasSuffix(path, ".js") {
			w.Header().Add("Content-Type", "text/javascript")
		} else if strings.HasSuffix(path, ".html") {
			w.Header().Add("Content-Type", "text/html")
		}
		w.Write(resp)
	}
}
