package main

import (
	"context"
	"os"
	"strings"

	"realm.pub/tavern/cli/auth"
)

// legacyEnvAPIKey is retained for backwards compatibility with older pwnboard-go deployments.
const legacyEnvAPIKey = "TAVERN_API_KEY"

func getAuthToken(ctx context.Context, tavernURL, cachePath string) (auth.Token, error) {
	envVar := auth.EnvAPIToken
	if strings.TrimSpace(os.Getenv(auth.EnvAPIToken)) == "" && strings.TrimSpace(os.Getenv(legacyEnvAPIKey)) != "" {
		envVar = legacyEnvAPIKey
	}

	return auth.Authenticate(
		ctx,
		auth.DefaultBrowser{},
		tavernURL,
		auth.WithCacheFile(cachePath),
		auth.WithAPIKeyFromEnv(envVar),
	)
}
