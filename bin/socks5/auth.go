package main

import (
	"context"
	"fmt"
	"log"
	"os"
	"strings"

	"google.golang.org/grpc/metadata"
	"realm.pub/tavern/cli/auth"
)

// EnvAPIToken is the name of the environment variable to optionally provide an API token
const EnvAPIToken = "TAVERN_API_TOKEN"

func getAuthToken(ctx context.Context, tavernURL, cachePath string) (auth.Token, error) {
	if token := os.Getenv(EnvAPIToken); token != "" {
		return auth.Token(token), nil
	}

	tokenData, err := os.ReadFile(cachePath)
	if os.IsNotExist(err) {
		token, err := auth.Authenticate(
			ctx,
			auth.BrowserFunc(
				func(url string) error {
					fmt.Printf("\n\nTavern Authentication URL: %s\n\n", url)
					return nil
				},
			),
			tavernURL,
		)
		if err != nil {
			return auth.Token(""), err
		}

		if err := os.WriteFile(cachePath, []byte(token), 0640); err != nil {
			log.Printf("[WARN] Failed to save token to credential cache (%q): %v", cachePath, err)
		}
		return token, nil
	}
	if err != nil {
		return auth.Token(""), fmt.Errorf("failed to read credential cache (%q): %v", cachePath, err)
	}

	log.Printf("Loaded authentication credentials from %q", cachePath)
	return auth.Token(tokenData), nil
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
