package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Task holds the schema definition for the Task entity.
type Task struct {
	ent.Schema
}

// Fields of the Task.
func (Task) Fields() []ent.Field {
	return []ent.Field{
		field.Time("claimed_at").
			Optional().
			Annotations(
				entgql.OrderField("CLAIMED_AT"),
			).
			Comment("Timestamp of when the task was claimed, null if not yet claimed"),
		field.Time("exec_started_at").
			Optional().
			Annotations(
				entgql.OrderField("EXEC_STARTED_AT"),
			).
			Comment("Timestamp of when execution of the task started, null if not yet started"),
		field.Time("exec_finished_at").
			Optional().
			Annotations(
				entgql.OrderField("EXEC_FINISHED_AT"),
			).
			Comment("Timestamp of when execution of the task finished, null if not yet finished"),
		field.Text("output").
			Optional().
			Comment("Output from executing the task"),
		field.String("error").
			Optional().
			Comment("Error, if any, produced while executing the Task"),
	}
}

// Edges of the Task.
func (Task) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("quest", Quest.Type).
			Ref("tasks").
			Required().
			Unique(),
		edge.To("beacon", Beacon.Type).
			Required().
			Unique(),
	}
}

// Annotations to configure code generation
func (Task) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
	}
}

// Mixin defines common shared properties for the ent.
func (Task) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
