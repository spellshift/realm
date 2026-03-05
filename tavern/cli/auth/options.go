package auth



type AuthOptions struct {
	EnvAPIKeyName string
	CachePath     string
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

// applyOptions applies the given options to a new AuthOptions struct
func applyOptions(opts ...AuthOption) AuthOptions {
	options := AuthOptions{}
	for _, o := range opts {
		o(&options)
	}
	return options
}
