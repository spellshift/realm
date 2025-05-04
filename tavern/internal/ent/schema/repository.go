package schema

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"encoding/pem"
	"fmt"
	"net/url"
	"regexp"
	"strings"

	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
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
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT", // Override MySQL, improve length maximum
			}).
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput | entgql.SkipMutationUpdateInput),
			).
			Comment("Public key associated with this repositories private key"),
		field.String("private_key").
			NotEmpty().
			Sensitive().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT", // Override MySQL, improve length maximum
			}).
			Annotations(
				entgql.Skip(entgql.SkipAll),
			).
			Comment("Private key used for authentication."),
		field.Time("last_imported_at").
			Optional().
			Annotations(
				entgql.OrderField("LAST_IMPORTED_AT"),
				entgql.Skip(entgql.SkipMutationCreateInput, entgql.SkipMutationUpdateInput),
			).
			Comment("Timestamp of when this repo was last imported"),
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
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
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
		hook.On(HookDeriveRepoOnCreate(), ent.OpCreate),
	}
}

// HookDeriveRepoOnCreate will generate private key for the repository upon creation.
// It will also format the git URL (if one is specified).
func HookDeriveRepoOnCreate() ent.Hook {
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

			// Format the URL (and detect errors)
			if rawurl, ok := mut.URL(); ok {
				formattedURL, err := FormatGitURL(rawurl)
				if err != nil {
					return nil, fmt.Errorf("failed to format git url: %w", err)
				}

				mut.SetURL(formattedURL.String())
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

var scpRegex = regexp.MustCompile(`^(ssh://)?([a-zA-Z0-9_]+)@([a-zA-Z0-9._-]+)(:\d+)?[:/]([a-zA-Z0-9./._-]+)(?:\?||$)(.*)$`)

func FormatGitURL(rawurl string) (*url.URL, error) {
	rawurl = strings.TrimSpace(rawurl)

	// If it's http(s), return the parsed url
	if strings.HasPrefix(rawurl, "http://") || strings.HasPrefix(rawurl, "https://") {
		return url.Parse(rawurl)
	}

	// Handle SCP (if user specified)
	scpParts := scpRegex.FindStringSubmatch(rawurl)
	if scpParts != nil {
		var (
			scheme   = "ssh"
			user     = "git"
			rawquery = ""
		)

		if scpParts[1] != "" {
			scheme = strings.TrimSuffix(scpParts[1], "://")
		}
		if scpParts[2] != "" {
			user = scpParts[2]
		}
		if len(scpParts) > 5 {
			rawquery = scpParts[6]
		}

		host := fmt.Sprintf("%s%s", scpParts[3], scpParts[4])
		return &url.URL{
			Scheme:   scheme,
			User:     url.User(user),
			Host:     host,
			Path:     scpParts[5],
			RawQuery: rawquery,
		}, nil

	}

	u, err := url.Parse(rawurl)
	if err != nil {
		return nil, err
	}

	// Handle SCP : with no user specified
	if u.Opaque != "" && u.Scheme != "ssh" && u.Scheme != "" && u.User == nil {
		return &url.URL{
			Scheme:   "ssh",
			User:     url.User("git"),
			Host:     u.Scheme, // How url will parse host with : instead of /
			Path:     u.Opaque,
			RawQuery: u.RawQuery,
		}, nil
	}

	// Default to SSH
	if u.Scheme == "" {
		u.Scheme = "ssh"
	}
	if u.User == nil {
		u.User = url.User("git")
	}
	return u, nil
}
