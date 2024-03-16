package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Shell holds the schema definition for the entity.
type Shell struct {
	ent.Schema
}

// Fields of the ent.
func (Shell) Fields() []ent.Field {
	return []ent.Field{
		field.Time("closed_at").
			Optional().
			Annotations(
				entgql.OrderField("CLOSED_AT"),
				entgql.Skip(entgql.SkipMutationCreateInput),
			).
			Comment("Timestamp of when this shell was closed"),
		field.Bytes("data").
			SchemaType(map[string]string{
				dialect.MySQL: "LONGBLOB", // Override MySQL, improve length maximum
			}).
			Annotations(
				entgql.Skip(), // Don't return in GraphQL queries
			).
			Comment("Shell data stream"),
	}
}

// Edges of the ent.
func (Shell) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("task", Task.Type).
			Unique().
			Required().
			Comment("Task that created the shell"),
		edge.To("beacon", Beacon.Type).
			Unique().
			Required().
			Comment("Beacon that created the shell"),
		edge.To("owner", User.Type).
			Unique().
			Required().
			Comment("User that created the shell"),
		edge.To("active_users", User.Type).
			Comment("Users that are currently using the shell"),
	}
}

// Annotations describes additional information for the ent.
func (Shell) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
	}
}

// Mixin defines common shared properties for the ent.
func (Shell) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
