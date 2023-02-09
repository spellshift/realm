package authtest

import (
	"net/http"

	"github.com/kcarretto/realm/tavern/auth"
)

// Middleware that associates an all powerful test identity with the request context.
func Middleware(handler http.Handler) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		r = r.WithContext(
			auth.ContextFromIdentity(r.Context(), NewAllPowerfulIdentityForTest()),
		)
		handler.ServeHTTP(w, r)
	})
}
