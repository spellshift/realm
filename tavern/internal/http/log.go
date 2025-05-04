package http

import (
	"log/slog"
	"net/http"

	"realm.pub/tavern/internal/auth"
)

// A Logger to display information about the incoming request.
type Logger interface {
	Log(r *http.Request)
}

// defaultRequestLogger logs the incoming request to slog.
type defaultRequestLogger struct{}

func (logger defaultRequestLogger) Log(r *http.Request) {
	// Get authentication information if available
	var (
		authID          = "unknown"
		isActivated     = false
		isAdmin         = false
		isAuthenticated = false
		authUserID      = 0
		authUserName    = ""
	)
	id := auth.IdentityFromContext(r.Context())
	if id != nil {
		authID = id.String()
		isActivated = id.IsActivated()
		isAdmin = id.IsAdmin()
		isAuthenticated = id.IsAuthenticated()
	}
	if authUser := auth.UserFromContext(r.Context()); authUser != nil {
		authUserID = authUser.ID
		authUserName = authUser.Name
	}

	slog.InfoContext(
		r.Context(),
		"tavern http request",
		"auth_generic_id", authID,
		"auth_user_name", authUserName,
		"auth_user_id", authUserID,
		"is_admin", isAdmin,
		"is_activated", isActivated,
		"is_authenticated", isAuthenticated,
		"remote_addr", r.RemoteAddr,
		"http_method", r.Method,
		"http_url", r.URL,
	)
}
