package schema

import (
	"context"
	"fmt"

	"golang.org/x/crypto/sha3"
	"realm.pub/tavern/internal/ent/hook"

	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// File holds the schema definition for the File entity.
type File struct {
	ent.Schema
}

// Fields of the File.
func (File) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			NotEmpty().
			Unique().
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("The name of the file, used to reference it for downloads"),
		field.Int("size").
			Default(0).
			Min(0).
			Annotations(
				entgql.OrderField("SIZE"),
			).
			Comment("The size of the file in bytes"),
		field.String("hash").
			MaxLen(100).
			Comment("A SHA3-256 digest of the content field"),
		field.Bytes("content").
			SchemaType(map[string]string{
				dialect.MySQL: "LONGBLOB", // Override MySQL, improve length maximum
			}).
			Annotations(
				entgql.Skip(), // Don't return file content in GraphQL queries
			).
			Comment("The content of the file"),
	}
}

// Edges of the File.
func (File) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("tomes", Tome.Type).
			Annotations(
				entgql.RelayConnection(),
				entgql.MultiOrder(),
			).
			Ref("files"),
	}
}

// Annotations describes additional information for the ent.
func (File) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}

// Mixin defines common shared properties for the ent.
func (File) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}

// Hooks defines middleware for mutations for the ent.
func (File) Hooks() []ent.Hook {
	return []ent.Hook{
		hook.On(HookDeriveFileInfo(), ent.OpCreate|ent.OpUpdate|ent.OpUpdateOne),
	}
}

// HookDeriveFileInfo will update file info (e.g. size, hash) whenever it is mutated.
func HookDeriveFileInfo() ent.Hook {
	// Get the relevant methods from the File Mutation
	// See this example: https://github.com/ent/ent/blob/master/entc/integration/hooks/ent/schema/user.go#L98
	type fMutation interface {
		Content() ([]byte, bool)
		SetSize(i int)
		SetHash(s string)
	}

	return func(next ent.Mutator) ent.Mutator {
		return ent.MutateFunc(func(ctx context.Context, m ent.Mutation) (ent.Value, error) {
			// Get the file mutation
			f, ok := m.(fMutation)
			if !ok {
				return nil, fmt.Errorf("expected file mutation in schema hook, got: %+v", m)
			}

			// Set the new size
			content, _ := f.Content()
			f.SetSize(len(content))

			// Set the new hash
			f.SetHash(fmt.Sprintf("%x", sha3.Sum256(content)))

			return next.Mutate(ctx, m)
		})
	}
}
