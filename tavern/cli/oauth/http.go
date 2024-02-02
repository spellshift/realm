package oauth

import (
	"context"
	"fmt"
	"io"
	"log"
	"net"
	"net/http"
	"net/url"
	"sync"
	"time"

	"realm.pub/tavern/internal/auth"
)

// RedactedToken is value returned by calling String() on a Token.
const RedactedToken = "--REDACTED--"

// Token is a type alias for string that prevents sensitive information from being displayed.
type Token string

// String returns the RedactedToken message to avoid accidentally printing sensitive information.
// In order to display the token intentionally, you must explicitly cast it to a string before printing.
func (Token) String() string {
	return RedactedToken
}

// Authenticate the provided http request, using this token to authenticate to the Tavern API.
// It is recommended to use this method instead of manually configuring the request, such that
// if authentication implementation details are changed, the request will still be properly authenticated.
func (token Token) Authenticate(r *http.Request) {
	r.Header.Set(auth.HeaderAPIAccessToken, string(token))
}

// Browser that will be opened to the Tavern OAuth consent flow.
type Browser interface {
	OpenURL(url string) error
}

// Authenticate the user to the tavern API using the provided browser.
// This will open the browser to a login URL, which will redirect to an http server (hosted locally)
// with authentication credentials. This prevents the need for copy pasting tokens manually.
//
// After authenticating, the resulting Token may be used to authenticate to Tavern for HTTP requests.
// This should be done by calling the `Authenticate(request)` method on the returned token.
func Authenticate(ctx context.Context, browser Browser, tavernURL string) (Token, error) {
	// Create Listener
	conn, err := net.Listen("tcp", ":0")
	if err != nil {
		return Token(""), fmt.Errorf("failed to start oauth http redirect handler: %w", err)
	}
	defer conn.Close()

	// Build OAuth Endpoint
	oauthLoginURL, err := url.Parse(tavernURL)
	if err != nil {
		return Token(""), fmt.Errorf("failed to parse tavern url: %w", err)
	}
	_, redirPort, err := net.SplitHostPort(conn.Addr().String())
	if err != nil {
		return Token(""), fmt.Errorf("failed to parse redirect port from listener addr: %q: %w", conn.Addr().String(), err)
	}
	oauthLoginURL.RawQuery = url.Values{
		auth.CLIRedirPortParamName: []string{redirPort},
	}.Encode()
	oauthLoginURL.Path = "/oauth/cli/login"

	// Log TLS Warning
	if oauthLoginURL.Scheme != "https" {
		log.Printf("[WARN] Using insecure OAuth URL (http), this may leak sensitive information")
	}

	// Create Channels
	tokenCh := make(chan Token, 1)
	errCh := make(chan error, 1)

	// Configure Server
	srv := &http.Server{
		Addr:    ":0",
		Handler: newTokenHandler(tokenCh, errCh),
	}

	// Start HTTP Server
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		if err := srv.Serve(conn); err != nil {
			errCh <- fmt.Errorf("http token redirect server failed: %w", err)
		}
	}()

	// Handle Cleanup
	defer func() {
		// Browsers keep open the connection, which means we must timeout the shutdown to
		// prevent the http package from waiting indefinitely.
		shutdownCtx, cancel := context.WithTimeout(ctx, 50*time.Millisecond)
		defer cancel()
		srv.Shutdown(shutdownCtx)
		wg.Wait()
	}()

	// Open Browser
	if err := browser.OpenURL(oauthLoginURL.String()); err != nil {
		return Token(""), fmt.Errorf("failed to open browser for oauth flow: %w", err)
	}

	// Wait for Token, Error, or Cancellation
	select {
	case <-ctx.Done():
		return Token(""), fmt.Errorf("authentication cancelled: %w", ctx.Err())
	case err := <-errCh:
		return Token(""), fmt.Errorf("failed to obtain credentials: %w", err)
	case token := <-tokenCh:
		return token, nil
	}
}

// newTokenHandler returns an HTTP handler that parses a token from an HTTP request.
// It will send the token to the provided token channel if successful.
// If any error occurs, it will be sent to the provided error channel.
func newTokenHandler(tokenCh chan<- Token, errCh chan<- error) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		params, err := url.ParseQuery(r.URL.RawQuery)
		if err != nil {
			http.Error(w, "failed to parse query params", http.StatusBadRequest)
			errCh <- err
			return
		}
		token := params.Get(auth.CLIRedirAPITokenParamName)
		if token == "" {
			http.Error(w, fmt.Sprintf("must set '%q' query param", auth.CLIRedirPortParamName), http.StatusBadRequest)
			errCh <- fmt.Errorf("must provide '%q' query parameter", auth.CLIRedirPortParamName)
			return
		}

		tokenCh <- Token(token)
		io.WriteString(w, "Succesfully Authenticated!")
	})
}
