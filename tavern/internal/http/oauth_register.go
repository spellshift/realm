package http

import (
	"crypto/rand"
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

type dynamicClientRegistrationRequest struct {
	ClientName              string   `json:"client_name"`
	RedirectURIs            []string `json:"redirect_uris"`
	GrantTypes              []string `json:"grant_types"`
	ResponseTypes           []string `json:"response_types"`
	TokenEndpointAuthMethod string   `json:"token_endpoint_auth_method"`
	Scope                   string   `json:"scope"`
}

type dynamicClientRegistrationResponse struct {
	ClientID                string   `json:"client_id"`
	ClientSecret            string   `json:"client_secret,omitempty"`
	ClientIDIssuedAt        int64    `json:"client_id_issued_at"`
	ClientSecretExpiresAt   int64    `json:"client_secret_expires_at"`
	ClientName              string   `json:"client_name,omitempty"`
	RedirectURIs            []string `json:"redirect_uris,omitempty"`
	GrantTypes              []string `json:"grant_types,omitempty"`
	ResponseTypes           []string `json:"response_types,omitempty"`
	TokenEndpointAuthMethod string   `json:"token_endpoint_auth_method,omitempty"`
	Scope                   string   `json:"scope,omitempty"`
}

func NewOAuthDynamicClientRegistrationHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req dynamicClientRegistrationRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}

		clientID, err := generateOAuthClientID()
		if err != nil {
			http.Error(w, "failed to generate client id", http.StatusInternalServerError)
			return
		}
		clientSecret, err := generateOAuthClientSecret()
		if err != nil {
			http.Error(w, "failed to generate client secret", http.StatusInternalServerError)
			return
		}

		if len(req.GrantTypes) == 0 {
			req.GrantTypes = []string{"authorization_code", "refresh_token"}
		}
		if len(req.ResponseTypes) == 0 {
			req.ResponseTypes = []string{"code"}
		}
		if req.TokenEndpointAuthMethod == "" {
			req.TokenEndpointAuthMethod = "client_secret_post"
		}

		resp := dynamicClientRegistrationResponse{
			ClientID:                clientID,
			ClientSecret:            clientSecret,
			ClientIDIssuedAt:        time.Now().Unix(),
			ClientSecretExpiresAt:   0,
			ClientName:              req.ClientName,
			RedirectURIs:            req.RedirectURIs,
			GrantTypes:              req.GrantTypes,
			ResponseTypes:           req.ResponseTypes,
			TokenEndpointAuthMethod: req.TokenEndpointAuthMethod,
			Scope:                   req.Scope,
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusCreated)
		_ = json.NewEncoder(w).Encode(resp)
	}
}

func generateOAuthClientID() (string, error) {
	b := make([]byte, 16)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return fmt.Sprintf("dcr_%x", b), nil
}

func generateOAuthClientSecret() (string, error) {
	b := make([]byte, 32)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return fmt.Sprintf("%x", b), nil
}
