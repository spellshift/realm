package main

import (
	"context"

	"realm.pub/tavern/internal/enrollment"
	"realm.pub/tavern/internal/ent"
)

// parseBuilderEnrollmentTokens validates the full list before any token is saved.
func parseBuilderEnrollmentTokens(value string) ([]string, error) {
	return enrollment.ParseTokens(value, EnvBuilderEnrollmentTokens.Key)
}

func seedBuilderEnrollmentTokens(ctx context.Context, client *ent.Client, tokens []string) error {
	return enrollment.Seed(ctx, client, enrollment.Builder, tokens)
}
