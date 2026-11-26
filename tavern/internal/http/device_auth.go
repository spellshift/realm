package http

import (
	"crypto/rand"
	"encoding/json"
	"fmt"
	"mime"
	"net"
	"net/http"
	"net/url"
	"strconv"
	"strings"
	"time"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/deviceauth"
	"realm.pub/tavern/internal/logging"
)

const (
	defaultRDAPollIntervalSeconds = 5
	headerDeviceCode              = "X-Device-Code"
)

var rdaLog = logging.For("auth.rda")

func generateUserCode() string {
	b := make([]byte, 4) // 8 hex chars
	rand.Read(b)
	return fmt.Sprintf("%x", b)
}

func generateDeviceCode() string {
	b := make([]byte, 16) // 32 hex chars
	rand.Read(b)
	return fmt.Sprintf("%x", b)
}

type RDACodeResponse struct {
	UserCode                string `json:"user_code"`
	DeviceCode              string `json:"device_code"`
	VerificationURI         string `json:"verification_uri"`
	VerificationURIComplete string `json:"verification_uri_complete"`
	ExpiresIn               int    `json:"expires_in"` // seconds
	Interval                int    `json:"interval"`   // seconds
}

func validateHost(rawHost string) (string, error) {
	if rawHost == "" {
		return "", fmt.Errorf("empty host")
	}
	for i := 0; i < len(rawHost); i++ {
		c := rawHost[i]
		if c <= 0x20 || c >= 0x7f || strings.ContainsRune(`/\?#@%`, rune(c)) {
			return "", fmt.Errorf("invalid character in host")
		}
	}
	u, err := url.Parse("http://" + rawHost)
	if err != nil || u.User != nil || u.Path != "" || u.RawQuery != "" || u.Fragment != "" || u.Host != rawHost {
		return "", fmt.Errorf("malformed host")
	}
	if strings.Contains(rawHost, ":") {
		if strings.HasPrefix(rawHost, "[") && strings.HasSuffix(rawHost, "]") {
			if net.ParseIP(rawHost[1:len(rawHost)-1]) == nil {
				return "", fmt.Errorf("invalid IPv6 host")
			}
			return rawHost, nil
		}
		host, portStr, err := net.SplitHostPort(rawHost)
		if err != nil || host == "" {
			return "", fmt.Errorf("invalid host:port")
		}
		port, err := strconv.Atoi(portStr)
		if err != nil || port < 1 || port > 65535 {
			return "", fmt.Errorf("invalid port")
		}
	}
	return rawHost, nil
}

func resolveVerificationBaseURL(r *http.Request, configuredBaseURL string) (*url.URL, error) {
	configuredBaseURL = strings.TrimSpace(configuredBaseURL)
	if configuredBaseURL != "" {
		if !strings.HasPrefix(configuredBaseURL, "http://") && !strings.HasPrefix(configuredBaseURL, "https://") {
			configuredBaseURL = "https://" + configuredBaseURL
		}
		u, err := url.Parse(configuredBaseURL)
		if err != nil || (u.Scheme != "http" && u.Scheme != "https") || u.User != nil {
			return nil, fmt.Errorf("invalid configured base URL")
		}
		host, err := validateHost(u.Host)
		if err != nil {
			return nil, err
		}
		return &url.URL{Scheme: u.Scheme, Host: host}, nil
	}

	host, err := validateHost(r.Host)
	if err != nil {
		return nil, err
	}

	scheme := "http"
	if r.TLS != nil {
		scheme = "https"
	} else if xfp := r.Header.Get("X-Forwarded-Proto"); xfp != "" {
		firstProto := strings.TrimSpace(strings.Split(xfp, ",")[0])
		if strings.EqualFold(firstProto, "https") {
			scheme = "https"
		}
	}

	return &url.URL{Scheme: scheme, Host: host}, nil
}

func validateCSRF(w http.ResponseWriter, r *http.Request) bool {
	mediaType, _, err := mime.ParseMediaType(r.Header.Get("Content-Type"))
	if err != nil || mediaType != "application/json" {
		http.Error(w, "unsupported media type: expected application/json", http.StatusUnsupportedMediaType)
		return false
	}

	if secFetchSite := strings.TrimSpace(r.Header.Get("Sec-Fetch-Site")); secFetchSite != "" {
		if secFetchSite != "same-origin" && secFetchSite != "none" {
			http.Error(w, "cross-site request forbidden", http.StatusForbidden)
			return false
		}
	}

	if origin := strings.TrimSpace(r.Header.Get("Origin")); origin != "" {
		originURL, err := url.Parse(origin)
		if err != nil || (originURL.Scheme != "http" && originURL.Scheme != "https") || originURL.Host == "" {
			http.Error(w, "invalid request origin", http.StatusForbidden)
			return false
		}
		if r.Host != "" && !strings.EqualFold(originURL.Host, r.Host) {
			http.Error(w, "invalid request origin", http.StatusForbidden)
			return false
		}
	}

	return true
}

