// Package errors provides internal error handling and presentation for Tavern.
package errors

import (
	"fmt"
	"net/http"
)

// HTTP wraps an error with a desired response status code, useful for error presentation.
type HTTP struct {
	WrappedErr error
	StatusCode int
}

// Error presents the underlying error
func (err HTTP) Error() string {
	return err.WrappedErr.Error()
}

// NewHTTP creates a new wrapped HTTP error that can be used for error presentation.
func NewHTTP(msg string, statusCode int) HTTP {
	return HTTP{WrappedErr: fmt.Errorf("%s", msg), StatusCode: statusCode}
}

// WrapHandler provides middleware for handlers to present errors to the user if any occur.
func WrapHandler(fn func(http.ResponseWriter, *http.Request) error) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		err := fn(w, req)
		if err == nil {
			return
		}

		// Handle Expected HTTP Errors
		if httpErr, ok := err.(HTTP); ok {
			http.Error(w, httpErr.Error(), httpErr.StatusCode)
			return
		}

		// Unhandled Errors
		http.Error(w, err.Error(), http.StatusInternalServerError)
	})
}
