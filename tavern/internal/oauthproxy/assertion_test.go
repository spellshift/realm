package oauthproxy

import (
	"crypto/ed25519"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

const testAudience = "https://tenant-a.run.app/oauth/complete"

func mustKeyPair(t *testing.T) (ed25519.PublicKey, ed25519.PrivateKey) {
	t.Helper()
	pub, priv, err := ed25519.GenerateKey(nil)
	if err != nil {
		t.Fatalf("failed to generate key pair: %v", err)
	}
	return pub, priv
}

func TestSignAndVerifyAssertion(t *testing.T) {
	pub, priv := mustKeyPair(t)

	profile := Profile{
		OAuthID:       "subject-123",
		Email:         "user@example.com",
		Name:          "Test User",
		PhotoURL:      "https://example.com/photo.png",
		EmailVerified: true,
	}

	token, err := SignAssertion(priv, profile, testAudience, "nonce-1", time.Minute)
	if err != nil {
		t.Fatalf("SignAssertion returned error: %v", err)
	}

	claims, err := VerifyAssertion(pub, token, testAudience)
	if err != nil {
		t.Fatalf("VerifyAssertion returned error: %v", err)
	}

	if claims.Nonce != "nonce-1" {
		t.Errorf("Nonce = %q, want nonce-1", claims.Nonce)
	}
	if claims.OAuthID != profile.OAuthID {
		t.Errorf("OAuthID = %q, want %q", claims.OAuthID, profile.OAuthID)
	}
	if claims.Email != profile.Email {
		t.Errorf("Email = %q, want %q", claims.Email, profile.Email)
	}
	if claims.Name != profile.Name {
		t.Errorf("Name = %q, want %q", claims.Name, profile.Name)
	}
	if claims.PhotoURL != profile.PhotoURL {
		t.Errorf("PhotoURL = %q, want %q", claims.PhotoURL, profile.PhotoURL)
	}
	if claims.EmailVerified != profile.EmailVerified {
		t.Errorf("EmailVerified = %v, want %v", claims.EmailVerified, profile.EmailVerified)
	}
}

func TestSignAssertionRequiresOAuthID(t *testing.T) {
	_, priv := mustKeyPair(t)

	if _, err := SignAssertion(priv, Profile{Email: "user@example.com"}, testAudience, "nonce-1", time.Minute); err == nil {
		t.Fatal("expected error signing assertion with empty OAuthID, got nil")
	}
}

func TestVerifyAssertionRejectsExpired(t *testing.T) {
	pub, priv := mustKeyPair(t)

	token, err := SignAssertion(priv, Profile{OAuthID: "subject-123"}, testAudience, "nonce-1", -time.Minute)
	if err != nil {
		t.Fatalf("SignAssertion returned error: %v", err)
	}

	if _, err := VerifyAssertion(pub, token, testAudience); err == nil {
		t.Fatal("expected error verifying expired assertion, got nil")
	}
}

func TestVerifyAssertionRejectsWrongKey(t *testing.T) {
	_, priv := mustKeyPair(t)
	otherPub, _ := mustKeyPair(t)

	token, err := SignAssertion(priv, Profile{OAuthID: "subject-123"}, testAudience, "nonce-1", time.Minute)
	if err != nil {
		t.Fatalf("SignAssertion returned error: %v", err)
	}

	if _, err := VerifyAssertion(otherPub, token, testAudience); err == nil {
		t.Fatal("expected error verifying assertion with the wrong public key, got nil")
	}
}

func TestVerifyAssertionRejectsTamperedToken(t *testing.T) {
	pub, priv := mustKeyPair(t)

	token, err := SignAssertion(priv, Profile{OAuthID: "subject-123"}, testAudience, "nonce-1", time.Minute)
	if err != nil {
		t.Fatalf("SignAssertion returned error: %v", err)
	}

	tampered := token + "x"
	if _, err := VerifyAssertion(pub, tampered, testAudience); err == nil {
		t.Fatal("expected error verifying tampered assertion, got nil")
	}
}

func TestVerifyAssertionRejectsWrongAudience(t *testing.T) {
	pub, priv := mustKeyPair(t)

	token, err := SignAssertion(priv, Profile{OAuthID: "subject-123"}, testAudience, "nonce-1", time.Minute)
	if err != nil {
		t.Fatalf("SignAssertion returned error: %v", err)
	}

	if _, err := VerifyAssertion(pub, token, "https://tenant-b.run.app/oauth/complete"); err == nil {
		t.Fatal("expected error verifying assertion issued for a different tenant, got nil")
	}
}

func TestSignAssertionRequiresAudienceAndNonce(t *testing.T) {
	_, priv := mustKeyPair(t)

	if _, err := SignAssertion(priv, Profile{OAuthID: "s"}, "", "nonce-1", time.Minute); err == nil {
		t.Error("expected error signing assertion with empty audience")
	}
	if _, err := SignAssertion(priv, Profile{OAuthID: "s"}, testAudience, "", time.Minute); err == nil {
		t.Error("expected error signing assertion with empty nonce")
	}
}

func TestVerifyAssertionRejectsStateToken(t *testing.T) {
	pub, priv := mustKeyPair(t)

	// A state token signed by the same key must never verify as an assertion (token confusion).
	state, err := SignState(priv, testAudience, "client-nonce", "nonce", time.Minute)
	if err != nil {
		t.Fatalf("SignState returned error: %v", err)
	}
	if _, err := VerifyAssertion(pub, state, testAudience); err == nil {
		t.Fatal("expected error verifying a state token as an assertion, got nil")
	}
}

func TestVerifyAssertionRequiresTypHeader(t *testing.T) {
	pub, priv := mustKeyPair(t)

	claims := ProfileAssertionClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Issuer:    assertionIssuer,
			Audience:  jwt.ClaimStrings{testAudience},
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(time.Minute)),
		},
		Nonce:   "nonce-1",
		OAuthID: "subject-123",
	}
	signed, err := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims).SignedString(priv)
	if err != nil {
		t.Fatalf("failed to sign token: %v", err)
	}
	if _, err := VerifyAssertion(pub, signed, testAudience); err == nil {
		t.Fatal("expected error verifying assertion without typ header, got nil")
	}
}

