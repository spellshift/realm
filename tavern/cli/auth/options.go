package auth

// EnvUseBrowserOAuth is the environment variable that enables the legacy local
// browser OAuth flow instead of the default Remote Device Authentication (RDA) flow.
const EnvUseBrowserOAuth = "TAVERN_USE_BROWSER_OAUTH"

type AuthOptions struct {
	EnvAPIKeyName   string
	CachePath       string
	UseBrowserOAuth bool
}

type AuthOption func(*AuthOptions)

func WithAPIKeyFromEnv(name string) AuthOption {
	return func(o *AuthOptions) {
		o.EnvAPIKeyName = name
	}
}

func WithCacheFile(path string) AuthOption {
	return func(o *AuthOptions) {
		o.CachePath = path
	}
}

// WithBrowserOAuth enables the legacy local browser OAuth flow instead of the
// default Remote Device Authentication (RDA) flow.
func WithBrowserOAuth() AuthOption {
	return func(o *AuthOptions) {
		o.UseBrowserOAuth = true
	}
}

// applyOptions applies the given options to a new AuthOptions struct
func applyOptions(opts ...AuthOption) AuthOptions {
	options := AuthOptions{}
	for _, o := range opts {
		o(&options)
	}
	return options
}
