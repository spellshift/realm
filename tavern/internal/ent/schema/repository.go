package schema

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"encoding/pem"
	"fmt"
	"strings"

	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"golang.org/x/crypto/ssh"
	"realm.pub/tavern/internal/ent/hook"
)

// Repository holds the schema definition for the entity.
type Repository struct {
	ent.Schema
}

// Fields of the ent.
func (Repository) Fields() []ent.Field {
	return []ent.Field{
		field.String("url").
			NotEmpty().
			Unique().
			Comment("URL of the repository"),
		field.String("public_key").
			NotEmpty().
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput | entgql.SkipMutationUpdateInput),
			).
			Comment("Public key associated with this repositories private key"),
		field.String("private_key").
			NotEmpty().
			Sensitive().
			Annotations(
				entgql.Skip(entgql.SkipAll),
			).
			Comment("Private key used for authentication."),
	}
}

// Edges of the ent.
func (Repository) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("tomes", Tome.Type).
			Ref("repository").
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput, entgql.SkipMutationUpdateInput),
			).
			Comment("Tomes imported using this repository."),
		edge.To("owner", User.Type).
			Unique().
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput, entgql.SkipMutationUpdateInput),
			).
			Comment("User that created this repository."),
	}
}

// Annotations describes additional information for the ent.
func (Repository) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.Mutations(entgql.MutationCreate()),
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entsql.Annotation{Table: "repositories"},
	}
}

// Mixin defines common shared properties for the ent.
func (Repository) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // createdAt, lastModifiedAt
	}
}

// Hooks defines middleware for mutations for the ent.
func (Repository) Hooks() []ent.Hook {
	return []ent.Hook{
		hook.On(HookCreateRepoPrivateKey(), ent.OpCreate),
	}
}

// HookCreateRepoPrivateKey will generate private key for the repository upon creation.
func HookCreateRepoPrivateKey() ent.Hook {
	// Get the relevant methods from the Mutation
	// See this example: https://github.com/ent/ent/blob/master/entc/integration/hooks/ent/schema/user.go#L98
	type tMutation interface {
		URL() (string, bool)
		SetURL(string)
		PrivateKey() (string, bool)
		SetPrivateKey(s string)
		SetPublicKey(s string)
	}

	return func(next ent.Mutator) ent.Mutator {
		return ent.MutateFunc(func(ctx context.Context, m ent.Mutation) (ent.Value, error) {
			// Get the mutation
			mut, ok := m.(tMutation)
			if !ok {
				return nil, fmt.Errorf("expected repository mutation in schema hook, got: %+v", m)
			}

			// Prepend https schema if no schema specified
			if u, ok := mut.URL(); ok && (!strings.HasPrefix(u, "http://") && !strings.HasPrefix(u, "ssh://")) {
				mut.SetURL(fmt.Sprintf("https://%s", u))
			}

			// Skip if key already set
			if key, ok := mut.PrivateKey(); ok && key != "" {
				return next.Mutate(ctx, m)
			}

			// Generate new key
			_, privKey, err := ed25519.GenerateKey(rand.Reader)
			if err != nil {
				return nil, fmt.Errorf("failed to generate ed25519 private key: %w", err)
			}

			// Marshal Keys
			signer, err := ssh.NewSignerFromKey(privKey)
			if err != nil {
				return nil, fmt.Errorf("could not convert private key to ssh signer: %v", err)
			}

			block, err := ssh.MarshalPrivateKey(privKey, "")
			if err != nil || block == nil {
				return nil, fmt.Errorf("failed to marshal ssh private key: %w", err)
			}

			mut.SetPrivateKey(string(pem.EncodeToMemory(block)))
			mut.SetPublicKey(string(ssh.MarshalAuthorizedKey(signer.PublicKey())))

			return next.Mutate(ctx, m)
		})
	}
}
