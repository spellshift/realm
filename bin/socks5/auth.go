package main

import (
	"context"
	"log"

	"google.golang.org/grpc/metadata"
	"realm.pub/tavern/cli/auth"
)

func getAuthToken(ctx context.Context, tavernURL, cachePath string) (auth.Token, error) {
	return auth.Authenticate(
		ctx,
		auth.DefaultBrowser{},
		tavernURL,
		auth.WithAPIKeyFromEnv(auth.EnvAPIToken),
		auth.WithCacheFile(cachePath),
	)
}

func authGRPCContext(ctx context.Context, upstream string, authCachePath string) context.Context {
	upstream = normalizeUpstreamURL(upstream)

	token, err := getAuthToken(ctx, upstream, authCachePath)
	if err != nil {
		log.Fatalf("authentication failed: %v", err)
	}

	return metadata.AppendToOutgoingContext(ctx,
		auth.HeaderAPIAccessToken, string(token),
	)
}
