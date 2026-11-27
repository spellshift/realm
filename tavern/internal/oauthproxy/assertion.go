// Package oauthproxy contains logic shared between the standalone oauthproxy Cloud Run service
// (./tavern/oauthproxy) and Tavern's consumption of it (tavern/internal/auth). It implements the
// signed "state" and "assertion" tokens that let a single, centrally-registered OAuth proxy
// front Google OAuth for many independently-deployed Tavern tenants, each with its own domain,
// without any of those domains ever being registered as a Google OAuth redirect URI.
package oauthproxy

import (
	"crypto/ed25519"
	"fmt"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

// assertionIssuer identifies tokens minted by the oauth proxy after completing the Google consent
// flow on a tenant's behalf.
const assertionIssuer = "realm-oauth-proxy/assertion"

// assertionTyp is the JWT "typ" header on assertions. It is checked on verification so a token
// of another kind signed by the same proxy key (e.g. a state token) is never accepted as one.
const assertionTyp = "realm-oauth-proxy-assertion"

// Profile is the verified identity information the proxy obtained from the upstream IDP (Google)
// on behalf of a tenant Tavern instance.
type Profile struct {
	OAuthID       string
	Email         string
	Name          string
	PhotoURL      string
	EmailVerified bool
}

// ProfileAssertionClaims is the signed JWT the proxy hands back to a tenant's /oauth/complete
// endpoint, attesting that Profile was verified by the upstream IDP. It is intentionally
// short-lived (see SignAssertion) since it is only meant to survive a single browser redirect.
type ProfileAssertionClaims struct {
	jwt.RegisteredClaims
	// Nonce echoes the tenant-chosen nonce from /oauth/start; the tenant checks it against the
	// cookie it set on the browser that began the login (login CSRF protection).
	Nonce         string `json:"nonce"`
	OAuthID       string `json:"oauth_id"`
	Email         string `json:"email"`
	Name          string `json:"name"`
	PhotoURL      string `json:"photo_url"`
	EmailVerified bool   `json:"email_verified"`
}

// SignAssertion signs a short-lived assertion of profile using the proxy's private key, for a
// tenant to verify and consume at its /oauth/complete endpoint. audience must be the exact
// return_to the tenant asked for; VerifyAssertion rejects the assertion at any other tenant.
// nonce is the tenant-chosen login nonce, echoed back so the tenant can bind the assertion to
// the browser that started the login.
func SignAssertion(privKey ed25519.PrivateKey, profile Profile, audience, nonce string, ttl time.Duration) (string, error) {
	if profile.OAuthID == "" {
		return "", fmt.Errorf("oauthproxy: cannot sign assertion for profile with empty OAuthID")
	}
	if audience == "" {
		return "", fmt.Errorf("oauthproxy: cannot sign assertion with empty audience")
	}
	if nonce == "" {
		return "", fmt.Errorf("oauthproxy: cannot sign assertion with empty nonce")
	}

	now := time.Now()
	claims := ProfileAssertionClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Issuer:    assertionIssuer,
			Subject:   profile.OAuthID,
			Audience:  jwt.ClaimStrings{audience},
			IssuedAt:  jwt.NewNumericDate(now),
			ExpiresAt: jwt.NewNumericDate(now.Add(ttl)),
		},
		Nonce:         nonce,
		OAuthID:       profile.OAuthID,
		Email:         profile.Email,
		Name:          profile.Name,
		PhotoURL:      profile.PhotoURL,
		EmailVerified: profile.EmailVerified,
	}

	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	token.Header["typ"] = assertionTyp
	signed, err := token.SignedString(privKey)
	if err != nil {
		return "", fmt.Errorf("oauthproxy: failed to sign assertion: %w", err)
	}
	return signed, nil
}

// VerifyAssertion verifies the signature, issuer, type, expiry and audience of an assertion token
// minted by SignAssertion, returning the embedded profile. audience must be this tenant's own
// return_to URL. Callers (tenants) must have obtained pubKey from a trusted source, such as the
// proxy's JWKS endpoint (see KeySource), and must separately check claims.Nonce.
func VerifyAssertion(pubKey ed25519.PublicKey, tokenStr, audience string) (*ProfileAssertionClaims, error) {
	var claims ProfileAssertionClaims
	token, err := jwt.ParseWithClaims(tokenStr, &claims, func(t *jwt.Token) (interface{}, error) {
		if _, ok := t.Method.(*jwt.SigningMethodEd25519); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", t.Header["alg"])
		}
		if typ, _ := t.Header["typ"].(string); typ != assertionTyp {
			return nil, fmt.Errorf("unexpected token type: %v", t.Header["typ"])
		}
		return pubKey, nil
	},
		jwt.WithValidMethods([]string{jwt.SigningMethodEdDSA.Alg()}),
		jwt.WithIssuer(assertionIssuer),
		jwt.WithAudience(audience),
		jwt.WithIssuedAt(),
		jwt.WithExpirationRequired(),
	)
	if err != nil {
		return nil, fmt.Errorf("oauthproxy: invalid assertion: %w", err)
	}
	if !token.Valid {
		return nil, fmt.Errorf("oauthproxy: assertion token is not valid")
	}
	if claims.OAuthID == "" {
		return nil, fmt.Errorf("oauthproxy: assertion missing oauth_id")
	}
	if claims.Nonce == "" {
		return nil, fmt.Errorf("oauthproxy: assertion missing nonce")
	}

	return &claims, nil
}
