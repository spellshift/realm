// Package oauthproxy contains logic shared between the standalone oauthproxy Cloud Run service
// (./oauthproxy) and Tavern's consumption of it (tavern/internal/auth). It implements the
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
const assertionIssuer = "realm-oauth-proxy"

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
	OAuthID       string `json:"oauth_id"`
	Email         string `json:"email"`
	Name          string `json:"name"`
	PhotoURL      string `json:"photo_url"`
	EmailVerified bool   `json:"email_verified"`
}

// SignAssertion signs a short-lived assertion of profile using the proxy's private key, for a
// tenant to verify and consume at its /oauth/complete endpoint.
func SignAssertion(privKey ed25519.PrivateKey, profile Profile, ttl time.Duration) (string, error) {
	if profile.OAuthID == "" {
		return "", fmt.Errorf("oauthproxy: cannot sign assertion for profile with empty OAuthID")
	}

	now := time.Now()
	claims := ProfileAssertionClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Issuer:    assertionIssuer,
			Subject:   profile.OAuthID,
			IssuedAt:  jwt.NewNumericDate(now),
			ExpiresAt: jwt.NewNumericDate(now.Add(ttl)),
		},
		OAuthID:       profile.OAuthID,
		Email:         profile.Email,
		Name:          profile.Name,
		PhotoURL:      profile.PhotoURL,
		EmailVerified: profile.EmailVerified,
	}

	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	signed, err := token.SignedString(privKey)
	if err != nil {
		return "", fmt.Errorf("oauthproxy: failed to sign assertion: %w", err)
	}
	return signed, nil
}

// VerifyAssertion verifies the signature and expiry of an assertion token minted by SignAssertion,
// returning the embedded profile. Callers (tenants) must have obtained pubKey from a trusted
// source, such as the proxy's JWKS endpoint (see KeySource).
func VerifyAssertion(pubKey ed25519.PublicKey, tokenStr string) (*ProfileAssertionClaims, error) {
	var claims ProfileAssertionClaims
	token, err := jwt.ParseWithClaims(tokenStr, &claims, func(t *jwt.Token) (interface{}, error) {
		if _, ok := t.Method.(*jwt.SigningMethodEd25519); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", t.Header["alg"])
		}
		return pubKey, nil
	})
	if err != nil {
		return nil, fmt.Errorf("oauthproxy: invalid assertion: %w", err)
	}
	if !token.Valid {
		return nil, fmt.Errorf("oauthproxy: assertion token is not valid")
	}
	if claims.OAuthID == "" {
		return nil, fmt.Errorf("oauthproxy: assertion missing oauth_id")
	}

	return &claims, nil
}
