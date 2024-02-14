package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"realm.pub/tavern/internal/c2/epb"
)

// HostProcess holds the schema definition for the HostProcess entity.
type HostProcess struct {
	ent.Schema
}

// Fields of the ent.
func (HostProcess) Fields() []ent.Field {
	return []ent.Field{
		field.Uint64("pid").
			Annotations(
				entgql.Type("Uint64"),
				entgql.OrderField("PROCESS_ID"),
			).
			Comment("ID of the process."),
		field.Uint64("ppid").
			Annotations(
				entgql.Type("Uint64"),
				entgql.OrderField("PARENT_PROCESS_ID"),
			).
			Comment("ID of the parent process."),
		field.String("name").
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("The name of the process."),
		field.String("principal").
			NotEmpty().
			Comment("The user the process is running as."),
		field.String("path").
			Optional().
			Comment("The path to the process executable."),
		field.String("cmd").
			Optional().
			Comment("The command used to execute the process."),
		field.String("env").
			Optional().
			Comment("The environment variables set for the process."),
		field.String("cwd").
			Optional().
			Comment("The current working directory for the process."),
		field.Enum("status").
			GoType(epb.Process_Status(0)).
			Comment("Current process status."),
	}
}

// Edges of the ent.
func (HostProcess) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("host", Host.Type).
			Required().
			Unique().
			Comment("Host the process was reported on."),
		edge.From("task", Task.Type).
			Required().
			Unique().
			Ref("reported_processes").
			Comment("Task that reported this process."),
	}
}

// Annotations describes additional information for the ent.
func (HostProcess) Annotations() []schema.Annotation {
	return []schema.Annotation{}
}

// Mixin defines common shared properties for the ent.
func (HostProcess) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
