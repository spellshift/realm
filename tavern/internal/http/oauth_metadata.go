package http

import (
	"encoding/json"
	"net/http"
	"strings"
)

type oauthProtectedResourceMetadata struct {
	Resource               string   `json:"resource"`
	AuthorizationServers   []string `json:"authorization_servers"`
	BearerMethodsSupported []string `json:"bearer_methods_supported"`
	ScopesSupported        []string `json:"scopes_supported,omitempty"`
}

type oauthAuthorizationServerMetadata struct {
	Issuer                                string   `json:"issuer"`
	AuthorizationEndpoint                 string   `json:"authorization_endpoint"`
	TokenEndpoint                         string   `json:"token_endpoint"`
	RegistrationEndpoint                  string   `json:"registration_endpoint,omitempty"`
	ScopesSupported                       []string `json:"scopes_supported"`
	ResponseTypesSupported                []string `json:"response_types_supported"`
	GrantTypesSupported                   []string `json:"grant_types_supported"`
	TokenEndpointAuthMethodsSupported     []string `json:"token_endpoint_auth_methods_supported"`
	CodeChallengeMethodsSupported         []string `json:"code_challenge_methods_supported"`
	AuthorizationResponseIssParameterSupported bool `json:"authorization_response_iss_parameter_supported"`
}

type openIDConfiguration struct {
	Issuer                            string   `json:"issuer"`
	AuthorizationEndpoint             string   `json:"authorization_endpoint"`
	TokenEndpoint                     string   `json:"token_endpoint"`
	RegistrationEndpoint              string   `json:"registration_endpoint,omitempty"`
	ScopesSupported                   []string `json:"scopes_supported"`
	ResponseTypesSupported            []string `json:"response_types_supported"`
	GrantTypesSupported               []string `json:"grant_types_supported"`
	TokenEndpointAuthMethodsSupported []string `json:"token_endpoint_auth_methods_supported"`
	CodeChallengeMethodsSupported     []string `json:"code_challenge_methods_supported"`
	SubjectTypesSupported             []string `json:"subject_types_supported"`
	IDTokenSigningAlgValuesSupported  []string `json:"id_token_signing_alg_values_supported"`
}

func NewOAuthProtectedResourceMetadataHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		issuer := requestIssuer(r)
		resource := issuer + "/mcp"
		metadata := oauthProtectedResourceMetadata{
			Resource:               resource,
			AuthorizationServers:   []string{issuer},
			BearerMethodsSupported: []string{"header"},
			ScopesSupported:        []string{"mcp:read", "mcp:tools:invoke"},
		}
		writeJSON(w, metadata)
	}
}

func NewOAuthAuthorizationServerMetadataHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		issuer := requestIssuer(r)
		metadata := oauthAuthorizationServerMetadata{
			Issuer:                                issuer,
			AuthorizationEndpoint:                 issuer + "/oauth/login",
			TokenEndpoint:                         issuer + "/oauth/token",
			RegistrationEndpoint:                  issuer + "/oauth/register",
			ScopesSupported:                       []string{"mcp:read", "mcp:tools:invoke", "offline_access"},
			ResponseTypesSupported:                []string{"code"},
			GrantTypesSupported:                   []string{"authorization_code", "refresh_token"},
			TokenEndpointAuthMethodsSupported:     []string{"client_secret_post", "client_secret_basic"},
			CodeChallengeMethodsSupported:         []string{"S256"},
			AuthorizationResponseIssParameterSupported: true,
		}
		writeJSON(w, metadata)
	}
}

func NewOpenIDConfigurationHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		issuer := requestIssuer(r)
		cfg := openIDConfiguration{
			Issuer:                            issuer,
			AuthorizationEndpoint:             issuer + "/oauth/login",
			TokenEndpoint:                     issuer + "/oauth/token",
			RegistrationEndpoint:              issuer + "/oauth/register",
			ScopesSupported:                   []string{"openid", "profile", "mcp:read", "mcp:tools:invoke", "offline_access"},
			ResponseTypesSupported:            []string{"code"},
			GrantTypesSupported:               []string{"authorization_code", "refresh_token"},
			TokenEndpointAuthMethodsSupported: []string{"client_secret_post", "client_secret_basic"},
			CodeChallengeMethodsSupported:     []string{"S256"},
			SubjectTypesSupported:             []string{"public"},
			IDTokenSigningAlgValuesSupported:  []string{"EdDSA"},
		}
		writeJSON(w, cfg)
	}
}

func requestIssuer(r *http.Request) string {
	scheme := "http"
	if r.TLS != nil {
		scheme = "https"
	}
	if forwardedProto := strings.TrimSpace(r.Header.Get("X-Forwarded-Proto")); forwardedProto != "" {
		scheme = forwardedProto
	}
	return scheme + "://" + r.Host
}

func writeJSON(w http.ResponseWriter, value any) {
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(value)
}
