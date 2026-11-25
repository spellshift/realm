// Package enrollment contains the shared one-time token lifecycle for builders
// and redirectors. Each kind uses its own table, so tokens cannot cross roles.
package enrollment

import (
	"context"
	"crypto/rand"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"strings"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/builderenrollmenttoken"
	"realm.pub/tavern/internal/ent/redirectorenrollmenttoken"
)

type Kind uint8

const (
	Builder Kind = iota
	Redirector
)

func Hash(token string) string {
	hash := sha256.Sum256([]byte(token))
	return hex.EncodeToString(hash[:])
}

func ParseTokens(value, envKey string) ([]string, error) {
	if value == "" {
		return nil, nil
	}
	items := strings.Split(value, ",")
	for i := range items {
		items[i] = strings.TrimSpace(items[i])
		if items[i] == "" {
			return nil, fmt.Errorf("%s contains an empty item", envKey)
		}
	}
	return items, nil
}

func Issue(ctx context.Context, client *ent.Client, kind Kind) (string, int, error) {
	raw := make([]byte, 32)
	if _, err := rand.Read(raw); err != nil {
		return "", 0, err
	}
	token := base64.RawURLEncoding.EncodeToString(raw)
	var id int
	switch kind {
	case Builder:
		row, err := client.BuilderEnrollmentToken.Create().SetTokenHash(Hash(token)).Save(ctx)
		if err != nil {
			return "", 0, err
		}
		id = row.ID
	case Redirector:
		row, err := client.RedirectorEnrollmentToken.Create().SetTokenHash(Hash(token)).Save(ctx)
		if err != nil {
			return "", 0, err
		}
		id = row.ID
	default:
		return "", 0, fmt.Errorf("invalid enrollment kind %d", kind)
	}
	return token, id, nil
}

func Seed(ctx context.Context, client *ent.Client, kind Kind, tokens []string) error {
	for _, token := range tokens {
		var err error
		switch kind {
		case Builder:
			err = client.BuilderEnrollmentToken.Create().SetTokenHash(Hash(token)).OnConflictColumns(builderenrollmenttoken.FieldTokenHash).Ignore().Exec(ctx)
		case Redirector:
			err = client.RedirectorEnrollmentToken.Create().SetTokenHash(Hash(token)).OnConflictColumns(redirectorenrollmenttoken.FieldTokenHash).Ignore().Exec(ctx)
		default:
			return fmt.Errorf("invalid enrollment kind %d", kind)
		}
		if err != nil {
			return err
		}
	}
	return nil
}

func Available(ctx context.Context, client *ent.Client, kind Kind, token string) (bool, error) {
	hash := Hash(token)
	switch kind {
	case Builder:
		return client.BuilderEnrollmentToken.Query().Where(builderenrollmenttoken.TokenHash(hash), builderenrollmenttoken.Used(false)).Exist(ctx)
	case Redirector:
		return client.RedirectorEnrollmentToken.Query().Where(redirectorenrollmenttoken.TokenHash(hash), redirectorenrollmenttoken.Used(false)).Exist(ctx)
	default:
		return false, fmt.Errorf("invalid enrollment kind %d", kind)
	}
}

// Consume atomically marks one token as used inside the caller's transaction.
// The caller must create the identity in the same transaction before commit.
func Consume(ctx context.Context, tx *ent.Tx, kind Kind, token string) (bool, error) {
	hash := Hash(token)
	var count int
	var err error
	switch kind {
	case Builder:
		count, err = tx.BuilderEnrollmentToken.Update().Where(builderenrollmenttoken.TokenHash(hash), builderenrollmenttoken.Used(false)).SetUsed(true).Save(ctx)
	case Redirector:
		count, err = tx.RedirectorEnrollmentToken.Update().Where(redirectorenrollmenttoken.TokenHash(hash), redirectorenrollmenttoken.Used(false)).SetUsed(true).Save(ctx)
	default:
		return false, fmt.Errorf("invalid enrollment kind %d", kind)
	}
	return count == 1, err
}
