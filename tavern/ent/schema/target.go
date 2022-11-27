package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Target holds the schema definition for the Target entity.
type Target struct {
	ent.Schema
}

// Fields of the Target.
func (Target) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			NotEmpty().
			Unique().
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("Human-readable name of the target"),
		field.Time("lastSeenAt").
			Optional().
			Annotations(
				entgql.OrderField("LAST_SEEN_AT"),
			).
			Comment("Timestamp of when a task was last claimed or updated for a target"),
	}
}

// Edges of the Target.
func (Target) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("tags", Tag.Type),
	}
}

// Annotations describes additional information for the ent.
func (Target) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.QueryField(),
	}
}
