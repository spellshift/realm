package http

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"time"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
)

// An Authenticator returns an authenticated context based on the given http request.
// If no authenticated identity is associated with the request, the authenticator should
// just return the request context, and it should not error in this case. This is to allow
// unauthenticated endpoints to function as expected, authorization will occur later in the stack.
type Authenticator interface {
	Authenticate(r *http.Request) (context.Context, error)
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
func (authenticator *bypassAuthenticator) Authenticate(r *http.Request) (context.Context, error) {
	// Authenticate as the first available user, create one if none exist
	authUser, err := authenticator.graph.User.Query().First(r.Context())
	if err != nil || authUser == nil {
		if !ent.IsNotFound(err) {
			panic(fmt.Errorf("bypass authenticator: failed to lookup users: %w", err))
		}

		authUser = authenticator.graph.User.Create().
			SetName("auth-disabled").
			SetOauthID("auth-disabled").
			SetPhotoURL("https://upload.wikimedia.org/wikipedia/commons/a/ac/Default_pfp.jpg").
			SetIsActivated(true).
			SetIsAdmin(true).
			SaveX(r.Context())
	}

	return auth.ContextFromSessionToken(r.Context(), authenticator.graph, authUser.SessionToken)
}

type cookieAuthenticator struct {
	graph *ent.Client
}

// Authenticate the request using the provided HTTP cookie.
// Returns an *http.Request associated with the authenticated identity.
// Returns nil if there was an authentication error, and writes error
// messages to the provided writer.
// If no authenticated identity is associated with the request, no error is returned.
// Instead, the context will not be associated with an authenticated identity.
func (authenticator *cookieAuthenticator) Authenticate(r *http.Request) (context.Context, error) {
	// Read SessionToken from auth cookie
	authCookie, err := r.Cookie(auth.SessionCookieName)
	if err != nil && err != http.ErrNoCookie {
		log.Printf("[ERROR] failed to read auth cookie: %v", err)
		return nil, ErrReadingAuthCookie
	}

	// If no auth cookie provided, do not authenticate the context
	if authCookie == nil {
		return r.Context(), nil
	}

	// Create an authenticated context (if provided cookie is valid)
	authCtx, err := auth.ContextFromSessionToken(r.Context(), authenticator.graph, authCookie.Value)
	if err != nil {
		log.Printf("failed to create session from auth cookie: %v", err)
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
