package auth

import (
	"crypto/ed25519"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"golang.org/x/oauth2"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/user"
)

const (
	OAuthCookieName   = "oauth-state"
	SessionCookieName = "auth-session"
)

var (
	ErrOAuthNoStatePresented          = fmt.Errorf("no OAuth state presented")
	ErrOAuthNoCookieFound             = fmt.Errorf("no OAuth cookie found")
	ErrOAuthInvalidCookie             = fmt.Errorf("invalid OAuth cookie provided")
	ErrOAuthInvalidState              = fmt.Errorf("presented OAuth state is invalid")
	ErrOAuthExchangeFailed            = fmt.Errorf("failed to exchange authorization code for an access token from identity provider")
	ErrOAuthFailedToObtainProfileInfo = fmt.Errorf("failed to obtain profile information from identity provider")
	ErrOAuthFailedToParseProfileInfo  = fmt.Errorf("failed to parse profile information returned by identity provider")
	ErrOAuthInvalidProfileInfo        = fmt.Errorf("failed to parse profile information returned by identity provider")
	ErrOAuthFailedUserLookup          = fmt.Errorf("failed to lookup user account")
)

type oauthStateClaims struct {
	RedirectURI string `json:"redirect_uri,omitempty"`
	ClientState string `json:"client_state,omitempty"`
	jwt.RegisteredClaims
}

// NewOAuthLoginHandler returns an http endpoint that redirects the user to the configured OAuth consent flow
// It will set a JWT in a cookie that will later be used to verify the OAuth state
func NewOAuthLoginHandler(cfg oauth2.Config, privKey ed25519.PrivateKey) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		clientRedirectURI := req.URL.Query().Get("redirect_uri")
		clientState := req.URL.Query().Get("state")

		// Create a signed OAuth state so callback verification can be stateless.
		state := newOAuthStateToken(privKey, clientRedirectURI, clientState)

		// Generate OAuth URL based on this state
		url := cfg.AuthCodeURL(state)

		// Redirect to identity provider
		http.Redirect(w, req, url, http.StatusFound)

		slog.Info("oauth: starting flow for new login")
	})
}

// NewOAuthAuthorizationHandler returns an http endpoint that validates the request was
// redirected from the identity provider after a consent flow and initializes a user session
func NewOAuthAuthorizationHandler(cfg oauth2.Config, pubKey ed25519.PublicKey, graph *ent.Client, profileURL string) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		// Determine state presented in redirect
		presentedState := req.FormValue("state")
		if presentedState == "" {
			http.Error(w, ErrOAuthNoStatePresented.Error(), http.StatusBadRequest)
			return
		}

		stateClaims, stateErr := parseOAuthStateToken(presentedState, pubKey)
		if stateErr != nil {
			// Fallback for legacy cookie-based state validation.
			oauthCookie, err := req.Cookie(stateCookieName(presentedState))
			if err == http.ErrNoCookie {
				// Backward compatibility with legacy single-cookie state storage.
				oauthCookie, err = req.Cookie(OAuthCookieName)
			}
			if err != nil || oauthCookie == nil {
				http.Error(w, ErrOAuthNoCookieFound.Error(), http.StatusBadRequest)
				return
			}

			stateClaims, err = parseOAuthStateToken(oauthCookie.Value, pubKey)
			if err != nil {
				http.Error(w, ErrOAuthInvalidCookie.Error(), http.StatusBadRequest)
				return
			}

			// Ensure presented OAuth state matches expected (stored in JWT)
			if presentedState != stateClaims.ID {
				http.Error(w, ErrOAuthInvalidState.Error(), http.StatusUnauthorized)
				return
			}
		}

		// Exchange the authorization code for an access token from the IDP
		code := req.FormValue("code")
		accessToken, err := cfg.Exchange(req.Context(), code)
		if err != nil {
			slog.ErrorContext(req.Context(), "oauth failed to exchange auth code for an access token", "err", err)
			http.Error(w, ErrOAuthExchangeFailed.Error(), http.StatusInternalServerError)
			return
		}

		// Obtain profile information from identity provider
		client := cfg.Client(req.Context(), accessToken)
		resp, err := client.Get(profileURL)
		if err != nil {
			slog.ErrorContext(req.Context(), "oauth failed to obtain profile info", "err", err)
			http.Error(w, ErrOAuthFailedToObtainProfileInfo.Error(), http.StatusInternalServerError)
			return
		}
		defer resp.Body.Close()

		// Parse Profile Info
		var profile struct {
			OAuthID    string `json:"sub"`
			PhotoURL   string `json:"picture"`
			Email      string `json:"email"`
			Name       string `json:"name"`
			IsVerified bool   `json:"email_verified"`
		}
		decoder := json.NewDecoder(resp.Body)
		if err := decoder.Decode(&profile); err != nil {
			slog.ErrorContext(req.Context(), "oauth failed to parse profile info", "err", err)
			http.Error(w, ErrOAuthFailedToParseProfileInfo.Error(), http.StatusInternalServerError)
			return
		}
		if profile.OAuthID == "" {
			slog.ErrorContext(req.Context(), "oauth profile info was missing oauth id")
			http.Error(w, ErrOAuthInvalidProfileInfo.Error(), http.StatusInternalServerError)
			return
		}

		// Lookup the user
		usr, err := graph.User.Query().
			Where(user.OauthID(profile.OAuthID)).
			Only(req.Context())
		if err == nil {
			http.SetCookie(w, &http.Cookie{
				Name:     SessionCookieName,
				Value:    usr.SessionToken,
				Path:     "/",
				HttpOnly: true,
				Expires:  time.Now().AddDate(0, 1, 0),
			})
			slog.InfoContext(req.Context(), "oauth new login", "user_id", usr.ID, "user_name", usr.Name, "is_admin", usr.IsAdmin, "is_activated", usr.IsActivated)
			http.Redirect(w, req, oauthPostLoginRedirect(usr.AccessToken, stateClaims), http.StatusFound)
			return
		}

		// If we encountered a DB error, fail
		if !ent.IsNotFound(err) {
			slog.ErrorContext(req.Context(), "oauth failed to lookup user profile", "err", err)
			http.Error(w, ErrOAuthFailedUserLookup.Error(), http.StatusInternalServerError)
			return
		}

		// If the user is not yet registered, create the user
		// Tavern uses a TOFU model, if this is the first user they are the admin
		isTOFU := graph.User.Query().CountX(req.Context()) == 0
		usr = graph.User.Create().
			SetName(profile.Name).
			SetOauthID(profile.OAuthID).
			SetPhotoURL(profile.PhotoURL).
			SetIsAdmin(isTOFU).
			SetIsActivated(isTOFU).
			SaveX(req.Context())

		http.SetCookie(w, &http.Cookie{
			Name:     SessionCookieName,
			Value:    usr.SessionToken,
			Path:     "/",
			HttpOnly: true,
			Expires:  time.Now().AddDate(0, 1, 0),
		})
		slog.InfoContext(req.Context(), "oauth registered new user %q", "user_id", usr.ID, "user_name", usr.Name, "is_admin", usr.IsAdmin, "is_activated", usr.IsActivated)
		http.Redirect(w, req, oauthPostLoginRedirect(usr.AccessToken, stateClaims), http.StatusFound)
	})
}

