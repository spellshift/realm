package main

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"strings"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/redirectorenrollmenttoken"
)

func parseRedirectorEnrollmentTokens(value string) ([]string, error) {
	if value == "" {
		return nil, nil
	}
	items := strings.Split(value, ",")
	for i := range items {
		items[i] = strings.TrimSpace(items[i])
		if items[i] == "" {
			return nil, fmt.Errorf("%s contains an empty item", EnvRedirectorEnrollmentTokens.Key)
		}
	}
	return items, nil
}

func seedRedirectorEnrollmentTokens(ctx context.Context, client *ent.Client, tokens []string) error {
	for _, token := range tokens {
		hash := sha256.Sum256([]byte(token))
		if err := client.RedirectorEnrollmentToken.Create().SetTokenHash(hex.EncodeToString(hash[:])).OnConflictColumns(redirectorenrollmenttoken.FieldTokenHash).Ignore().Exec(ctx); err != nil {
			return err
		}
	}
	return nil
}
