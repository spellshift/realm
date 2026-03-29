package main

import (
	"context"
	"fmt"
	"log"
	"strings"

	"google.golang.org/grpc/metadata"
	"realm.pub/tavern/cli/auth"
)

// EnvAPIKey is the name of the environment variable to optionally provide an API key
const EnvAPIKey = "TAVERN_API_TOKEN"

func getAuthToken(ctx context.Context, tavernURL, cachePath string) (auth.Token, error) {
	return auth.Authenticate(
		ctx,
		auth.DefaultBrowser{},
		tavernURL,
		auth.WithAPIKeyFromEnv(EnvAPIKey),
		auth.WithCacheFile(cachePath),
	)
}

func authGRPCContext(ctx context.Context, upstream string, authCachePath string) context.Context {
	// Default to http if no scheme provided
	if !strings.HasPrefix(upstream, "http://") && !strings.HasPrefix(upstream, "https://") {
		upstream = fmt.Sprintf("http://%s", upstream)
	}

	token, err := getAuthToken(ctx, upstream, authCachePath)
	if err != nil {
		log.Fatalf("authentication failed: %v", err)
	}

	return metadata.AppendToOutgoingContext(ctx,
		auth.HeaderAPIAccessToken, string(token),
	)
}
