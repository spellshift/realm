package main

import (
	"crypto/ed25519"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"time"

	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/oauthproxy"
)

var proxyLog = logging.For("oauthproxy")

// stateCookieName holds the random nonce bound into the signed "state" sent to Google, so the
// callback handler can detect a state token that was never actually issued by this proxy for
// this browser (defense in depth alongside the state token's own signature/expiry).
const stateCookieName = "oauth-proxy-state"

// maxClientNonceLen bounds the tenant-supplied nonce echoed into the state and assertion tokens.
const maxClientNonceLen = 128

// maxProfileBytes bounds how much of an upstream userinfo response is read.
const maxProfileBytes = 64 << 10

// newStartHandler begins a login: validates the tenant's requested return_to against the
// allow-list, then redirects to Google's consent screen using the proxy's single, shared OAuth
// client and its one fixed redirect_uri.
func newStartHandler(cfg *Config, privKey ed25519.PrivateKey) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		returnTo := r.URL.Query().Get("return_to")
		if returnTo == "" {
			http.Error(w, "missing return_to parameter", http.StatusBadRequest)
			return
		}
		if !oauthproxy.IsAllowedReturnTo(returnTo, cfg.AllowedSuffixes) {
			proxyLog.WarnContext(ctx, "rejected disallowed return_to", "return_to", returnTo)
			http.Error(w, "return_to is not an allowed destination", http.StatusBadRequest)
			return
		}

		// The tenant's own login nonce (set as a cookie on the browser at the tenant); it is
		// carried through state into the assertion so the tenant can tie the assertion to the
		// browser that started the login.
		clientNonce := r.URL.Query().Get("nonce")
		if clientNonce == "" || len(clientNonce) > maxClientNonceLen {
			http.Error(w, "missing or invalid nonce parameter", http.StatusBadRequest)
			return
		}

		nonce, err := newNonce()
		if err != nil {
			proxyLog.ErrorContext(ctx, "failed to generate nonce", "error", err)
			http.Error(w, "failed to start oauth flow", http.StatusInternalServerError)
			return
		}

		state, err := oauthproxy.SignState(privKey, returnTo, clientNonce, nonce, stateTTL)
		if err != nil {
			proxyLog.ErrorContext(ctx, "failed to sign state", "error", err)
			http.Error(w, "failed to start oauth flow", http.StatusInternalServerError)
			return
		}

		http.SetCookie(w, &http.Cookie{
			Name:     stateCookieName,
			Value:    nonce,
			Path:     "/",
			Secure:   cfg.secureCookies(),
			HttpOnly: true,
			SameSite: http.SameSiteLaxMode,
			MaxAge:   int(stateTTL.Seconds()),
		})

		proxyLog.InfoContext(ctx, "oauth proxy flow started", "return_to", returnTo)
		http.Redirect(w, r, cfg.OAuth.AuthCodeURL(state), http.StatusFound)
	})
}

