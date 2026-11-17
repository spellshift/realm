package http

import (
	"crypto/rand"
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"
	"time"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/deviceauth"
	"realm.pub/tavern/internal/logging"
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
}

func NewRDACodeHandler(client *ent.Client) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		// Only allow POST
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
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

		scheme := "http"
		if r.TLS != nil || r.Header.Get("X-Forwarded-Proto") == "https" {
			scheme = "https"
		}
		verificationURI := fmt.Sprintf("%s://%s/profile", scheme, r.Host)

		resp := RDACodeResponse{
			UserCode:                userCode,
			DeviceCode:              deviceCode,
			VerificationURI:         verificationURI,
			VerificationURIComplete: fmt.Sprintf("%s?device-code=%s", verificationURI, userCode),
			ExpiresIn:               600,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}
}

type RDATokenResponse struct {
	Status      string `json:"status"`
	AccessToken string `json:"access_token,omitempty"`
}

func NewRDATokenHandler(client *ent.Client) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		// Only allow GET
		if r.Method != http.MethodGet {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		deviceCode := r.URL.Query().Get("device_code")
		if deviceCode == "" {
			http.Error(w, "missing device_code query param", http.StatusBadRequest)
			return
		}

		da, err := client.DeviceAuth.Query().Where(deviceauth.DeviceCode(deviceCode)).WithUser().First(ctx)
		if err != nil {
			if ent.IsNotFound(err) {
				rdaLog.WarnContext(ctx, "device auth token poll with invalid device_code")
				logging.AuditDenied(ctx, "auth", "rda.token_issued", "invalid device auth code presented", "reason", "invalid_device_code")
				http.Error(w, "invalid device_code", http.StatusNotFound)
				return
			}
			rdaLog.ErrorContext(ctx, "failed to query device auth by device_code", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}

		if time.Now().After(da.ExpiresAt) {
			rdaLog.WarnContext(ctx, "device auth token poll with expired device_code", "device_auth_id", da.ID)
			logging.AuditDenied(ctx, "auth", "rda.token_issued", "expired device auth code presented",
				"device_auth_id", da.ID,
				"user_code", da.UserCode,
				"reason", "expired_device_code",
			)
			http.Error(w, "device_code expired", http.StatusBadRequest)
			return
		}

		resp := RDATokenResponse{
			Status: string(da.Status),
		}

		switch da.Status {
		case deviceauth.StatusAPPROVED:
			if da.Edges.User == nil {
				rdaLog.ErrorContext(ctx, "approved device auth has no associated user", "device_auth_id", da.ID)
				logging.AuditFailure(ctx, "auth", "rda.token_issued", "approved device auth missing user", "device_auth_id", da.ID)
				http.Error(w, "approved device has no associated user", http.StatusInternalServerError)
				return
			}
			resp.AccessToken = da.Edges.User.AccessToken

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
		default:
			rdaLog.DebugContext(ctx, "device auth token poll", "device_auth_id", da.ID, "status", string(da.Status))
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
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

		var req RDAApproveRequest // Reuse same struct
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid request body", http.StatusBadRequest)
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
			rdaLog.WarnContext(ctx, "invalid user_code for device revocation", "user_code", req.UserCode)
			http.Error(w, "invalid user_code", http.StatusNotFound)
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

		// Deny it instead of deleting so UI shows it revoked
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
