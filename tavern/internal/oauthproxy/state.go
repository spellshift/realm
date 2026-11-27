package oauthproxy

import (
	"crypto/ed25519"
	"crypto/subtle"
	"fmt"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

// stateIssuer identifies the "state" tokens the proxy sends to Google (and gets echoed back
// verbatim), binding the flow to the tenant return URL that initiated it.
const stateIssuer = "realm-oauth-proxy/state"

// stateTyp is the JWT "typ" header on state tokens, checked on verification so an assertion
// signed by the same key is never accepted as a state token.
const stateTyp = "realm-oauth-proxy-state"

// StateClaims binds the OAuth "state" parameter sent to Google to the tenant return URL that
// initiated the login, so the proxy can recover it (and be sure it has not been tampered with)
// when Google redirects back to the proxy's single, centrally-registered callback URL.
type StateClaims struct {
	jwt.RegisteredClaims
	ReturnTo string `json:"return_to"`
	// ClientNonce is the tenant-chosen login nonce, carried through to the assertion.
	ClientNonce string `json:"client_nonce"`
}

// SignState signs a short-lived state token binding returnTo and the tenant's clientNonce to a
// random nonce. The nonce is also set as a short-lived cookie on the browser by the caller, so
// VerifyState can detect a state token that was never actually issued by this proxy instance for
// this browser.
func SignState(privKey ed25519.PrivateKey, returnTo, clientNonce, nonce string, ttl time.Duration) (string, error) {
	if returnTo == "" {
		return "", fmt.Errorf("oauthproxy: cannot sign state with empty return_to")
	}
	if clientNonce == "" {
		return "", fmt.Errorf("oauthproxy: cannot sign state with empty client nonce")
	}

	now := time.Now()
	claims := StateClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Issuer:    stateIssuer,
			ID:        nonce,
			IssuedAt:  jwt.NewNumericDate(now),
			ExpiresAt: jwt.NewNumericDate(now.Add(ttl)),
		},
		ReturnTo:    returnTo,
		ClientNonce: clientNonce,
	}

	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	token.Header["typ"] = stateTyp
	signed, err := token.SignedString(privKey)
	if err != nil {
		return "", fmt.Errorf("oauthproxy: failed to sign state: %w", err)
	}
	return signed, nil
}

// VerifyState verifies the signature and expiry of a state token, and that its nonce matches
// expectedNonce (the value from the caller's CSRF cookie). It returns the embedded return-to URL
// and the tenant's client nonce.
func VerifyState(pubKey ed25519.PublicKey, tokenStr, expectedNonce string) (returnTo, clientNonce string, err error) {
	var claims StateClaims
	token, err := jwt.ParseWithClaims(tokenStr, &claims, func(t *jwt.Token) (interface{}, error) {
		if _, ok := t.Method.(*jwt.SigningMethodEd25519); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", t.Header["alg"])
		}
		if typ, _ := t.Header["typ"].(string); typ != stateTyp {
			return nil, fmt.Errorf("unexpected token type: %v", t.Header["typ"])
		}
		return pubKey, nil
	},
		jwt.WithValidMethods([]string{jwt.SigningMethodEdDSA.Alg()}),
		jwt.WithIssuer(stateIssuer),
		jwt.WithIssuedAt(),
		jwt.WithExpirationRequired(),
	)
	if err != nil {
		return "", "", fmt.Errorf("oauthproxy: invalid state: %w", err)
	}
	if !token.Valid {
		return "", "", fmt.Errorf("oauthproxy: state token is not valid")
	}
	if expectedNonce == "" || subtle.ConstantTimeCompare([]byte(claims.ID), []byte(expectedNonce)) != 1 {
		return "", "", fmt.Errorf("oauthproxy: state nonce mismatch")
	}
	if claims.ReturnTo == "" || claims.ClientNonce == "" {
		return "", "", fmt.Errorf("oauthproxy: state missing return_to or client nonce")
	}

	return claims.ReturnTo, claims.ClientNonce, nil
}
