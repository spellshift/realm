package http

import (
	"fmt"
	"log"
	"net/http"
	"time"

	"github.com/kcarretto/realm/tavern/internal/auth"
	"github.com/kcarretto/realm/tavern/internal/ent"
)

// An authenticator associates an HTTP request with an authenticated identity.
type authenticator interface {
	Authenticate(req *http.Request) (*http.Request, *Error)
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
func (authenticator *bypassAuthenticator) Authenticate(req *http.Request) (*http.Request, *Error) {
	// Authenticate as the first available user, create one if none exist
	authUser, err := authenticator.graph.User.Query().First(req.Context())
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
			SaveX(req.Context())
	}

	ctx, err := auth.ContextFromSessionToken(req.Context(), authenticator.graph, authUser.SessionToken)
	if err != nil {
		panic(fmt.Errorf("bypass authenticator: failed to create authenticated session for user: %w", err))
	}
	return req.WithContext(ctx), nil
}

type cookieAuthenticator struct {
	graph *ent.Client
}

// Authenticate the request using the provided HTTP cookie.
// Returns an *http.Request associated with the authenticated identity.
// Returns nil if there was an authentication error, and writes error
// messages to the provided writer.
func (authenticator *cookieAuthenticator) Authenticate(req *http.Request) (*http.Request, *Error) {
	/*
	 * User Authentication
	 */
	authCookie, err := req.Cookie(auth.SessionCookieName)
	if err != nil && err != http.ErrNoCookie || authCookie == nil {
		log.Printf("failed to obtain auth cookie: %v", err)
		return nil, &Error{"failed to obtain auth cookie", http.StatusBadRequest}
	}

	authCtx, err := auth.ContextFromSessionToken(req.Context(), authenticator.graph, authCookie.Value)
	if err != nil {
		log.Printf("failed to create session from auth cookie: %v", err)
		return nil, &Error{"invalid auth cookie", http.StatusUnauthorized}
	}

	return req.WithContext(authCtx), nil
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
