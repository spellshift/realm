package http

import (
	"context"
	"fmt"
	"log/slog"
	"net/http"
	"strings"
	"time"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	entuser "realm.pub/tavern/internal/ent/user"
)

// An Authenticator returns an authenticated context based on the given http request.
// If no authenticated identity is associated with the request, the authenticator should
// just return the request context, and it should not error in this case. This is to allow
// unauthenticated endpoints to function as expected, authorization will occur later in the stack.
type Authenticator interface {
	Authenticate(w http.ResponseWriter, r *http.Request) (context.Context, error)
}

type bypassAuthenticator struct {
	graph *ent.Client
}

// Authenticate the request with authentication bypass enabled.
// This will automatically authenticate requests as the first available user in the db.
// If no users exist in the db, it will create one and authenticate as that user.
// The created user will automatically be set to activated and promoted to administrator.
// Returns an *http.Request associated with the authenticated identity.
// Any issues with this authenticator will cause a panic.
func (authenticator *bypassAuthenticator) Authenticate(w http.ResponseWriter, r *http.Request) (context.Context, error) {
	// Read SessionToken from auth cookie
	authCookie, err := r.Cookie(auth.SessionCookieName)
	var authUser *ent.User

	if err == nil && authCookie != nil {
		authUser, _ = authenticator.graph.User.Query().Where(entuser.SessionToken(authCookie.Value)).First(r.Context())
	}

	// If the cookie didn't match or wasn't provided, try to find any existing user to act as the bypass user
	if authUser == nil {
		authUser, _ = authenticator.graph.User.Query().First(r.Context())
	}

	// If there are still no users in the database at all, create the initial bypass user
	if authUser == nil {
		uid := fmt.Sprintf("bypass-%x", time.Now().UnixNano()%1000000)
		authUser = authenticator.graph.User.Create().
			SetName(uid).
			SetOauthID(uid).
			SetPhotoURL("https://upload.wikimedia.org/wikipedia/commons/a/ac/Default_pfp.jpg").
			SetIsActivated(true).
			SetIsAdmin(true).
			SaveX(r.Context())
	}

	// Always ensure the client has the cookie for the user we decided to bypass as
	http.SetCookie(w, &http.Cookie{
		Name:     auth.SessionCookieName,
		Value:    authUser.SessionToken,
		Path:     "/",
		HttpOnly: true,
		Expires:  time.Now().Add(24 * time.Hour),
	})

	return auth.ContextFromSessionToken(r.Context(), authenticator.graph, authUser.SessionToken)
}

type requestAuthenticator struct {
	graph *ent.Client
}

// Authenticate the request using the provided HTTP request.
// Returns a context associated with the authenticated identity.
// If no authenticated identity is associated with the request, no error is returned.
// Instead, the context will not be associated with an authenticated identity.
func (authenticator *requestAuthenticator) Authenticate(w http.ResponseWriter, r *http.Request) (context.Context, error) {
	// Check for Access Token header (X-Tavern-Access-Token)
	accessToken := r.Header.Get(auth.HeaderAPIAccessToken)
	if accessToken != "" {
		authCtx, err := auth.ContextFromAccessToken(r.Context(), authenticator.graph, accessToken)
		if err != nil {
			slog.ErrorContext(r.Context(), "failed to authenticate access token from header", "err", err)
			return nil, ErrInvalidAccessToken
		}
		return authCtx, nil
	}

	// Check for Authorization: Bearer <token> header (used by MCP OAuth clients)
	if authHeader := r.Header.Get("Authorization"); authHeader != "" {
		if strings.HasPrefix(authHeader, "Bearer ") {
			bearerToken := strings.TrimPrefix(authHeader, "Bearer ")
			if bearerToken != "" {
				authCtx, err := auth.ContextFromAccessToken(r.Context(), authenticator.graph, bearerToken)
				if err != nil {
					slog.ErrorContext(r.Context(), "failed to authenticate bearer token from Authorization header", "err", err)
					return nil, ErrInvalidAccessToken
				}
				return authCtx, nil
			}
		}
	}

	// Read SessionToken from auth cookie
	authCookie, err := r.Cookie(auth.SessionCookieName)
	if err != nil && err != http.ErrNoCookie {
		slog.ErrorContext(r.Context(), "failed to read auth cookie", "err", err)
		return nil, ErrReadingAuthCookie
	}

	// If no auth cookie provided, do not authenticate the context
	if authCookie == nil {
		return r.Context(), nil
	}

	// Create an authenticated context (if provided cookie is valid)
	authCtx, err := auth.ContextFromSessionToken(r.Context(), authenticator.graph, authCookie.Value)
	if err != nil {
		slog.ErrorContext(r.Context(), "failed to create session from auth cookie", "err", err)
		return nil, ErrInvalidAuthCookie
	}

	return authCtx, nil
}

func resetAuthCookie(w http.ResponseWriter) {
	http.SetCookie(w, &http.Cookie{
		Name:     auth.SessionCookieName,
		Value:    "",
		Path:     "/",
		HttpOnly: true,
		Expires:  time.Unix(0, 0),
	})
}
