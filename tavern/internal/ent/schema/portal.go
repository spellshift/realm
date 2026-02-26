package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Portal holds the schema definition for the entity.
type Portal struct {
	ent.Schema
}

// Fields of the ent.
func (Portal) Fields() []ent.Field {
	return []ent.Field{
		field.Time("closed_at").
			Optional().
			Annotations(
				entgql.OrderField("CLOSED_AT"),
				entgql.Skip(entgql.SkipMutationCreateInput),
			).
			Comment("Timestamp of when this portal was closed"),
	}
}

// Edges of the ent.
func (Portal) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("task", Task.Type).
			Unique().
			Comment("Task that created the portal"),
		edge.To("shell_task", ShellTask.Type).
			Unique().
			Comment("ShellTask that created the portal"),
		edge.To("beacon", Beacon.Type).
			Unique().
			Required().
			Comment("Beacon that created the portal"),
		edge.To("owner", User.Type).
			Unique().
			Required().
			Comment("User that created the portal"),
		edge.To("active_users", User.Type).
			Annotations(
				entgql.RelayConnection(),
				entgql.MultiOrder(),
			).
			Comment("Users that are currently using the portal"),
	}
}

// Annotations describes additional information for the ent.
func (Portal) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}

// Mixin defines common shared properties for the ent.
func (Portal) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
