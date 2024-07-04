package oauth

import (
	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"
)

type Google struct {
	Name         string
	Config       oauth2.Config
	UserProfiles string
}

// GetName implements Oauth.
func (g Google) GetName() string {
	return g.Name
}

// GetConfig implements Oauth.
func (g Google) GetConfig() oauth2.Config {
	return g.Config
}

// GetUserProfiles implements Oauth.
func (g Google) GetUserProfiles() string {
	return g.UserProfiles
}

func NewGoogle(clientID string, clientSecret string, domain string, redirectPath string) Oauth {
	return Google{
		Name: "google",
		Config: oauth2.Config{
			ClientID:     clientID,
			ClientSecret: clientSecret,
			RedirectURL:  domain + redirectPath,
			Scopes: []string{
				"https://www.googleapis.com/auth/userinfo.profile",
			},
			Endpoint: google.Endpoint,
		},
		UserProfiles: "https://www.googleapis.com/oauth2/v3/userinfo",
	}
}
