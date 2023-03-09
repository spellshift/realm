package auth

import (
	"crypto/ed25519"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"time"

	"github.com/golang-jwt/jwt"
	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/ent/user"
	"golang.org/x/oauth2"
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

// NewOAuthLoginHandler returns an http endpoint that redirects the user to the configured OAuth consent flow
// It will set a JWT in a cookie that will later be used to verify the OAuth state
func NewOAuthLoginHandler(cfg oauth2.Config, privKey ed25519.PrivateKey) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		// Create a new random string to prevent against CSRF attacks
		state := newOAuthState()

		// Generate OAuth URL based on this state
		url := cfg.AuthCodeURL(state)

		// Set a JWT to verify the state after consent flow redirect
		http.SetCookie(w, newOAuthStateCookie(privKey, state))

		// Redirect to identity provider
		http.Redirect(w, req, url, http.StatusFound)

		log.Printf("[INFO] [OAuth] starting flow for new login")
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

		// Lookup OAuth cookie for verification
		var oauthCookie *http.Cookie
		for _, cookie := range req.Cookies() {
			if cookie.Name == OAuthCookieName {
				oauthCookie = cookie
				break
			}
		}
		if oauthCookie == nil {
			http.Error(w, ErrOAuthNoCookieFound.Error(), http.StatusBadRequest)
			return
		}

		// Verify JWT in provided OAuth cookie
		var claims jwt.StandardClaims
		stateToken, err := jwt.ParseWithClaims(oauthCookie.Value, &claims, func(stateToken *jwt.Token) (interface{}, error) {
			// Ensure ed25519 was used
			if _, ok := stateToken.Method.(*jwt.SigningMethodEd25519); !ok {
				return nil, fmt.Errorf("unexpected signing method: %v", stateToken.Header["alg"])
			}

			return pubKey, nil
		})
		if err != nil || !stateToken.Valid {
			http.Error(w, ErrOAuthInvalidCookie.Error(), http.StatusBadRequest)
			return
		}

		// Ensure presented OAuth state matches expected (stored in JWT)
		if presentedState != claims.Id {
			http.Error(w, ErrOAuthInvalidState.Error(), http.StatusUnauthorized)
			return
		}

		// Exchange the authorization code for an access token from the IDP
		code := req.FormValue("code")
		accessToken, err := cfg.Exchange(req.Context(), code)
		if err != nil {
			log.Printf("[ERROR] [OAuth] failed to exchange auth code for an access token: %v", err)
			http.Error(w, ErrOAuthExchangeFailed.Error(), http.StatusInternalServerError)
			return
		}

		// Obtain profile information from identity provider
		client := cfg.Client(req.Context(), accessToken)
		resp, err := client.Get(profileURL)
		if err != nil {
			log.Printf("[ERROR] [OAuth] failed to obtain profile info: %v", err)
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
			log.Printf("[ERROR] [OAuth] failed to parse profile info: %v", err)
			http.Error(w, ErrOAuthFailedToParseProfileInfo.Error(), http.StatusInternalServerError)
			return
		}
		if profile.OAuthID == "" {
			log.Printf("[ERROR] [OAuth] profile info was missing OAuthID")
			http.Error(w, ErrOAuthInvalidProfileInfo.Error(), http.StatusInternalServerError)
			return
		}

		// Lookup the user
		usr, err := graph.User.Query().
			Where(user.OAuthID(profile.OAuthID)).
			Only(req.Context())
		if err == nil {
			http.SetCookie(w, &http.Cookie{
				Name:     SessionCookieName,
				Value:    usr.SessionToken,
				Path:     "/",
				HttpOnly: true,
				Expires:  time.Now().AddDate(0, 1, 0),
			})
			log.Printf("[INFO] [OAuth] new login for user %q", usr.Name)
			return
		}

		// If we encountered a DB error, fail
		if !ent.IsNotFound(err) {
			log.Printf("[ERROR] [OAuth] failed to lookup user profile: %v", err)
			http.Error(w, ErrOAuthFailedUserLookup.Error(), http.StatusInternalServerError)
			return
		}

		// If the user is not yet registered, create the user
		// Tavern uses a TOFU model, if this is the first user they are the admin
		isTOFU := graph.User.Query().CountX(req.Context()) == 0
		usr = graph.User.Create().
			SetName(profile.Name).
			SetOAuthID(profile.OAuthID).
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
		log.Printf("[INFO] [OAuth] registered new user %q", usr.Name)
		http.Redirect(w, req, "/", http.StatusFound)
	})
}

func newOAuthStateCookie(privKey ed25519.PrivateKey, state string) *http.Cookie {
	expiresAt := time.Now().Add(10 * time.Minute)
	claims := jwt.StandardClaims{
		Id:        state,
		IssuedAt:  time.Now().Unix(),
		ExpiresAt: expiresAt.Unix(),
	}
	token := jwt.NewWithClaims(&jwt.SigningMethodEd25519{}, claims)
	tokenStr, err := token.SignedString(privKey)
	if err != nil {
		panic(fmt.Errorf("failed to create oauth-state JWT: %w", err))
	}

	return &http.Cookie{
		Name:     OAuthCookieName,
		Value:    tokenStr,
		Secure:   true,
		HttpOnly: true,
		Expires:  expiresAt,
	}
}

func newOAuthState() string {
	buf := make([]byte, 32)
	_, err := io.ReadFull(rand.Reader, buf)
	if err != nil {
		panic(fmt.Errorf("failed to generate oauth state: %w", err))
	}
	return base64.StdEncoding.EncodeToString(buf)
}
