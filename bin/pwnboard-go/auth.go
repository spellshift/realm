package main

import (
	"context"

	"realm.pub/tavern/cli/auth"
)

// EnvAPIKey is the name of the environment variable to optionally provide an API key
const EnvAPIKey = "TAVERN_API_KEY"

func getAuthToken(ctx context.Context, tavernURL, cachePath string) (auth.Token, error) {
	return auth.Authenticate(
		ctx,
		auth.DefaultBrowser{},
		tavernURL,
		auth.WithCacheFile(cachePath),
		auth.WithAPIKeyFromEnv(EnvAPIKey),
	)
}
