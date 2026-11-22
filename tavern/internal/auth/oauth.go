package auth

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"golang.org/x/oauth2"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/user"
	"realm.pub/tavern/internal/logging"
)

var oauthLog = logging.For("auth.oauth")

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
	ErrOAuthFailedUserCreation        = fmt.Errorf("failed to create user account")
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

		oauthLog.InfoContext(req.Context(), "oauth login flow started")
	})
}

// NewOAuthAuthorizationHandler returns an http endpoint that validates the request was
// redirected from the identity provider after a consent flow and initializes a user session
func NewOAuthAuthorizationHandler(cfg oauth2.Config, pubKey ed25519.PublicKey, graph *ent.Client, profileURL string) http.Handler {
	var registerMu sync.Mutex

	return http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		ctx := req.Context()

		// Determine state presented in redirect
		presentedState := req.FormValue("state")
		if presentedState == "" {
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth authorization missing state", "reason", "missing_state")
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
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth authorization missing state cookie", "reason", "missing_cookie")
			http.Error(w, ErrOAuthNoCookieFound.Error(), http.StatusBadRequest)
			return
		}

		// Verify JWT in provided OAuth cookie
		var claims jwt.RegisteredClaims
		stateToken, err := jwt.ParseWithClaims(oauthCookie.Value, &claims, func(stateToken *jwt.Token) (interface{}, error) {
			// Ensure ed25519 was used
			if _, ok := stateToken.Method.(*jwt.SigningMethodEd25519); !ok {
				return nil, fmt.Errorf("unexpected signing method: %v", stateToken.Header["alg"])
			}

			return pubKey, nil
		})
		if err != nil || !stateToken.Valid {
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth authorization invalid state cookie", "reason", "invalid_cookie")
			http.Error(w, ErrOAuthInvalidCookie.Error(), http.StatusBadRequest)
			return
		}

		// Ensure presented OAuth state matches expected (stored in JWT)
		if presentedState != claims.ID {
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth authorization state mismatch", "reason", "invalid_state")
			http.Error(w, ErrOAuthInvalidState.Error(), http.StatusUnauthorized)
			return
		}

		// Clear one-time OAuth state cookie after verification to prevent replay
		http.SetCookie(w, clearOAuthStateCookie())

		// Exchange the authorization code for an access token from the IDP
		code := req.FormValue("code")
		accessToken, err := cfg.Exchange(ctx, code)
		if err != nil {
			oauthLog.ErrorContext(ctx, "oauth failed to exchange auth code for an access token", "error", err)
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth token exchange failed", "reason", "exchange_failed")
			http.Error(w, ErrOAuthExchangeFailed.Error(), http.StatusInternalServerError)
			return
		}

		// Obtain profile information from identity provider
		client := cfg.Client(ctx, accessToken)
		resp, err := client.Get(profileURL)
		if err != nil {
			oauthLog.ErrorContext(ctx, "oauth failed to obtain profile info", "error", err)
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth profile fetch failed", "reason", "profile_fetch_failed")
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
			oauthLog.ErrorContext(ctx, "oauth failed to parse profile info", "error", err)
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth profile parse failed", "reason", "profile_parse_failed")
			http.Error(w, ErrOAuthFailedToParseProfileInfo.Error(), http.StatusInternalServerError)
			return
		}
		if profile.OAuthID == "" {
			oauthLog.ErrorContext(ctx, "oauth profile info was missing oauth id")
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth profile missing subject id", "reason", "missing_oauth_id")
			http.Error(w, ErrOAuthInvalidProfileInfo.Error(), http.StatusInternalServerError)
			return
		}

		// Lookup the user
		usr, err := graph.User.Query().
			Where(user.OauthID(profile.OAuthID)).
			Only(ctx)
		if err == nil {
			completeOAuthLogin(w, req, usr, false)
			return
		}

		// If we encountered a DB error, fail
		if !ent.IsNotFound(err) {
			oauthLog.ErrorContext(ctx, "oauth failed to lookup user profile", "error", err)
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth user lookup failed", "reason", "user_lookup_failed")
			http.Error(w, ErrOAuthFailedUserLookup.Error(), http.StatusInternalServerError)
			return
		}

		// If the user is not yet registered, create the user atomically in a transaction.
		// Tavern uses a TOFU model: if this is the first user they are the admin.
		usr, isNewUser, err := registerOrLookupOAuthUser(ctx, &registerMu, graph, profile.Name, profile.OAuthID, profile.PhotoURL)
		if err != nil {
			oauthLog.ErrorContext(ctx, "oauth failed to create user profile", "error", err)
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth user creation failed", "reason", "user_creation_failed")
			http.Error(w, ErrOAuthFailedUserCreation.Error(), http.StatusInternalServerError)
			return
		}

		completeOAuthLogin(w, req, usr, isNewUser)
	})
}

