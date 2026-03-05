package auth

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

// AuthenticateRemoteDevice begins the remote device authentication flow by polling
// the provided tavern URL until the device gets approved or rejected.
func AuthenticateRemoteDevice(ctx context.Context, tavernURL string, tokenCh chan<- Token, errCh chan<- error) {
	// 1. Request Code
	codeReq, err := http.NewRequestWithContext(ctx, http.MethodPost, tavernURL+"/auth/rda/code", nil)
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

	fmt.Printf("\n\nOpen %s on any device and enter the following code:\n\n\t%s\n\nWaiting for approval...\n\n", codeRes.VerificationURI, codeRes.UserCode)

	// 2. Poll for Token
	interval := time.Duration(codeRes.Interval) * time.Second
	if interval < time.Second {
		interval = 5 * time.Second
	}

	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	timeout := time.After(time.Duration(codeRes.ExpiresIn) * time.Second)

	for {
		select {
		case <-ctx.Done():
			errCh <- ctx.Err()
			return
		case <-timeout:
			errCh <- fmt.Errorf("remote device authentication timed out")
			return
		case <-ticker.C:
			// Poll token endpoint
			tokenURL := fmt.Sprintf("%s/auth/rda/token?device_code=%s", tavernURL, codeRes.DeviceCode)
			req, err := http.NewRequestWithContext(ctx, http.MethodGet, tokenURL, nil)
			if err != nil {
				continue // try again
			}

			resp, err := http.DefaultClient.Do(req)
			if err != nil {
				continue // connection error, try again
			}

			if resp.StatusCode == http.StatusOK {
				var tokenRes struct {
					AccessToken string `json:"access_token"`
				}
				if err := json.NewDecoder(resp.Body).Decode(&tokenRes); err == nil && tokenRes.AccessToken != "" {
					resp.Body.Close()
					tokenCh <- Token(tokenRes.AccessToken)
					return
				}
			} else if resp.StatusCode == http.StatusForbidden || resp.StatusCode == http.StatusUnauthorized {
				// Rejected or expired
				var errRes struct {
					Error string `json:"error"`
				}
				json.NewDecoder(resp.Body).Decode(&errRes)
				resp.Body.Close()
				errCh <- fmt.Errorf("authorization failed: %s", errRes.Error)
				return
			}
			resp.Body.Close()
			// If not OK and not Forbidden (e.g., 400 Bad Request which we return for authorization_pending), just continue
		}
	}
}