func TestVerifyAssertionRequiresExpiry(t *testing.T) {
	pub, priv := mustKeyPair(t)

	claims := ProfileAssertionClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Issuer:   assertionIssuer,
			Audience: jwt.ClaimStrings{testAudience},
		},
		Nonce:   "nonce-1",
		OAuthID: "subject-123",
	}
	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	token.Header["typ"] = assertionTyp
	signed, err := token.SignedString(priv)
	if err != nil {
		t.Fatalf("failed to sign token: %v", err)
	}
	if _, err := VerifyAssertion(pub, signed, testAudience); err == nil {
		t.Fatal("expected error verifying assertion without exp, got nil")
	}
}

func TestVerifyAssertionRejectsFutureIssuedAt(t *testing.T) {
	pub, priv := mustKeyPair(t)

	now := time.Now()
	claims := ProfileAssertionClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Issuer:    assertionIssuer,
			Audience:  jwt.ClaimStrings{testAudience},
			IssuedAt:  jwt.NewNumericDate(now.Add(5 * time.Minute)),
			ExpiresAt: jwt.NewNumericDate(now.Add(10 * time.Minute)),
		},
		Nonce:   "nonce-1",
		OAuthID: "subject-123",
	}
	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	token.Header["typ"] = assertionTyp
	signed, err := token.SignedString(priv)
	if err != nil {
		t.Fatalf("failed to sign token: %v", err)
	}
	if _, err := VerifyAssertion(pub, signed, testAudience); err == nil {
		t.Fatal("expected error verifying assertion with future iat, got nil")
	}
}

func TestVerifyAssertionRejectsWrongAlg(t *testing.T) {
	pub, _ := mustKeyPair(t)

	claims := ProfileAssertionClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Issuer:    assertionIssuer,
			Audience:  jwt.ClaimStrings{testAudience},
			IssuedAt:  jwt.NewNumericDate(time.Now()),
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(time.Minute)),
		},
		Nonce:   "nonce-1",
		OAuthID: "subject-123",
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	token.Header["typ"] = assertionTyp
	signed, err := token.SignedString([]byte("secret"))
	if err != nil {
		t.Fatalf("failed to sign token: %v", err)
	}
	if _, err := VerifyAssertion(pub, signed, testAudience); err == nil {
		t.Fatal("expected error verifying HS256 assertion, got nil")
	}
}
