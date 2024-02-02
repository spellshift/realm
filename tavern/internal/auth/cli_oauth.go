package auth

import (
	"fmt"
	"net"
	"net/http"
	"net/url"
	"strconv"
)

// CLIRedirPortParamName is the name of the query parameter CLI OAuth requests must set to indicate
// which local port the client should be redirected to.
const CLIRedirPortParamName = "redir_port"

// CLIRedirAPITokenParamName is the name of the query parameter CLI OAuth http servers should parse to receive the
// Tavern API personal access token.
const CLIRedirAPITokenParamName = "access_token"

// HeaderAPIAccessToken is the name of the header clients should set to authenticate with personal access tokens.
const HeaderAPIAccessToken = "X-Tavern-Access-Token"

// NewOAuthCLILoginHandler returns a new http endpoint that redirects the requestor to 127.0.0.1 at the port specified
// in the query parameters. This method requires an authenticated session, and will set an API key in the redirected
// URL query parameters intended for use by CLI applications authenticating to Tavern.
func NewOAuthCLILoginHandler() http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		params, err := url.ParseQuery(r.URL.RawQuery)
		if err != nil {
			http.Error(w, fmt.Sprintf("failed to parse query params: %v", err), http.StatusBadRequest)
			return
		}
		redirPortStr := params.Get(CLIRedirPortParamName)
		if redirPortStr == "" {
			http.Error(w, fmt.Sprintf("must set %q query param", CLIRedirPortParamName), http.StatusBadRequest)
			return
		}
		redirPort, err := strconv.Atoi(redirPortStr)
		if err != nil {
			http.Error(w, fmt.Sprintf("invalid value provided for '%q': %v", CLIRedirPortParamName, err), http.StatusBadRequest)
			return
		}

		user := UserFromContext(r.Context())
		if user == nil {
			http.Error(w, "must be authenticated", http.StatusUnauthorized)
			return
		}
		redirParams := url.Values{
			CLIRedirAPITokenParamName: []string{user.PersonalAccessToken},
		}

		redirUrl := url.URL{
			Scheme:   "http",
			Host:     net.JoinHostPort("127.0.0.1", fmt.Sprintf("%d", redirPort)),
			RawQuery: redirParams.Encode(),
		}

		http.Redirect(w, r, redirUrl.String(), http.StatusFound)
	})
}
