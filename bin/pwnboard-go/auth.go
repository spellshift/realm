package main

import (
	"context"
	"fmt"
	"log"
	"os"

	"realm.pub/tavern/cli/auth"
)

func getAuthToken(ctx context.Context, tavernURL, cachePath string) (auth.Token, error) {
	return auth.Authenticate(
		ctx,
		auth.DefaultBrowser{},
		tavernURL,
		auth.WithCacheFile(cachePath),
	)
}
