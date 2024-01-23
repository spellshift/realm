package schema

import (
	"context"
	"fmt"

	"golang.org/x/crypto/sha3"
	"realm.pub/tavern/internal/ent/hook"

	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// HostFile holds the schema definition for the HostFile entity.
type HostFile struct {
	ent.Schema
}

// Fields of the HostFile.
func (HostFile) Fields() []ent.Field {
	return []ent.Field{
		field.String("path").
			NotEmpty().
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("Path to the file on the host system."),
		field.String("owner").
			Optional().
			Comment("User who owns the file on the host system."),
		field.String("group").
			Optional().
			Comment("Group who owns the file on the host system."),
		field.String("permissions").
			Optional().
			Comment("Permissions for the file on the host system."),
		field.Int("size").
			Default(0).
			Min(0).
			Annotations(
				entgql.OrderField("SIZE"),
			).
			Comment("The size of the file in bytes"),
		field.String("hash").
			Optional().
			MaxLen(100).
			Comment("A SHA3-256 digest of the content field"),
		field.Bytes("content").
			Optional().
			Annotations(
				entgql.Skip(), // Don't return file content in GraphQL queries
			).
			Comment("The content of the file"),
	}
}

// Edges of the ent.
func (HostFile) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("host", Host.Type).
			Required().
			Unique().
			Comment("Host the file was reported on."),
		edge.From("task", Task.Type).
			Required().
			Unique().
			Ref("reported_files").
			Comment("Task that reported this file."),
	}
}

// Annotations describes additional information for the ent.
func (HostFile) Annotations() []schema.Annotation {
	return []schema.Annotation{}
}

// Mixin defines common shared properties for the ent.
func (HostFile) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}

// Hooks defines middleware for mutations for the ent.
func (HostFile) Hooks() []ent.Hook {
	return []ent.Hook{
		hook.On(HookDeriveHostFileInfo(), ent.OpCreate|ent.OpUpdate|ent.OpUpdateOne),
	}
}

// HookDeriveHostFileInfo will update file info (e.g. size, hash) whenever it is mutated.
func HookDeriveHostFileInfo() ent.Hook {
	// Get the relevant methods from the HostFile Mutation
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
				return nil, fmt.Errorf("expected hostfile mutation in schema hook, got: %+v", m)
			}

			// Set the new size
			content, _ := f.Content()
			f.SetSize(len(content))

			// Set the new hash (if content exists)
			if len(content) > 0 {
				f.SetHash(fmt.Sprintf("%x", sha3.Sum256(content)))
			}

			return next.Mutate(ctx, m)
		})
	}
}
