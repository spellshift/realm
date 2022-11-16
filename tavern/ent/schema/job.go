package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Job holds the schema definition for the Job entity.
type Job struct {
	ent.Schema
}

// Fields of the Job.
func (Job) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			NotEmpty().
			Unique().
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("Name of the job"),
	}
}

// Edges of the Job.
func (Job) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("createdBy", User.Type).
			Required().
			Unique(),
		edge.To("tome", Tome.Type).
			Required().
			Unique(),
		edge.To("tasks", Task.Type),
	}
}

// Annotations describes additional information for the ent.
func (Job) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.QueryField(),
	}
}