func newOAuthStateCookie(req *http.Request, privKey ed25519.PrivateKey, state string) *http.Cookie {
	expiresAt := time.Now().Add(10 * time.Minute)
	claims := jwt.RegisteredClaims{
		ID:        state,
		IssuedAt:  jwt.NewNumericDate(time.Now()),
		ExpiresAt: jwt.NewNumericDate(expiresAt),
	}
	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	tokenStr, err := token.SignedString(privKey)
	if err != nil {
		panic(fmt.Errorf("failed to create oauth-state JWT: %w", err))
	}

	secure := req.TLS != nil || strings.EqualFold(req.Header.Get("X-Forwarded-Proto"), "https")

	return &http.Cookie{
		Name:     stateCookieName(state),
		Value:    tokenStr,
		Secure:   secure,
		HttpOnly: true,
		Path:     "/",
		SameSite: http.SameSiteLaxMode,
		Expires:  expiresAt,
	}
}

func parseOAuthStateToken(tokenStr string, pubKey ed25519.PublicKey) (*oauthStateClaims, error) {
	var claims oauthStateClaims
	stateToken, err := jwt.ParseWithClaims(tokenStr, &claims, func(stateToken *jwt.Token) (interface{}, error) {
		if _, ok := stateToken.Method.(*jwt.SigningMethodEd25519); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", stateToken.Header["alg"])
		}
		return pubKey, nil
	})
	if err != nil || !stateToken.Valid {
		return nil, ErrOAuthInvalidState
	}
	return &claims, nil
}

func oauthPostLoginRedirect(accessToken string, claims *oauthStateClaims) string {
	if claims == nil || claims.RedirectURI == "" {
		return "/"
	}

	redirectURI, err := url.Parse(claims.RedirectURI)
	if err != nil || !redirectURI.IsAbs() {
		return "/"
	}

	query := redirectURI.Query()
	if claims.ClientState != "" {
		query.Set("state", claims.ClientState)
	}
	query.Set("code", accessToken)
	redirectURI.RawQuery = query.Encode()

	return redirectURI.String()
}

func stateCookieName(state string) string {
	return OAuthCookieName + "-" + state
}

func newOAuthState() string {
	buf := make([]byte, 32)
	_, err := io.ReadFull(rand.Reader, buf)
	if err != nil {
		panic(fmt.Errorf("failed to generate oauth state: %w", err))
	}
	// Use URL-safe base64 to avoid intermediary rewriting of '+' and '/' characters.
	return base64.RawURLEncoding.EncodeToString(buf)
}

func newOAuthStateToken(privKey ed25519.PrivateKey, clientRedirectURI, clientState string) string {
	expiresAt := time.Now().Add(10 * time.Minute)
	claims := oauthStateClaims{
		RedirectURI: clientRedirectURI,
		ClientState: clientState,
		RegisteredClaims: jwt.RegisteredClaims{
			ID:        newOAuthState(),
			IssuedAt:  jwt.NewNumericDate(time.Now()),
			ExpiresAt: jwt.NewNumericDate(expiresAt),
		},
	}
	stateToken := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	stateTokenStr, err := stateToken.SignedString(privKey)
	if err != nil {
		panic(fmt.Errorf("failed to create oauth-state JWT: %w", err))
	}

	return stateTokenStr
}
