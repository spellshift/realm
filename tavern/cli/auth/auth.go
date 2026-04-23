package auth

import (
	"context"
	"fmt"
	"io"
	"log/slog"
	"net"
	"net/http"
	"net/url"
	"os"
	"strings"
	"sync"
	"time"

	"realm.pub/tavern/internal/auth"
)

// HeaderAPIAccessToken is re-exported from internal/auth
const HeaderAPIAccessToken = auth.HeaderAPIAccessToken

// RedactedToken is the value returned by calling String() on a Token.
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

// Browser that will be opened to the Tavern authentication flow.
type Browser interface {
	OpenURL(url string) error
}

// The BrowserFunc type is an adapter to allow the use of ordinary functions as browsers.
// If f is a function with the appropriate signature, then BrowserFunc(f) is a Browser that calls f.
type BrowserFunc func(tavernURL string) error

// OpenURL calls f(tavernURL).
func (f BrowserFunc) OpenURL(tavernURL string) error {
	return f(tavernURL)
}

// Authenticate the user to the Tavern API using the provided browser.
// This will open the browser to a login URL, which will redirect to an http server (hosted locally)
// with authentication credentials. This prevents the need for copy pasting tokens manually.
//
// After authenticating, the resulting Token may be used to authenticate to Tavern for HTTP requests.
// This should be done by calling the `Authenticate(request)` method on the returned token.
func Authenticate(ctx context.Context, browser Browser, tavernURL string, opts ...AuthOption) (Token, error) {
	options := applyOptions(opts...)

	// 1. Check Environment Variable
	if options.EnvAPIKeyName != "" {
		if token := os.Getenv(options.EnvAPIKeyName); token != "" {
			return Token(token), nil
		}
	}

	// 2. Check Cache File
	if options.CachePath != "" {
		tokenData, err := os.ReadFile(options.CachePath)
		if err == nil {
			cachedToken := Token(tokenData)
			valid, valErr := validateToken(ctx, tavernURL, cachedToken)
			switch {
			case valErr != nil:
				// Unable to verify the cached token (e.g. network error).
				// Return the cached value optimistically rather than
				// forcing a re-authentication for transient failures.
				slog.Warn("failed to verify cached auth token, using cached value", "path", options.CachePath, "error", valErr)
				return cachedToken, nil
			case valid:
				slog.Debug("Loaded authentication credentials from cache", "path", options.CachePath)
				return cachedToken, nil
			default:
				// Cached credentials were rejected by the server. Remove the
				// stale cache file and fall through so a fresh authentication
				// flow can proceed without the invalid cache interfering.
				slog.Warn("cached auth token was rejected, removing cache file and re-authenticating", "path", options.CachePath)
				if rmErr := os.Remove(options.CachePath); rmErr != nil && !os.IsNotExist(rmErr) {
					slog.Warn("failed to remove invalid credential cache", "path", options.CachePath, "error", rmErr)
				}
			}
		} else if !os.IsNotExist(err) {
			slog.Warn("failed to read credential cache", "path", options.CachePath, "error", err)
		}
	}

	// Create Channels
	tokenCh := make(chan Token, 1)
	errCh := make(chan error, 1)

	// Use Browser OAuth Flow if Explicitly Enabled
	if os.Getenv("TAVERN_USE_BROWSER_OAUTH") == "1" {
		// Create Listener
		conn, err := net.Listen("tcp", ":0")
		if err != nil {
			return Token(""), fmt.Errorf("failed to start access_token http redirect handler: %w", err)
		}
		defer conn.Close()

		// Build Access Token Endpoint
		accessTokenRedirURL, err := url.Parse(tavernURL)
		if err != nil {
			return Token(""), fmt.Errorf("%w: %v", ErrInvalidURL, err)
		}
		_, redirPort, err := net.SplitHostPort(conn.Addr().String())
		if err != nil {
			return Token(""), fmt.Errorf("%w: %q: %v", ErrInvalidURL, conn.Addr().String(), err)
		}
		accessTokenRedirURL.RawQuery = url.Values{
			auth.ParamTokenRedirPort: []string{redirPort},
		}.Encode()
		accessTokenRedirURL.Path = "/access_token/redirect"

		// Log TLS Warning
		if accessTokenRedirURL.Scheme == "http" {
			slog.WarnContext(ctx, "using insecure access token url (http), this may leak sensitive information")
		}

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

		// Try to open the browser
		if browser != nil {
			err = browser.OpenURL(accessTokenRedirURL.String())
		} else {
			err = fmt.Errorf("no browser provided")
		}

		// Fallback to Remote Device Authentication if browser fails
		if err != nil {
			slog.WarnContext(ctx, "failed to open browser, falling back to remote device authentication", "error", err)
			go AuthenticateRemoteDevice(ctx, tavernURL, tokenCh, errCh)
		}
	} else {
		// Default to Remote Device Authentication
		go AuthenticateRemoteDevice(ctx, tavernURL, tokenCh, errCh)
	}

	// Wait for Token, Error, or Cancellation
	select {
	case <-ctx.Done():
		return Token(""), fmt.Errorf("authentication cancelled: %w", ctx.Err())
	case err := <-errCh:
		return Token(""), fmt.Errorf("failed to obtain credentials: %w", err)
	case token := <-tokenCh:
		if options.CachePath != "" {
			err := os.WriteFile(options.CachePath, []byte(token), 0600)
			if err != nil {
				slog.Warn("failed to write credential cache", "path", options.CachePath, "error", err)
			}
		}
		return token, nil
	}
}

