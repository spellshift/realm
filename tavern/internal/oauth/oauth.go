package oauth

import "golang.org/x/oauth2"

type Oauth interface {
	GetName() string
	GetConfig() oauth2.Config
	GetUserProfiles() string
}