func writeRDATokenJSON(w http.ResponseWriter, statusCode int, resp RDATokenResponse) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(statusCode)
	_ = json.NewEncoder(w).Encode(resp)
}

func NewRDACodeHandler(client *ent.Client, publicBaseURL ...string) http.HandlerFunc {
	var configuredBaseURL string
	if len(publicBaseURL) > 0 {
		configuredBaseURL = publicBaseURL[0]
	}

	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		// Only allow POST
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		baseURL, err := resolveVerificationBaseURL(r, configuredBaseURL)
		if err != nil {
			rdaLog.WarnContext(ctx, "invalid host for device auth verification URI", "error", err)
			http.Error(w, "invalid host header", http.StatusBadRequest)
			return
		}

		userCode := generateUserCode()
		deviceCode := generateDeviceCode()
		expiresAt := time.Now().Add(10 * time.Minute)

		da, err := client.DeviceAuth.Create().
			SetUserCode(userCode).
			SetDeviceCode(deviceCode).
			SetStatus(deviceauth.StatusPENDING).
			SetExpiresAt(expiresAt).
			Save(ctx)

		if err != nil {
			rdaLog.ErrorContext(ctx, "failed to create device auth code", "error", err)
			logging.AuditFailure(ctx, "auth", "rda.code_created", "failed to create device auth code")
			http.Error(w, "failed to create device auth", http.StatusInternalServerError)
			return
		}

		rdaLog.InfoContext(ctx, "created device auth code", "device_auth_id", da.ID, "user_code", userCode)
		logging.Audit(ctx, "auth", "rda.code_created", "created remote device auth code",
			"device_auth_id", da.ID,
			"user_code", userCode,
		)

		verificationURL := &url.URL{
			Scheme: baseURL.Scheme,
			Host:   baseURL.Host,
			Path:   "/profile",
		}
		completeURL := *verificationURL
		q := url.Values{}
		q.Set("device-code", userCode)
		completeURL.RawQuery = q.Encode()

		resp := RDACodeResponse{
			UserCode:                userCode,
			DeviceCode:              deviceCode,
			VerificationURI:         verificationURL.String(),
			VerificationURIComplete: completeURL.String(),
			ExpiresIn:               600,
			Interval:                defaultRDAPollIntervalSeconds,
		}
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(resp)
	}
}

type RDATokenRequest struct {
	DeviceCode string `json:"device_code"`
}

type RDATokenResponse struct {
	Status      string `json:"status,omitempty"`
	AccessToken string `json:"access_token,omitempty"`
	Error       string `json:"error,omitempty"`
}

func extractDeviceCode(r *http.Request) (string, error) {
	if r.Method == http.MethodPost {
		var req RDATokenRequest
		if r.Body != nil {
			if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
				return "", err
			}
		}
		if req.DeviceCode != "" {
			return req.DeviceCode, nil
		}
		return strings.TrimSpace(r.Header.Get(headerDeviceCode)), nil
	}

	if code := strings.TrimSpace(r.Header.Get(headerDeviceCode)); code != "" {
		return code, nil
	}
	return r.URL.Query().Get("device_code"), nil
}

