package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/dialect/entsql"
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
		field.Text("input").
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT",
			}).
			Comment("The command input sent to the shell"),
		field.Text("output").
			Optional().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT",
			}).
			Comment("Any output received from the shell"),
		field.Text("error").
			Optional().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT",
			}).
			Comment("Any error received from the shell"),
		field.String("stream_id").
			Comment("Unique identifier for the stream that created this shell task (likely a websocket uuid)"),
		field.Uint64("sequence_id").
			Comment("Sequence number for ordering tasks within the same stream_id"),
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
		edge.To("reported_credentials", HostCredential.Type).
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Credentials reported by this shell task"),
		edge.To("reported_files", HostFile.Type).
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Files reported by this shell task"),
		edge.To("reported_screenshots", Screenshot.Type).
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Screenshots reported by this shell task"),
		edge.To("reported_processes", HostProcess.Type).
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Processes reported by this shell task"),
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
