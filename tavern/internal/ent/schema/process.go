package schema

import (
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/field"
)

// Process holds the schema definition for the Process entity.
type Process struct {
	ent.Schema
}

// Fields of the ent.
func (Process) Fields() []ent.Field {
	return []ent.Field{
		field.Uint64("pid").
			Comment("ID of the process."),
		field.String("name").
			Comment("The name of the process."),
		field.String("principal").
			NotEmpty().
			Comment("The user the process is running as."),
	}
}

// Edges of the ent.
func (Process) Edges() []ent.Edge {
	return []ent.Edge{
		// edge.To("host", Host.Type).
		// 	Required().
		// 	Unique().
		// 	Comment("Host the process was reported on."),
		// edge.From("task", Task.Type).
		// 	Required().
		// 	Unique().
		// 	Ref("reported_processes").
		// 	Comment("Task that reported this process."),
	}
}

// Annotations describes additional information for the ent.
func (Process) Annotations() []schema.Annotation {
	return []schema.Annotation{
		// entgql.Skip(entgql.SkipMutationCreateInput, entgql.SkipMutationUpdateInput),
	}
}

// Mixin defines common shared properties for the ent.
func (Process) Mixin() []ent.Mixin {
	return []ent.Mixin{
		// MixinHistory{}, // created_at, last_modified_at
	}
}
