package auth

import (
	"net/http"

	"github.com/kcarretto/realm/tavern/ent"
)

// authDisabledIdentity is used whenever authentication has been disabled.
type authDisabledIdentity struct{}

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
			panic(err)
		}
		if authCookie != nil {
			authCtx, err := ContextFromSessionToken(r.Context(), graph, authCookie.Value)
			if err != nil {
				panic(err)
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
