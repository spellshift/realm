package auth

import (
	"fmt"
	"net"
	"net/http"
	"net/url"
	"strconv"
)

// ParamNameAuthRedirPort is the name of the query parameter PAT requests must set to indicate which local port
// the client should be redirected to.
const ParamTokenRedirPort = "redir_port"

// ParamTokenRedirToken is the name of the query parameter CLI OAuth http servers should parse to receive the
// Tavern API personal access token.
const ParamTokenRedirToken = "access_token"

// HeaderAPIAccessToken is the name of the header clients should set to authenticate with personal access tokens.
const HeaderAPIAccessToken = "X-Tavern-Access-Token"

// NewTokenRedirectHandler returns a new http endpoint that redirects the requestor to http://127.0.0.1 at the port specified
// in the query parameters. This method requires an authenticated session, and will set the user's personal access token in the redirected
// URL query parameters intended for use by CLI applications authenticating to Tavern.
func NewTokenRedirectHandler() http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		params, err := url.ParseQuery(r.URL.RawQuery)
		if err != nil {
			http.Error(w, fmt.Sprintf("failed to parse query params: %v", err), http.StatusBadRequest)
			return
		}
		redirPortStr := params.Get(ParamTokenRedirPort)
		if redirPortStr == "" {
			http.Error(w, fmt.Sprintf("must set %q query param", ParamTokenRedirPort), http.StatusBadRequest)
			return
		}
		redirPort, err := strconv.Atoi(redirPortStr)
		if err != nil {
			http.Error(w, fmt.Sprintf("invalid value provided for '%q': %v", ParamTokenRedirPort, err), http.StatusBadRequest)
			return
		}

		user := UserFromContext(r.Context())
		if user == nil {
			http.Error(w, "must be authenticated", http.StatusUnauthorized)
			return
		}
		redirParams := url.Values{
			ParamTokenRedirToken: []string{user.AccessToken},
		}

		redirUrl := url.URL{
			Scheme:   "http",
			Host:     net.JoinHostPort("127.0.0.1", fmt.Sprintf("%d", redirPort)),
			RawQuery: redirParams.Encode(),
		}

		http.Redirect(w, r, redirUrl.String(), http.StatusFound)
	})
}
