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

// Screenshot holds the schema definition for the Screenshot entity.
type Screenshot struct {
	ent.Schema
}

// Fields of the Screenshot.
func (Screenshot) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			NotEmpty().
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("Name of the screenshot file (e.g. screenshot_<hostname>_<timestamp>_<monitor>.png)."),
		field.Uint64("size").
			Default(0).
			Min(0).
			Annotations(
				entgql.Type("Uint64"),
				entgql.OrderField("SIZE"),
			).
			Comment("The size of the screenshot in bytes"),
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
				entgql.Skip(), // Don't return content in GraphQL
			).
			Comment("The content of the screenshot"),
	}
}

// Edges of the Screenshot.
func (Screenshot) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("host", Host.Type).
			Required().
			Unique().
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Host the screenshot was taken on."),
		edge.From("task", Task.Type).
			Unique().
			Ref("screenshots").
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Task that reported this screenshot."),
		edge.From("shell_task", ShellTask.Type).
			Unique().
			Ref("screenshots").
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Shell Task that reported this screenshot."),
	}
}

// Annotations describes additional information for the ent.
func (Screenshot) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}

// Mixin defines common shared properties for the ent.
func (Screenshot) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}

// Hooks defines middleware for mutations for the ent.
func (Screenshot) Hooks() []ent.Hook {
	return []ent.Hook{
		hook.On(HookDeriveScreenshotInfo(), ent.OpCreate|ent.OpUpdate|ent.OpUpdateOne),
	}
}

// HookDeriveScreenshotInfo will update file info (e.g. size, hash) whenever it is mutated.
func HookDeriveScreenshotInfo() ent.Hook {
	type sMutation interface {
		Content() ([]byte, bool)
		SetSize(i uint64)
		SetHash(s string)
	}

	return func(next ent.Mutator) ent.Mutator {
		return ent.MutateFunc(func(ctx context.Context, m ent.Mutation) (ent.Value, error) {
			s, ok := m.(sMutation)
			if !ok {
				return nil, fmt.Errorf("expected screenshot mutation in schema hook, got: %+v", m)
			}

			// Set the new size
			content, _ := s.Content()
			s.SetSize(uint64(len(content)))

			// Set the new hash (if content exists)
			if len(content) > 0 {
				s.SetHash(fmt.Sprintf("%x", sha3.Sum256(content)))
			}

			return next.Mutate(ctx, m)
		})
	}
}
