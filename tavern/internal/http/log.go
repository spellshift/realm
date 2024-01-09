package http

import (
	"log"
	"net/http"

	"realm.pub/tavern/internal/auth"
)

// A Logger to display information about the incoming request.
type Logger interface {
	Log(r *http.Request)
}

type defaultRequestLogger struct {
	*log.Logger
}

func (logger defaultRequestLogger) Log(r *http.Request) {
	authName := "unknown"
	id := auth.IdentityFromContext(r.Context())
	if id != nil {
		authName = id.String()
	}

	logger.Logger.Printf("%s (%s) %s %s\n", r.RemoteAddr, authName, r.Method, r.URL)
}