func NewRDATokenHandler(client *ent.Client) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		// Prefer POST (body/header) to avoid leaking device_code in URL query/logs,
		// while keeping GET for backwards compatibility with older CLI clients.
		if r.Method != http.MethodPost && r.Method != http.MethodGet {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		deviceCode, err := extractDeviceCode(r)
		if err != nil {
			writeRDATokenJSON(w, http.StatusBadRequest, RDATokenResponse{Error: "invalid_request"})
			return
		}
		if deviceCode == "" {
			writeRDATokenJSON(w, http.StatusBadRequest, RDATokenResponse{Error: "missing_device_code"})
			return
		}

		da, err := client.DeviceAuth.Query().Where(deviceauth.DeviceCode(deviceCode)).WithUser().First(ctx)
		if err != nil {
			if ent.IsNotFound(err) {
				rdaLog.WarnContext(ctx, "device auth token poll with invalid device_code")
				logging.AuditDenied(ctx, "auth", "rda.token_issued", "invalid device auth code presented", "reason", "invalid_device_code")
				writeRDATokenJSON(w, http.StatusNotFound, RDATokenResponse{Error: "invalid_device_code"})
				return
			}
			rdaLog.ErrorContext(ctx, "failed to query device auth by device_code", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}

		if da.Status == deviceauth.StatusDENIED {
			rdaLog.InfoContext(ctx, "device auth token poll for denied device_code", "device_auth_id", da.ID)
			writeRDATokenJSON(w, http.StatusForbidden, RDATokenResponse{
				Status: string(da.Status),
				Error:  "access_denied",
			})
			return
		}

		now := time.Now()
		if !now.Before(da.ExpiresAt) {
			rdaLog.WarnContext(ctx, "device auth token poll with expired or consumed device_code", "device_auth_id", da.ID)
			logging.AuditDenied(ctx, "auth", "rda.token_issued", "expired or consumed device auth code presented",
				"device_auth_id", da.ID,
				"user_code", da.UserCode,
				"reason", "expired_device_code",
			)
			writeRDATokenJSON(w, http.StatusGone, RDATokenResponse{
				Status: "EXPIRED",
				Error:  "expired_token",
			})
			return
		}

		switch da.Status {
		case deviceauth.StatusAPPROVED:
			if da.Edges.User == nil {
				rdaLog.ErrorContext(ctx, "approved device auth has no associated user", "device_auth_id", da.ID)
				logging.AuditFailure(ctx, "auth", "rda.token_issued", "approved device auth missing user", "device_auth_id", da.ID)
				http.Error(w, "approved device has no associated user", http.StatusInternalServerError)
				return
			}

			// Atomically consume this approved device_code so the access token can only
			// be retrieved once while keeping Status=APPROVED visible in the Profile UI.
			affected, err := client.DeviceAuth.Update().
				Where(
					deviceauth.ID(da.ID),
					deviceauth.StatusEQ(deviceauth.StatusAPPROVED),
					deviceauth.ExpiresAtGT(now),
				).
				SetExpiresAt(now.Add(-time.Second)).
				Save(ctx)
			if err != nil {
				rdaLog.ErrorContext(ctx, "failed to consume approved device auth code", "device_auth_id", da.ID, "error", err)
				http.Error(w, "internal server error", http.StatusInternalServerError)
				return
			}
			if affected == 0 {
				rdaLog.WarnContext(ctx, "device auth code already consumed", "device_auth_id", da.ID)
				logging.AuditDenied(ctx, "auth", "rda.token_issued", "already consumed device auth code presented",
					"device_auth_id", da.ID,
					"user_code", da.UserCode,
					"reason", "consumed_device_code",
				)
				writeRDATokenJSON(w, http.StatusGone, RDATokenResponse{
					Status: "EXPIRED",
					Error:  "expired_token",
				})
				return
			}

			tokenCtx := logging.WithActor(ctx, logging.Actor{
				Kind:       logging.ActorUser,
				ID:         strconv.Itoa(da.Edges.User.ID),
				Name:       da.Edges.User.Name,
				IsAdmin:    da.Edges.User.IsAdmin,
				AuthMethod: "rda",
			})
			rdaLog.InfoContext(tokenCtx, "issued access token for approved device",
				"device_auth_id", da.ID,
				"user_code", da.UserCode,
				"user_id", da.Edges.User.ID,
				"user_name", da.Edges.User.Name,
			)
			logging.Audit(tokenCtx, "auth", "rda.token_issued", "issued access token via remote device auth",
				"device_auth_id", da.ID,
				"user_code", da.UserCode,
				"user_id", da.Edges.User.ID,
				"user_name", da.Edges.User.Name,
			)
			writeRDATokenJSON(w, http.StatusOK, RDATokenResponse{
				Status:      string(da.Status),
				AccessToken: da.Edges.User.AccessToken,
			})
		default:
			rdaLog.DebugContext(ctx, "device auth token poll", "device_auth_id", da.ID, "status", string(da.Status))
			writeRDATokenJSON(w, http.StatusOK, RDATokenResponse{
				Status: string(da.Status),
			})
		}
	}
}

type RDAApproveRequest struct {
	UserCode string `json:"user_code"`
}

