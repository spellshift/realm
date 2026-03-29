package http

import (
	"crypto/rand"
	"encoding/json"
	"fmt"
	"net/http"
	"time"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/deviceauth"
)

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
		// Only allow POST
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		userCode := generateUserCode()
		deviceCode := generateDeviceCode()
		expiresAt := time.Now().Add(10 * time.Minute)

		_, err := client.DeviceAuth.Create().
			SetUserCode(userCode).
			SetDeviceCode(deviceCode).
			SetStatus(deviceauth.StatusPENDING).
			SetExpiresAt(expiresAt).
			Save(r.Context())

		if err != nil {
			http.Error(w, "failed to create device auth", http.StatusInternalServerError)
			return
		}

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

		da, err := client.DeviceAuth.Query().Where(deviceauth.DeviceCode(deviceCode)).WithUser().First(r.Context())
		if err != nil {
			if ent.IsNotFound(err) {
				http.Error(w, "invalid device_code", http.StatusNotFound)
				return
			}
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}

		if time.Now().After(da.ExpiresAt) {
			http.Error(w, "device_code expired", http.StatusBadRequest)
			return
		}

		resp := RDATokenResponse{
			Status: string(da.Status),
		}

		if da.Status == deviceauth.StatusAPPROVED {
			if da.Edges.User == nil {
				http.Error(w, "approved device has no associated user", http.StatusInternalServerError)
				return
			}
			resp.AccessToken = da.Edges.User.AccessToken

			// Optional: transition to a used state or delete the device auth so it can't be reused,
			// but we will keep it around so users can see approved devices in UI and revoke them!
			// We might want to use a separate token, but just returning AccessToken works.
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

		authUser := auth.UserFromContext(r.Context())
		if authUser == nil {
			http.Error(w, "unauthenticated", http.StatusUnauthorized)
			return
		}

		da, err := client.DeviceAuth.Query().Where(deviceauth.UserCode(req.UserCode)).First(r.Context())
		if err != nil {
			if ent.IsNotFound(err) {
				http.Error(w, "invalid user_code", http.StatusNotFound)
				return
			}
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}

		if time.Now().After(da.ExpiresAt) {
			http.Error(w, "user_code expired", http.StatusBadRequest)
			return
		}

		if da.Status != deviceauth.StatusPENDING {
			http.Error(w, "device auth is no longer pending", http.StatusBadRequest)
			return
		}

		_, err = client.DeviceAuth.UpdateOne(da).
			SetStatus(deviceauth.StatusAPPROVED).
			SetUser(authUser).
			Save(r.Context())

		if err != nil {
			http.Error(w, "failed to approve device", http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"success":true}`))
	}
}

func NewRDARevokeHandler(client *ent.Client) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req RDAApproveRequest // Reuse same struct
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}

		authUser := auth.UserFromContext(r.Context())
		if authUser == nil {
			http.Error(w, "unauthenticated", http.StatusUnauthorized)
			return
		}

		da, err := client.DeviceAuth.Query().Where(deviceauth.UserCode(req.UserCode)).WithUser().First(r.Context())
		if err != nil {
			http.Error(w, "invalid user_code", http.StatusNotFound)
			return
		}

		if !authUser.IsAdmin && (da.Edges.User == nil || da.Edges.User.ID != authUser.ID) {
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}

		// Deny it instead of deleting so UI shows it revoked
		_, err = client.DeviceAuth.UpdateOne(da).SetStatus(deviceauth.StatusDENIED).Save(r.Context())
		if err != nil {
			http.Error(w, "failed to revoke device", http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"success":true}`))
	}
}

func NewSignoutHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		resetAuthCookie(w)
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"success":true}`))
	}
}
