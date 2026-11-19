package oauthproxy

import (
	"testing"
	"time"
)

func TestSignAndVerifyState(t *testing.T) {
	pub, priv := mustKeyPair(t)

	returnTo := "https://my-tenant-tavern-123.us-central1.run.app/oauth/complete"
	nonce := "random-nonce-value"

	token, err := SignState(priv, returnTo, nonce, time.Minute)
	if err != nil {
		t.Fatalf("SignState returned error: %v", err)
	}

	got, err := VerifyState(pub, token, nonce)
	if err != nil {
		t.Fatalf("VerifyState returned error: %v", err)
	}
	if got != returnTo {
		t.Errorf("VerifyState return_to = %q, want %q", got, returnTo)
	}
}

func TestVerifyStateRejectsNonceMismatch(t *testing.T) {
	pub, priv := mustKeyPair(t)

	token, err := SignState(priv, "https://tenant.run.app/oauth/complete", "nonce-a", time.Minute)
	if err != nil {
		t.Fatalf("SignState returned error: %v", err)
	}

	if _, err := VerifyState(pub, token, "nonce-b"); err == nil {
		t.Fatal("expected error verifying state with mismatched nonce, got nil")
	}
}

func TestVerifyStateRejectsExpired(t *testing.T) {
	pub, priv := mustKeyPair(t)

	token, err := SignState(priv, "https://tenant.run.app/oauth/complete", "nonce-a", -time.Minute)
	if err != nil {
		t.Fatalf("SignState returned error: %v", err)
	}

	if _, err := VerifyState(pub, token, "nonce-a"); err == nil {
		t.Fatal("expected error verifying expired state, got nil")
	}
}

func TestSignStateRequiresReturnTo(t *testing.T) {
	_, priv := mustKeyPair(t)

	if _, err := SignState(priv, "", "nonce-a", time.Minute); err == nil {
		t.Fatal("expected error signing state with empty return_to, got nil")
	}
}
