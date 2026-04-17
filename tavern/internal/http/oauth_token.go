package http

import (
	"net/http"
	"strings"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/user"
)

type oauthTokenResponse struct {
	AccessToken  string `json:"access_token"`
	TokenType    string `json:"token_type"`
	ExpiresIn    int    `json:"expires_in,omitempty"`
	RefreshToken string `json:"refresh_token,omitempty"`
	Scope        string `json:"scope,omitempty"`
}

func NewOAuthTokenHandler(client *ent.Client) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		if err := r.ParseForm(); err != nil {
			http.Error(w, "invalid form body", http.StatusBadRequest)
			return
		}

		grantType := strings.TrimSpace(r.FormValue("grant_type"))
		switch grantType {
		case "authorization_code":
			handleAuthorizationCodeGrant(w, r, client)
			return
		case "refresh_token":
			handleRefreshTokenGrant(w, r, client)
			return
		default:
			http.Error(w, "unsupported grant_type", http.StatusBadRequest)
			return
		}
	}
}

func handleAuthorizationCodeGrant(w http.ResponseWriter, r *http.Request, client *ent.Client) {
	code := strings.TrimSpace(r.FormValue("code"))
	if code == "" {
		http.Error(w, "missing code", http.StatusBadRequest)
		return
	}

	usr, err := client.User.Query().
		Where(user.AccessToken(code)).
		Only(r.Context())
	if err != nil {
		http.Error(w, "invalid code", http.StatusUnauthorized)
		return
	}

	resp := oauthTokenResponse{
		AccessToken:  usr.AccessToken,
		TokenType:    "Bearer",
		ExpiresIn:    3600,
		RefreshToken: usr.AccessToken,
		Scope:        strings.TrimSpace(r.FormValue("scope")),
	}

	writeJSON(w, resp)
}

func handleRefreshTokenGrant(w http.ResponseWriter, r *http.Request, client *ent.Client) {
	refreshToken := strings.TrimSpace(r.FormValue("refresh_token"))
	if refreshToken == "" {
		http.Error(w, "missing refresh_token", http.StatusBadRequest)
		return
	}

	usr, err := client.User.Query().
		Where(user.AccessToken(refreshToken)).
		Only(r.Context())
	if err != nil {
		http.Error(w, "invalid refresh_token", http.StatusUnauthorized)
		return
	}

	resp := oauthTokenResponse{
		AccessToken:  usr.AccessToken,
		TokenType:    "Bearer",
		ExpiresIn:    3600,
		RefreshToken: usr.AccessToken,
		Scope:        strings.TrimSpace(r.FormValue("scope")),
	}

	writeJSON(w, resp)
}
