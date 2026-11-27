package oauthproxy

import (
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

func TestSignAndVerifyState(t *testing.T) {
	pub, priv := mustKeyPair(t)

	returnTo := "https://my-tenant-tavern-123.us-central1.run.app/oauth/complete"
	nonce := "random-nonce-value"

	token, err := SignState(priv, returnTo, "client-nonce", nonce, time.Minute)
	if err != nil {
		t.Fatalf("SignState returned error: %v", err)
	}

	got, gotClient, err := VerifyState(pub, token, nonce)
	if err != nil {
		t.Fatalf("VerifyState returned error: %v", err)
	}
	if gotClient != "client-nonce" {
		t.Errorf("VerifyState client nonce = %q, want client-nonce", gotClient)
	}
	if got != returnTo {
		t.Errorf("VerifyState return_to = %q, want %q", got, returnTo)
	}
}

func TestVerifyStateRejectsNonceMismatch(t *testing.T) {
	pub, priv := mustKeyPair(t)

	token, err := SignState(priv, "https://tenant.run.app/oauth/complete", "client-nonce", "nonce-a", time.Minute)
	if err != nil {
		t.Fatalf("SignState returned error: %v", err)
	}

	if _, _, err := VerifyState(pub, token, "nonce-b"); err == nil {
		t.Fatal("expected error verifying state with mismatched nonce, got nil")
	}
}

func TestVerifyStateRejectsExpired(t *testing.T) {
	pub, priv := mustKeyPair(t)

	token, err := SignState(priv, "https://tenant.run.app/oauth/complete", "client-nonce", "nonce-a", -time.Minute)
	if err != nil {
		t.Fatalf("SignState returned error: %v", err)
	}

	if _, _, err := VerifyState(pub, token, "nonce-a"); err == nil {
		t.Fatal("expected error verifying expired state, got nil")
	}
}

func TestSignStateRequiresReturnTo(t *testing.T) {
	_, priv := mustKeyPair(t)

	if _, err := SignState(priv, "", "client-nonce", "nonce-a", time.Minute); err == nil {
		t.Fatal("expected error signing state with empty return_to, got nil")
	}
}

func TestSignStateRequiresClientNonce(t *testing.T) {
	_, priv := mustKeyPair(t)

	if _, err := SignState(priv, "https://tenant.run.app/oauth/complete", "", "nonce-a", time.Minute); err == nil {
		t.Fatal("expected error signing state with empty client nonce, got nil")
	}
}

func TestVerifyStateRejectsAssertion(t *testing.T) {
	pub, priv := mustKeyPair(t)

	// An assertion signed by the same key must never verify as a state token.
	assertion, err := SignAssertion(priv, Profile{OAuthID: "subject-123"}, testAudience, "nonce-1", time.Minute)
	if err != nil {
		t.Fatalf("SignAssertion returned error: %v", err)
	}
	if _, _, err := VerifyState(pub, assertion, "nonce-1"); err == nil {
		t.Fatal("expected error verifying an assertion as a state token, got nil")
	}
}

func TestVerifyStateRejectsFutureIssuedAt(t *testing.T) {
	pub, priv := mustKeyPair(t)

	now := time.Now()
	claims := StateClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Issuer:    stateIssuer,
			ID:        "nonce-a",
			IssuedAt:  jwt.NewNumericDate(now.Add(5 * time.Minute)),
			ExpiresAt: jwt.NewNumericDate(now.Add(10 * time.Minute)),
		},
		ReturnTo:    "https://tenant.run.app/oauth/complete",
		ClientNonce: "client-nonce",
	}
	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	token.Header["typ"] = stateTyp
	signed, err := token.SignedString(priv)
	if err != nil {
		t.Fatalf("failed to sign token: %v", err)
	}
	if _, _, err := VerifyState(pub, signed, "nonce-a"); err == nil {
		t.Fatal("expected error verifying state with future iat, got nil")
	}
}

func TestVerifyStateRejectsWrongAlg(t *testing.T) {
	pub, _ := mustKeyPair(t)

	claims := StateClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Issuer:    stateIssuer,
			ID:        "nonce-a",
			IssuedAt:  jwt.NewNumericDate(time.Now()),
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(time.Minute)),
		},
		ReturnTo:    "https://tenant.run.app/oauth/complete",
		ClientNonce: "client-nonce",
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	token.Header["typ"] = stateTyp
	signed, err := token.SignedString([]byte("secret"))
	if err != nil {
		t.Fatalf("failed to sign token: %v", err)
	}
	if _, _, err := VerifyState(pub, signed, "nonce-a"); err == nil {
		t.Fatal("expected error verifying HS256 state token, got nil")
	}
}
