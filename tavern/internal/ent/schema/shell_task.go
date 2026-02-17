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
			Comment("The output received from the shell"),
		field.Uint64("sequence_id").
			Comment("Sequence number for ordering tasks within a shell"),
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
