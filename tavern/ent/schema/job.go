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
		field.String("parameters").
			Optional().
			Comment("Value of parameters that were specified for the job (as a JSON string)."),
	}
}

// Edges of the Job.
func (Job) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("tome", Tome.Type).
			Comment("Tome that this job will be executing").
			Required().
			Unique(),
		edge.To("bundle", File.Type).
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput),
			).
			Comment("Bundle file that the executing tome depends on (if any)").
			Unique(),
		edge.To("tasks", Task.Type).
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput),
			).
			Comment("Tasks tracking the status and output of individual tome execution on targets"),
		edge.To("creator", User.Type).
			Unique().
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput),
			).
			Comment("User that created the job if available."),
	}
}

// Annotations describes additional information for the ent.
func (Job) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.Mutations(
			entgql.MutationCreate(),
		),
	}
}

// Mixin defines common shared properties for the ent.
func (Job) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // createdAt, lastModifiedAt
	}
}
