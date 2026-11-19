package main

import (
	"context"

	"realm.pub/tavern/internal/enrollment"
	"realm.pub/tavern/internal/ent"
)

func parseRedirectorEnrollmentTokens(value string) ([]string, error) {
	return enrollment.ParseTokens(value, EnvRedirectorEnrollmentTokens.Key)
}

func seedRedirectorEnrollmentTokens(ctx context.Context, client *ent.Client, tokens []string) error {
	return enrollment.Seed(ctx, client, enrollment.Redirector, tokens)
}
