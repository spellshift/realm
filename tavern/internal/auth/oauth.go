package auth

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/subtle"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strconv"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"golang.org/x/oauth2"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/user"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/oauthproxy"
)

var oauthLog = logging.For("auth.oauth")

const (
	OAuthCookieName   = "oauth-state"
	SessionCookieName = "auth-session"

	// OAuthProxyNonceCookieName binds a login started via the oauth proxy to the browser that
	// started it (see NewProxyLoginHandler / NewProxyAssertionHandler).
	OAuthProxyNonceCookieName = "oauth-proxy-nonce"

	oauthProxyNonceTTL = 10 * time.Minute
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

		oauthLog.InfoContext(req.Context(), "oauth login flow started")
	})
}

// NewOAuthAuthorizationHandler returns an http endpoint that validates the request was
// redirected from the identity provider after a consent flow and initializes a user session
func NewOAuthAuthorizationHandler(cfg oauth2.Config, pubKey ed25519.PublicKey, graph *ent.Client, profileURL string) http.Handler {
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
		var profile oauthProfile
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

		completeOAuthLogin(ctx, w, req, graph, profile)
	})
}

// oauthProfile is the verified identity information used to look up or create a Tavern user,
// regardless of whether it was obtained directly from the identity provider
// (NewOAuthAuthorizationHandler) or relayed via a signed assertion from a centralized OAuth proxy
// (NewProxyAssertionHandler).
type oauthProfile struct {
	OAuthID    string `json:"sub"`
	PhotoURL   string `json:"picture"`
	Email      string `json:"email"`
	Name       string `json:"name"`
	IsVerified bool   `json:"email_verified"`
}

// completeOAuthLogin looks up (or, on first login, creates) the Tavern user matching profile,
// sets their session cookie, and redirects to the app. Tavern uses a TOFU model: if this is the
// very first user, they become the admin.
func completeOAuthLogin(ctx context.Context, w http.ResponseWriter, req *http.Request, graph *ent.Client, profile oauthProfile) {
	usr, err := graph.User.Query().
		Where(user.OauthID(profile.OAuthID)).
		Only(ctx)
	if err == nil {
		http.SetCookie(w, &http.Cookie{
			Name:     SessionCookieName,
			Value:    usr.SessionToken,
			Path:     "/",
			HttpOnly: true,
			Expires:  time.Now().AddDate(0, 1, 0),
		})
		authCtx := logging.WithActor(ctx, logging.Actor{
			Kind:       logging.ActorUser,
			ID:         strconv.Itoa(usr.ID),
			Name:       usr.Name,
			IsAdmin:    usr.IsAdmin,
			AuthMethod: "oauth",
		})
		oauthLog.InfoContext(authCtx, "oauth user logged in",
			"user_id", usr.ID,
			"user_name", usr.Name,
			"is_admin", usr.IsAdmin,
			"is_activated", usr.IsActivated,
		)
		logging.Audit(authCtx, "auth", "oauth.login_succeeded", "oauth user logged in",
			"new_user", false,
			"is_admin", usr.IsAdmin,
			"is_activated", usr.IsActivated,
			"user_id", usr.ID,
			"user_name", usr.Name,
		)
		http.Redirect(w, req, "/", http.StatusFound)
		return
	}

	// If we encountered a DB error, fail
	if !ent.IsNotFound(err) {
		oauthLog.ErrorContext(ctx, "oauth failed to lookup user profile", "error", err)
		logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth user lookup failed", "reason", "user_lookup_failed")
		http.Error(w, ErrOAuthFailedUserLookup.Error(), http.StatusInternalServerError)
		return
	}

	// If the user is not yet registered, create the user
	// Tavern uses a TOFU model, if this is the first user they are the admin
	isTOFU := graph.User.Query().CountX(ctx) == 0
	usr = graph.User.Create().
		SetName(profile.Name).
		SetOauthID(profile.OAuthID).
		SetPhotoURL(profile.PhotoURL).
		SetIsAdmin(isTOFU).
		SetIsActivated(isTOFU).
		SaveX(ctx)

	http.SetCookie(w, &http.Cookie{
		Name:     SessionCookieName,
		Value:    usr.SessionToken,
		Path:     "/",
		HttpOnly: true,
		Expires:  time.Now().AddDate(0, 1, 0),
	})
	authCtx := logging.WithActor(ctx, logging.Actor{
		Kind:       logging.ActorUser,
		ID:         strconv.Itoa(usr.ID),
		Name:       usr.Name,
		IsAdmin:    usr.IsAdmin,
		AuthMethod: "oauth",
	})
	oauthLog.InfoContext(authCtx, "oauth registered new user",
		"user_id", usr.ID,
		"user_name", usr.Name,
		"is_admin", usr.IsAdmin,
		"is_activated", usr.IsActivated,
	)
	logging.Audit(authCtx, "auth", "oauth.login_succeeded", "oauth registered new user",
		"new_user", true,
		"is_admin", usr.IsAdmin,
		"is_activated", usr.IsActivated,
		"user_id", usr.ID,
		"user_name", usr.Name,
	)
	http.Redirect(w, req, "/", http.StatusFound)
}

