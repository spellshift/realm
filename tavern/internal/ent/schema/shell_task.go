package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// ShellTask holds the schema definition for the ShellTask entity.
type ShellTask struct {
	ent.Schema
}

// Fields of the ShellTask.
func (ShellTask) Fields() []ent.Field {
	return []ent.Field{
		field.String("input").
			Comment("The command input sent to the shell"),
		field.String("output").
			Optional().
			Comment("Any output received from the shell"),
		field.String("error").
			Optional().
			Comment("Any error received from the shell"),
		field.String("stream_id").
			Comment("Unique identifier for the stream that created this shell task (likely a websocket uuid)"),
		field.Uint64("sequence_id").
			Comment("Sequence number for ordering tasks within the same stream_id"),
	}
}

// Edges of the ShellTask.
func (ShellTask) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("shell", Shell.Type).
			Ref("shell_tasks").
			Unique().
			Required().
			Comment("The shell this task belongs to"),
		edge.To("creator", User.Type).
			Unique().
			Required().
			Comment("The user who created this ShellTask"),
	}
}

// Mixin of the ShellTask.
func (ShellTask) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{},
	}
}

// Annotations of the ShellTask.
func (ShellTask) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
	}
}
