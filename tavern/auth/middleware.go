package auth

import (
	"fmt"
	"log"
	"net/http"
	"time"

	"github.com/kcarretto/realm/tavern/ent"
)

// AuthDisabledMiddleware should only be used when authentication has been disabled.
func AuthDisabledMiddleware(handler http.Handler, graph *ent.Client) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Authenticate as the first available user, create one if none exist
		authUser, err := graph.User.Query().First(r.Context())
		if err != nil || authUser == nil {
			if !ent.IsNotFound(err) {
				panic(fmt.Errorf("failed to lookup users: %w", err))
			}

			authUser = graph.User.Create().
				SetName("auth-disabled").
				SetOAuthID("auth-disabled").
				SetPhotoURL("https://upload.wikimedia.org/wikipedia/commons/a/ac/Default_pfp.jpg").
				SetIsActivated(true).
				SetIsAdmin(true).
				SaveX(r.Context())
		}

		ctx, err := ContextFromSessionToken(r.Context(), graph, authUser.SessionToken)
		if err != nil {
			panic(fmt.Errorf("failed to create authenticated session for user: %w", err))
		}

		r = r.WithContext(ctx)
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