// NewProxyLoginHandler returns an http endpoint that redirects the user to the centralized OAuth
// proxy's consent flow, passing this Tavern instance's own /oauth/complete endpoint as the
// return_to destination. Used instead of NewOAuthLoginHandler when Tavern is configured with
// OAUTH_PROXY_URL (see ConfigureOAuthFromEnv in tavern/config.go) so that this tenant's domain
// never needs to be registered as a redirect URI with Google directly.
func NewProxyLoginHandler(proxyURL, returnTo string) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		// Bind this login to this browser: the nonce is set as a cookie here, round-tripped
		// through the proxy inside the signed assertion, and checked in NewProxyAssertionHandler.
		nonceBytes := make([]byte, 32)
		if _, err := rand.Read(nonceBytes); err != nil {
			oauthLog.ErrorContext(req.Context(), "oauth proxy failed to generate login nonce", "error", err)
			http.Error(w, "failed to start oauth login", http.StatusInternalServerError)
			return
		}
		nonce := base64.RawURLEncoding.EncodeToString(nonceBytes)
		http.SetCookie(w, &http.Cookie{
			Name:     OAuthProxyNonceCookieName,
			Value:    nonce,
			Path:     "/",
			Secure:   strings.HasPrefix(returnTo, "https://"),
			HttpOnly: true,
			SameSite: http.SameSiteLaxMode, // sent on the top-level redirect back from the proxy
			MaxAge:   int(oauthProxyNonceTTL.Seconds()),
		})

		startURL := fmt.Sprintf("%s/oauth/start?return_to=%s&nonce=%s", strings.TrimSuffix(proxyURL, "/"), url.QueryEscape(returnTo), url.QueryEscape(nonce))
		oauthLog.InfoContext(req.Context(), "oauth proxy login flow started", "proxy_url", proxyURL)
		http.Redirect(w, req, startURL, http.StatusFound)
	})
}

// NewProxyAssertionHandler returns an http endpoint that verifies a signed identity assertion
// relayed from the centralized OAuth proxy (after it completed the actual Google consent flow
// and code exchange on this tenant's behalf), and then completes login exactly as
// NewOAuthAuthorizationHandler does for a direct OAuth flow.
//
// returnTo is this Tavern's own /oauth/complete URL; assertions are only accepted if the proxy
// issued them for exactly that audience, and only for the browser that started the login.
func NewProxyAssertionHandler(keySource *oauthproxy.KeySource, graph *ent.Client, returnTo string) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		ctx := req.Context()

		nonceCookie, err := req.Cookie(OAuthProxyNonceCookieName)
		if err != nil || nonceCookie.Value == "" {
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth proxy callback missing login nonce cookie", "reason", "missing_nonce_cookie")
			http.Error(w, "missing oauth login cookie", http.StatusBadRequest)
			return
		}

		assertion := req.FormValue("assertion")
		if assertion == "" {
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth proxy callback missing assertion", "reason", "missing_assertion")
			http.Error(w, "missing assertion parameter", http.StatusBadRequest)
			return
		}

		pubKey, err := keySource.PublicKey(ctx)
		if err != nil {
			oauthLog.ErrorContext(ctx, "oauth proxy failed to obtain jwks public key", "error", err)
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth proxy jwks fetch failed", "reason", "jwks_fetch_failed")
			http.Error(w, "failed to verify oauth proxy assertion", http.StatusInternalServerError)
			return
		}

		claims, err := oauthproxy.VerifyAssertion(pubKey, assertion, returnTo)
		if err != nil {
			oauthLog.WarnContext(ctx, "oauth proxy presented invalid assertion", "error", err)
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth proxy assertion invalid", "reason", "invalid_assertion")
			http.Error(w, "invalid oauth proxy assertion", http.StatusUnauthorized)
			return
		}

		if subtle.ConstantTimeCompare([]byte(claims.Nonce), []byte(nonceCookie.Value)) != 1 {
			oauthLog.WarnContext(ctx, "oauth proxy assertion nonce did not match login cookie")
			logging.AuditFailure(ctx, "auth", "oauth.login_failed", "oauth proxy assertion nonce mismatch", "reason", "nonce_mismatch")
			http.Error(w, "invalid oauth proxy assertion", http.StatusUnauthorized)
			return
		}

		// Consume the login nonce only after verifying the assertion and binding, so a stray or
		// forged cross-site request cannot wipe a user's in-flight login cookie.
		http.SetCookie(w, &http.Cookie{
			Name:     OAuthProxyNonceCookieName,
			Value:    "",
			Path:     "/",
			Secure:   strings.HasPrefix(returnTo, "https://"),
			HttpOnly: true,
			SameSite: http.SameSiteLaxMode,
			MaxAge:   -1,
			Expires:  time.Unix(0, 0),
		})

		completeOAuthLogin(ctx, w, req, graph, oauthProfile{
			OAuthID:    claims.OAuthID,
			PhotoURL:   claims.PhotoURL,
			Email:      claims.Email,
			Name:       claims.Name,
			IsVerified: claims.EmailVerified,
		})
	})
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
