package schema

import (
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
				dialect.MySQL: "LONGBLOB",
			}).
			Annotations(
				entgql.Skip(),
			).
			Comment("The content of the screenshot"),
	}
}

// Edges of the Screenshot.
func (Screenshot) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("host", Host.Type).
			Ref("screenshots").
			Required().
			Unique().
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Host the screenshot was taken on."),
		edge.From("task", Task.Type).
			Ref("screenshots").
			Unique().
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Task that reported this screenshot."),
		edge.From("shell_task", ShellTask.Type).
			Ref("screenshots").
			Unique().
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Shell Task that reported this screenshot."),
	}
}

// Mixin of the Screenshot.
func (Screenshot) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{},
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