func registerOrLookupOAuthUser(
	ctx context.Context,
	mu *sync.Mutex,
	graph *ent.Client,
	name, oauthID, photoURL string,
) (*ent.User, bool, error) {
	mu.Lock()
	defer mu.Unlock()

	tx, err := graph.Tx(ctx)
	if err != nil {
		return nil, false, fmt.Errorf("begin tx: %w", err)
	}
	committed := false
	defer func() {
		if !committed {
			_ = tx.Rollback()
		}
	}()

	// Re-check inside the transaction in case a concurrent login already registered this user
	existing, err := tx.User.Query().
		Where(user.OauthID(oauthID)).
		Only(ctx)
	if err == nil {
		return existing, false, nil
	}
	if !ent.IsNotFound(err) {
		return nil, false, fmt.Errorf("lookup user in tx: %w", err)
	}

	userCount, err := tx.User.Query().Count(ctx)
	if err != nil {
		return nil, false, fmt.Errorf("count users: %w", err)
	}
	isTOFU := userCount == 0

	usr, err := tx.User.Create().
		SetName(name).
		SetOauthID(oauthID).
		SetPhotoURL(photoURL).
		SetIsAdmin(isTOFU).
		SetIsActivated(isTOFU).
		Save(ctx)
	if err != nil {
		if ent.IsConstraintError(err) {
			_ = tx.Rollback()
			committed = true
			existing, lookupErr := graph.User.Query().
				Where(user.OauthID(oauthID)).
				Only(ctx)
			if lookupErr == nil {
				return existing, false, nil
			}
		}
		return nil, false, fmt.Errorf("save user: %w", err)
	}

	if err := tx.Commit(); err != nil {
		return nil, false, fmt.Errorf("commit tx: %w", err)
	}
	committed = true
	return usr, true, nil
}

func completeOAuthLogin(w http.ResponseWriter, req *http.Request, usr *ent.User, isNewUser bool) {
	http.SetCookie(w, newSessionCookie(req, usr.SessionToken))
	authCtx := logging.WithActor(req.Context(), logging.Actor{
		Kind:       logging.ActorUser,
		ID:         strconv.Itoa(usr.ID),
		Name:       usr.Name,
		IsAdmin:    usr.IsAdmin,
		AuthMethod: "oauth",
	})
	msg := "oauth user logged in"
	if isNewUser {
		msg = "oauth registered new user"
	}
	oauthLog.InfoContext(authCtx, msg,
		"user_id", usr.ID,
		"user_name", usr.Name,
		"is_admin", usr.IsAdmin,
		"is_activated", usr.IsActivated,
	)
	logging.Audit(authCtx, "auth", "oauth.login_succeeded", msg,
		"new_user", isNewUser,
		"is_admin", usr.IsAdmin,
		"is_activated", usr.IsActivated,
		"user_id", usr.ID,
		"user_name", usr.Name,
	)
	http.Redirect(w, req, "/", http.StatusFound)
}

func newSessionCookie(req *http.Request, sessionToken string) *http.Cookie {
	return &http.Cookie{
		Name:     SessionCookieName,
		Value:    sessionToken,
		Path:     "/",
		HttpOnly: true,
		Secure:   isSecureRequest(req),
		SameSite: http.SameSiteLaxMode,
		Expires:  time.Now().AddDate(0, 1, 0),
	}
}

func isSecureRequest(req *http.Request) bool {
	if req == nil {
		return false
	}
	if req.TLS != nil {
		return true
	}
	if req.URL != nil && strings.EqualFold(req.URL.Scheme, "https") {
		return true
	}
	if proto := req.Header.Get("X-Forwarded-Proto"); proto != "" {
		first := strings.TrimSpace(strings.Split(proto, ",")[0])
		if strings.EqualFold(first, "https") {
			return true
		}
	}
	return false
}

func clearOAuthStateCookie() *http.Cookie {
	return &http.Cookie{
		Name:     OAuthCookieName,
		Value:    "",
		Path:     "/",
		Secure:   true,
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
		MaxAge:   -1,
		Expires:  time.Unix(0, 0),
	}
}

func newOAuthStateCookie(privKey ed25519.PrivateKey, state string) *http.Cookie {
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

	return &http.Cookie{
		Name:     OAuthCookieName,
		Value:    tokenStr,
		Path:     "/",
		Secure:   true,
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
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