func NewRDAApproveHandler(client *ent.Client) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		if !validateCSRF(w, r) {
			return
		}

		var req RDAApproveRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}

		if req.UserCode == "" {
			http.Error(w, "missing user_code", http.StatusBadRequest)
			return
		}

		authUser := auth.UserFromContext(ctx)
		if authUser == nil {
			logging.AuditDenied(ctx, "auth", "rda.approved", "unauthenticated attempt to approve device auth", "user_code", req.UserCode)
			http.Error(w, "unauthenticated", http.StatusUnauthorized)
			return
		}

		da, err := client.DeviceAuth.Query().Where(deviceauth.UserCode(req.UserCode)).First(ctx)
		if err != nil {
			if ent.IsNotFound(err) {
				rdaLog.WarnContext(ctx, "invalid user_code for device approval", "user_code", req.UserCode)
				http.Error(w, "invalid user_code", http.StatusNotFound)
				return
			}
			rdaLog.ErrorContext(ctx, "failed to query device auth for approval", "user_code", req.UserCode, "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}

		if time.Now().After(da.ExpiresAt) {
			rdaLog.WarnContext(ctx, "expired user_code for device approval", "device_auth_id", da.ID, "user_code", req.UserCode)
			http.Error(w, "user_code expired", http.StatusBadRequest)
			return
		}

		if da.Status != deviceauth.StatusPENDING {
			rdaLog.WarnContext(ctx, "device auth is no longer pending", "device_auth_id", da.ID, "user_code", req.UserCode, "status", string(da.Status))
			http.Error(w, "device auth is no longer pending", http.StatusBadRequest)
			return
		}

		_, err = client.DeviceAuth.UpdateOne(da).
			SetStatus(deviceauth.StatusAPPROVED).
			SetUser(authUser).
			Save(ctx)

		if err != nil {
			rdaLog.ErrorContext(ctx, "failed to approve device auth", "device_auth_id", da.ID, "user_code", req.UserCode, "error", err)
			logging.AuditFailure(ctx, "auth", "rda.approved", "failed to approve remote device auth",
				"device_auth_id", da.ID,
				"user_code", req.UserCode,
			)
			http.Error(w, "failed to approve device", http.StatusInternalServerError)
			return
		}

		rdaLog.InfoContext(ctx, "approved remote device auth", "device_auth_id", da.ID, "user_code", req.UserCode)
		logging.Audit(ctx, "auth", "rda.approved", "approved remote device auth",
			"device_auth_id", da.ID,
			"user_code", req.UserCode,
		)

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"success":true}`))
	}
}

func NewRDARevokeHandler(client *ent.Client) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		if !validateCSRF(w, r) {
			return
		}

		var req RDAApproveRequest // Reuse same struct
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}

		if req.UserCode == "" {
			http.Error(w, "missing user_code", http.StatusBadRequest)
			return
		}

		authUser := auth.UserFromContext(ctx)
		if authUser == nil {
			logging.AuditDenied(ctx, "auth", "rda.revoked", "unauthenticated attempt to revoke device auth", "user_code", req.UserCode)
			http.Error(w, "unauthenticated", http.StatusUnauthorized)
			return
		}

		da, err := client.DeviceAuth.Query().Where(deviceauth.UserCode(req.UserCode)).WithUser().First(ctx)
		if err != nil {
			if ent.IsNotFound(err) {
				rdaLog.WarnContext(ctx, "invalid user_code for device revocation", "user_code", req.UserCode)
				http.Error(w, "invalid user_code", http.StatusNotFound)
				return
			}
			rdaLog.ErrorContext(ctx, "failed to query device auth for revocation", "user_code", req.UserCode, "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}

		if !authUser.IsAdmin && (da.Edges.User == nil || da.Edges.User.ID != authUser.ID) {
			rdaLog.WarnContext(ctx, "unauthorized attempt to revoke device auth", "device_auth_id", da.ID, "user_code", req.UserCode)
			logging.AuditDenied(ctx, "auth", "rda.revoked", "unauthorized attempt to revoke remote device auth",
				"device_auth_id", da.ID,
				"user_code", req.UserCode,
			)
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}

		// Mark the DeviceAuth record DENIED so pending/unredeemed codes are rejected
		// and the UI reflects revocation. Note: User.AccessToken is shared across all
		// CLI/API sessions for a user, so rotating it here would break other active
		// sessions; per-device token revocation requires dedicated device tokens.
		_, err = client.DeviceAuth.UpdateOne(da).SetStatus(deviceauth.StatusDENIED).Save(ctx)
		if err != nil {
			rdaLog.ErrorContext(ctx, "failed to revoke device auth", "device_auth_id", da.ID, "user_code", req.UserCode, "error", err)
			logging.AuditFailure(ctx, "auth", "rda.revoked", "failed to revoke remote device auth",
				"device_auth_id", da.ID,
				"user_code", req.UserCode,
			)
			http.Error(w, "failed to revoke device", http.StatusInternalServerError)
			return
		}

		rdaLog.InfoContext(ctx, "revoked remote device auth", "device_auth_id", da.ID, "user_code", req.UserCode)
		logging.Audit(ctx, "auth", "rda.revoked", "revoked remote device auth",
			"device_auth_id", da.ID,
			"user_code", req.UserCode,
		)

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"success":true}`))
	}
}

func NewSignoutHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		resetAuthCookie(w)

		authSessionLog.InfoContext(ctx, "user signed out")
		logging.Audit(ctx, "auth", "auth.signout", "user signed out")

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"success":true}`))
	}
}
