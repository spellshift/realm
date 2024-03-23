package main

import (
	"context"
	"fmt"
	"log"
	"os"

	"realm.pub/tavern/cli/auth"
)

func getAuthToken(ctx context.Context, tavernURL, cachePath string) (auth.Token, error) {
	tokenData, err := os.ReadFile(cachePath)
	if os.IsNotExist(err) {
		token, err := auth.Authenticate(ctx, auth.BrowserFunc(func(url string) error { log.Printf("OPEN THIS: %s", url); return nil }), tavernURL)

		// token, err := auth.Authenticate(ctx, auth.BrowserFunc(browser.OpenURL), tavernURL)
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
