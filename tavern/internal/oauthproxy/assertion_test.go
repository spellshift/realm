package oauthproxy

import (
	"crypto/ed25519"
	"testing"
	"time"
)

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

	token, err := SignAssertion(priv, profile, time.Minute)
	if err != nil {
		t.Fatalf("SignAssertion returned error: %v", err)
	}

	claims, err := VerifyAssertion(pub, token)
	if err != nil {
		t.Fatalf("VerifyAssertion returned error: %v", err)
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

	if _, err := SignAssertion(priv, Profile{Email: "user@example.com"}, time.Minute); err == nil {
		t.Fatal("expected error signing assertion with empty OAuthID, got nil")
	}
}

func TestVerifyAssertionRejectsExpired(t *testing.T) {
	pub, priv := mustKeyPair(t)

	token, err := SignAssertion(priv, Profile{OAuthID: "subject-123"}, -time.Minute)
	if err != nil {
		t.Fatalf("SignAssertion returned error: %v", err)
	}

	if _, err := VerifyAssertion(pub, token); err == nil {
		t.Fatal("expected error verifying expired assertion, got nil")
	}
}

func TestVerifyAssertionRejectsWrongKey(t *testing.T) {
	_, priv := mustKeyPair(t)
	otherPub, _ := mustKeyPair(t)

	token, err := SignAssertion(priv, Profile{OAuthID: "subject-123"}, time.Minute)
	if err != nil {
		t.Fatalf("SignAssertion returned error: %v", err)
	}

	if _, err := VerifyAssertion(otherPub, token); err == nil {
		t.Fatal("expected error verifying assertion with the wrong public key, got nil")
	}
}

func TestVerifyAssertionRejectsTamperedToken(t *testing.T) {
	pub, priv := mustKeyPair(t)

	token, err := SignAssertion(priv, Profile{OAuthID: "subject-123"}, time.Minute)
	if err != nil {
		t.Fatalf("SignAssertion returned error: %v", err)
	}

	tampered := token + "x"
	if _, err := VerifyAssertion(pub, tampered); err == nil {
		t.Fatal("expected error verifying tampered assertion, got nil")
	}
}
