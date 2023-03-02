package auth

import (
	"log"
	"net/http"
	"time"

	"github.com/kcarretto/realm/tavern/ent"
)

// authDisabledIdentity is used whenever authentication has been disabled.
type authDisabledIdentity struct{}

func (id authDisabledIdentity) String() string        { return "auth_disabled" }
func (id authDisabledIdentity) IsAuthenticated() bool { return true }
func (id authDisabledIdentity) IsActivated() bool     { return true }
func (id authDisabledIdentity) IsAdmin() bool         { return true }

// AuthDisabledMiddleware should only be used when authentication has been disabled.
func AuthDisabledMiddleware(handler http.Handler) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		r = r.WithContext(
			ContextFromIdentity(r.Context(), authDisabledIdentity{}),
		)
		handler.ServeHTTP(w, r)
	})
}

// Middleware that associates the requestor identity with the request context.
func Middleware(handler http.Handler, graph *ent.Client) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {

		/*
		 * User Authentication
		 */
		authCookie, err := r.Cookie(SessionCookieName)
		if err != nil && err != http.ErrNoCookie {
			log.Printf("failed to obtain session cookie: %v", err)
			http.Error(w, "failed to obtain auth cookie", http.StatusBadRequest)
			return
		}
		if authCookie != nil {
			authCtx, err := ContextFromSessionToken(r.Context(), graph, authCookie.Value)
			if err != nil {
				log.Printf("failed to create session from cookie: %v", err)

				// Reset session token cookie
				http.SetCookie(w, &http.Cookie{
					Name:     SessionCookieName,
					Value:    "",
					Path:     "/",
					HttpOnly: true,
					Expires:  time.Unix(0, 0),
				})
				http.Error(w, "invalid auth cookie", http.StatusUnauthorized)

				return
			}

			r = r.WithContext(authCtx)
			handler.ServeHTTP(w, r)
			return
		}

		/*
		 * Agent Authentication
		 */
		// TODO

		handler.ServeHTTP(w, r)
	})
}