// newCallbackHandler is the proxy's single, centrally-registered Google OAuth redirect URI. It
// exchanges the authorization code using the proxy's own credentials, fetches the verified
// profile, mints a short-lived signed assertion of it, and redirects the browser back to the
// tenant's return_to with that assertion attached.
func newCallbackHandler(cfg *Config, privKey ed25519.PrivateKey) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		state := r.FormValue("state")
		if state == "" {
			http.Error(w, "missing state parameter", http.StatusBadRequest)
			return
		}

		cookie, err := r.Cookie(stateCookieName)
		if err != nil {
			proxyLog.WarnContext(ctx, "oauth callback missing state cookie")
			http.Error(w, "missing oauth state cookie", http.StatusBadRequest)
			return
		}

		returnTo, clientNonce, err := oauthproxy.VerifyState(privKey.Public().(ed25519.PublicKey), state, cookie.Value)
		if err != nil {
			proxyLog.WarnContext(ctx, "rejected invalid oauth state", "error", err)
			http.Error(w, "invalid oauth state", http.StatusBadRequest)
			return
		}
		// Consume the state cookie only after verification succeeds so a stray or forged
		// cross-site request cannot wipe a user's in-flight login cookie.
		clearStateCookie(w, cfg.secureCookies())

		// Re-check the allow-list here too: this is the boundary before we attach a real
		// identity assertion to a redirect, and must not rely solely on the earlier /oauth/start
		// check (e.g. if the allow-list was narrowed between the two requests).
		if !oauthproxy.IsAllowedReturnTo(returnTo, cfg.AllowedSuffixes) {
			proxyLog.WarnContext(ctx, "rejected disallowed return_to on callback", "return_to", returnTo)
			http.Error(w, "return_to is not an allowed destination", http.StatusBadRequest)
			return
		}

		code := r.FormValue("code")
		if code == "" {
			http.Error(w, "missing code parameter", http.StatusBadRequest)
			return
		}

		token, err := cfg.OAuth.Exchange(ctx, code)
		if err != nil {
			proxyLog.ErrorContext(ctx, "failed to exchange authorization code", "error", err)
			http.Error(w, "failed to complete oauth exchange", http.StatusInternalServerError)
			return
		}

		client := cfg.OAuth.Client(ctx, token)
		resp, err := client.Get(cfg.ProfileURL)
		if err != nil {
			proxyLog.ErrorContext(ctx, "failed to fetch profile", "error", err)
			http.Error(w, "failed to obtain profile information", http.StatusInternalServerError)
			return
		}
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			proxyLog.ErrorContext(ctx, "profile endpoint returned non-200 status", "status_code", resp.StatusCode)
			http.Error(w, "failed to obtain profile information", http.StatusInternalServerError)
			return
		}

		var profile struct {
			OAuthID       string `json:"sub"`
			PhotoURL      string `json:"picture"`
			Email         string `json:"email"`
			Name          string `json:"name"`
			EmailVerified bool   `json:"email_verified"`
		}
		if err := json.NewDecoder(io.LimitReader(resp.Body, maxProfileBytes)).Decode(&profile); err != nil {
			proxyLog.ErrorContext(ctx, "failed to parse profile", "error", err)
			http.Error(w, "failed to parse profile information", http.StatusInternalServerError)
			return
		}
		if profile.OAuthID == "" {
			proxyLog.ErrorContext(ctx, "profile info missing subject id")
			http.Error(w, "profile information was missing a subject id", http.StatusInternalServerError)
			return
		}

		assertion, err := oauthproxy.SignAssertion(privKey, oauthproxy.Profile{
			OAuthID:       profile.OAuthID,
			Email:         profile.Email,
			Name:          profile.Name,
			PhotoURL:      profile.PhotoURL,
			EmailVerified: profile.EmailVerified,
		}, returnTo, clientNonce, assertionTTL)
		if err != nil {
			proxyLog.ErrorContext(ctx, "failed to sign assertion", "error", err)
			http.Error(w, "failed to complete oauth flow", http.StatusInternalServerError)
			return
		}

		// returnTo has no query or fragment (enforced by IsAllowedReturnTo), but build the URL
		// structurally rather than by string concatenation anyway.
		dest, err := url.Parse(returnTo)
		if err != nil {
			proxyLog.ErrorContext(ctx, "failed to parse return_to", "error", err)
			http.Error(w, "failed to complete oauth flow", http.StatusInternalServerError)
			return
		}
		q := dest.Query()
		q.Set("assertion", assertion)
		dest.RawQuery = q.Encode()
		redirectURL := dest.String()
		proxyLog.InfoContext(ctx, "oauth proxy flow completed", "return_to", returnTo)
		http.Redirect(w, r, redirectURL, http.StatusFound)
	})
}

// newJWKSHandler serves the proxy's current public key so tenants can verify assertions without
// any manual key distribution.
func newJWKSHandler(pubKey ed25519.PublicKey) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		data, err := oauthproxy.EncodeJWKS(pubKey)
		if err != nil {
			proxyLog.ErrorContext(r.Context(), "failed to encode jwks", "error", err)
			http.Error(w, "failed to encode jwks", http.StatusInternalServerError)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(data)
	})
}

func newStatusHandler() http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	})
}

func clearStateCookie(w http.ResponseWriter, secure bool) {
	http.SetCookie(w, &http.Cookie{
		Name:     stateCookieName,
		Value:    "",
		Path:     "/",
		Secure:   secure,
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
		MaxAge:   -1,
		Expires:  time.Unix(0, 0),
	})
}

func newNonce() (string, error) {
	buf := make([]byte, 32)
	if _, err := io.ReadFull(rand.Reader, buf); err != nil {
		return "", fmt.Errorf("failed to generate nonce: %w", err)
	}
	return base64.RawURLEncoding.EncodeToString(buf), nil
}
