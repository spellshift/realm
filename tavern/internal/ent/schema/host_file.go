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
		field.Uint64("size").
			Default(0).
			Min(0).
			Annotations(
				entgql.Type("Uint64"),
				entgql.OrderField("SIZE"),
			).
			Comment("The size of the file in bytes"),
		field.String("hash").
			Optional().
			MaxLen(100).
			Comment("A SHA3-256 digest of the content field"),
		field.Bytes("content").
			Optional().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGBLOB", // Override MySQL, improve length maximum
			}).
			Annotations(
				entgql.Skip(), // Don't return file content in GraphQL queries
			).
			Comment("The content of the file"),
		field.Enum("preview_type").
			Values("TEXT", "NONE").
			Default("NONE").
			Comment("The type of preview available for the file"),
		field.Bytes("preview").
			Optional().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGBLOB", // Override MySQL, improve length maximum
			}).
			Annotations(
				entgql.Type("String"),
			).
			Comment("A preview of the file content (max 512kb)"),
	}
}

// Edges of the ent.
func (HostFile) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("host", Host.Type).
			Required().
			Unique().
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Host the file was reported on."),
		edge.From("task", Task.Type).
			Unique().
			Ref("reported_files").
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Task that reported this file."),
		edge.From("shell_task", ShellTask.Type).
			Unique().
			Ref("reported_files").
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Shell Task that reported this file."),
	}
}

// Annotations describes additional information for the ent.
func (HostFile) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
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
		SetSize(i uint64)
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
			f.SetSize(uint64(len(content)))

			// Set the new hash (if content exists)
			if len(content) > 0 {
				f.SetHash(fmt.Sprintf("%x", sha3.Sum256(content)))
			}

			return next.Mutate(ctx, m)
		})
	}
}
