package main

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"strings"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/builderenrollmenttoken"
)

// parseBuilderEnrollmentTokens validates the full list before any token is saved.
func parseBuilderEnrollmentTokens(value string) ([]string, error) {
	if value == "" {
		return nil, nil
	}
	items := strings.Split(value, ",")
	for i := range items {
		items[i] = strings.TrimSpace(items[i])
		if items[i] == "" {
			return nil, fmt.Errorf("%s contains an empty item", EnvBuilderEnrollmentTokens.Key)
		}
	}
	return items, nil
}

func seedBuilderEnrollmentTokens(ctx context.Context, client *ent.Client, tokens []string) error {
	for _, token := range tokens {
		hash := sha256.Sum256([]byte(token))
		if err := client.BuilderEnrollmentToken.Create().
			SetTokenHash(hex.EncodeToString(hash[:])).
			OnConflictColumns(builderenrollmenttoken.FieldTokenHash).
			Ignore().Exec(ctx); err != nil {
			return err
		}
	}
	return nil
}