// validateToken verifies that the provided token is still accepted by the tavern
// server at tavernURL. It returns (true, nil) if the server accepts the token,
// (false, nil) if the server explicitly rejects it as unauthorized, and an error
// if the request could not be completed (e.g. network failure, invalid URL).
//
// Validation is performed by issuing a minimal GraphQL request against the
// tavern `/graphql` endpoint, which already requires authentication. Relying on
// an endpoint that is inherently authenticated avoids turning an otherwise
// unauthenticated endpoint (such as `/status`) into a dedicated token
// validity oracle.
//
// Callers should retain the cached credentials when an error is returned so that
// transient failures do not evict otherwise-valid credentials from the cache.
func validateToken(ctx context.Context, tavernURL string, token Token) (bool, error) {
	u, err := url.Parse(tavernURL)
	if err != nil {
		return false, fmt.Errorf("%w: %v", ErrInvalidURL, err)
	}
	u.Path = "/graphql"
	u.RawQuery = ""

	reqCtx, cancel := context.WithTimeout(ctx, 10*time.Second)
	defer cancel()
	// Minimal authenticated GraphQL query. The `me` query requires an
	// authenticated user and is used purely to probe whether the cached
	// token is accepted by the server's auth middleware.
	body := strings.NewReader(`{"query":"{ me { id } }"}`)
	req, err := http.NewRequestWithContext(reqCtx, http.MethodPost, u.String(), body)
	if err != nil {
		return false, err
	}
	req.Header.Set("Content-Type", "application/json")
	token.Authenticate(req)

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return false, err
	}
	defer resp.Body.Close()
	// Drain the body so the underlying connection can be reused.
	_, _ = io.Copy(io.Discard, resp.Body)

	return resp.StatusCode != http.StatusUnauthorized, nil
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
		token := params.Get(auth.ParamTokenRedirToken)
		if token == "" {
			http.Error(w, fmt.Sprintf("must set '%q' query param", auth.ParamTokenRedirToken), http.StatusBadRequest)
			errCh <- fmt.Errorf("must provide '%q' query parameter", auth.ParamTokenRedirToken)
			return
		}

		tokenCh <- Token(token)
		io.WriteString(w, "Successfully Authenticated!")
	})
}
