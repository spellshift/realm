package auth

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"os"
	"strings"
	"time"

	"realm.pub/tavern/cli/qrcode"
)

const (
	defaultRDAInterval = 5 * time.Second
	defaultRDAExpires  = 10 * time.Minute
)

type rdaPollResult struct {
	token    Token
	terminal bool
	err      error
}

// AuthenticateRemoteDevice begins the remote device authentication flow by polling
// the provided tavern URL until the device gets approved or rejected.
func AuthenticateRemoteDevice(ctx context.Context, tavernURL string, tokenCh chan<- Token, errCh chan<- error) {
	baseURL := strings.TrimRight(tavernURL, "/")

	// 1. Request Code
	codeReq, err := http.NewRequestWithContext(ctx, http.MethodPost, baseURL+"/auth/rda/code", nil)
	if err != nil {
		errCh <- fmt.Errorf("failed to build code request: %w", err)
		return
	}
	codeResp, err := http.DefaultClient.Do(codeReq)
	if err != nil {
		errCh <- fmt.Errorf("failed to request device code: %w", err)
		return
	}
	defer codeResp.Body.Close()

	if codeResp.StatusCode != http.StatusOK {
		b, _ := io.ReadAll(codeResp.Body)
		errCh <- fmt.Errorf("failed to request device code: %s: %s", codeResp.Status, b)
		return
	}

	var codeRes struct {
		DeviceCode              string `json:"device_code"`
		UserCode                string `json:"user_code"`
		VerificationURI         string `json:"verification_uri"`
		VerificationURIComplete string `json:"verification_uri_complete"`
		ExpiresIn               int    `json:"expires_in"`
		Interval                int    `json:"interval"`
	}
	if err := json.NewDecoder(codeResp.Body).Decode(&codeRes); err != nil {
		errCh <- fmt.Errorf("failed to decode device code response: %w", err)
		return
	}

	fmt.Fprintf(os.Stderr, "\n\nOpen %s on any device and enter the following code:\n\n\t%s\n\nWaiting for approval...\n\n", codeRes.VerificationURI, codeRes.UserCode)
	if codeRes.VerificationURIComplete != "" {
		if err := qrcode.FprintQRCode(os.Stderr, codeRes.VerificationURIComplete); err != nil {
			slog.WarnContext(ctx, "failed to print QR code", "error", err)
		}
	}

	// 2. Poll for Token
	interval := time.Duration(codeRes.Interval) * time.Second
	if interval < time.Second {
		interval = defaultRDAInterval
	}

	expiresIn := time.Duration(codeRes.ExpiresIn) * time.Second
	if expiresIn <= 0 {
		expiresIn = defaultRDAExpires
	}

	tokenPayload, err := json.Marshal(struct {
		DeviceCode string `json:"device_code"`
	}{
		DeviceCode: codeRes.DeviceCode,
	})
	if err != nil {
		errCh <- fmt.Errorf("failed to encode device code request: %w", err)
		return
	}
	tokenURL := baseURL + "/auth/rda/token"

	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	timeout := time.NewTimer(expiresIn)
	defer timeout.Stop()

	for {
		select {
		case <-ctx.Done():
			errCh <- ctx.Err()
			return
		case <-timeout.C:
			errCh <- fmt.Errorf("remote device authentication timed out")
			return
		case <-ticker.C:
			res := pollRDAToken(ctx, tokenURL, tokenPayload)
			if res.token != "" {
				tokenCh <- res.token
				return
			}
			if res.terminal {
				errCh <- res.err
				return
			}
		}
	}
}

func pollRDAToken(ctx context.Context, tokenURL string, payload []byte) rdaPollResult {
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, tokenURL, bytes.NewReader(payload))
	if err != nil {
		return rdaPollResult{terminal: true, err: fmt.Errorf("failed to build token poll request: %w", err)}
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return rdaPollResult{} // transient connection error, retry
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(io.LimitReader(resp.Body, 1<<16))
	var tokenRes struct {
		Status      string `json:"status"`
		AccessToken string `json:"access_token"`
		Error       string `json:"error"`
	}
	_ = json.Unmarshal(body, &tokenRes)

	if resp.StatusCode == http.StatusOK {
		if tokenRes.AccessToken != "" {
			return rdaPollResult{token: Token(tokenRes.AccessToken)}
		}
		if strings.EqualFold(tokenRes.Status, "DENIED") {
			return rdaPollResult{terminal: true, err: fmt.Errorf("authorization failed: access_denied")}
		}
		return rdaPollResult{}
	}

	// RFC 8628-style pending responses over 400 should keep polling.
	if resp.StatusCode == http.StatusBadRequest && (tokenRes.Error == "authorization_pending" || tokenRes.Error == "slow_down") {
		return rdaPollResult{}
	}

	// Terminal client/auth errors (denied, expired, consumed, invalid code).
	if resp.StatusCode == http.StatusForbidden ||
		resp.StatusCode == http.StatusUnauthorized ||
		resp.StatusCode == http.StatusGone ||
		resp.StatusCode == http.StatusNotFound ||
		resp.StatusCode == http.StatusBadRequest {
		reason := tokenRes.Error
		if reason == "" {
			reason = strings.TrimSpace(string(body))
		}
		if reason == "" {
			reason = resp.Status
		}
		return rdaPollResult{terminal: true, err: fmt.Errorf("authorization failed: %s", reason)}
	}

	// Transient 5xx or unexpected status: keep retrying until timeout.
	return rdaPollResult{}
}
