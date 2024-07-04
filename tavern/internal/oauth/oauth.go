package oauth

import "golang.org/x/oauth2"

type Oauth interface {
	GetName() string
	GetConfig() oauth2.Config
	GetUserProfiles() string
}

func Default(clientID string, clientSecret string, domain string, redirectPath string) Oauth {
	return NewGoogle(clientID, clientSecret, domain, redirectPath)
}
