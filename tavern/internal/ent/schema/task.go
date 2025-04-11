package schema

import (
	"context"
	"fmt"

	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"realm.pub/tavern/internal/ent/hook"
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
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT", // Override MySQL, improve length maximum
			}).
			Comment("Output from executing the task"),
		field.Int("output_size").
			Default(0).
			Min(0).
			Annotations(
				entgql.OrderField("OUTPUT_SIZE"),
			).
			Comment("The size of the output in bytes"),
		field.String("error").
			Optional().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT", // Override MySQL, improve length maximum
			}).
			Comment("Error, if any, produced while executing the Task"),
	}
}

// Edges of the Task.
func (Task) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("quest", Quest.Type).
			Ref("tasks").
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Required().
			Unique(),
		edge.To("beacon", Beacon.Type).
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Required().
			Unique(),
		edge.To("reported_files", HostFile.Type).
			Comment("Files that have been reported by this task."),
		edge.To("reported_processes", HostProcess.Type).
			Comment("Processes that have been reported by this task."),
		edge.To("reported_credentials", HostCredential.Type).
			Comment("Credentials that have been reported by this task."),
		edge.From("shells", Shell.Type).
			Ref("task").
			Comment("Shells that were created by this task"),
	}
}

// Annotations to configure code generation
func (Task) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}

// Mixin defines common shared properties for the ent.
func (Task) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}

// Hooks defines middleware for mutations for the ent.
func (Task) Hooks() []ent.Hook {
	return []ent.Hook{
		hook.On(HookDeriveTaskInfo(), ent.OpCreate|ent.OpUpdate|ent.OpUpdateOne),
	}
}

// HookDeriveTaskInfo will update task info (e.g. output_size) whenever it is mutated.
func HookDeriveTaskInfo() ent.Hook {
	// Get the relevant methods from the Task Mutation
	// See this example: https://github.com/ent/ent/blob/master/entc/integration/hooks/ent/schema/user.go#L98
	type tMutation interface {
		Output() (string, bool)
		SetOutputSize(i int)
	}

	return func(next ent.Mutator) ent.Mutator {
		return ent.MutateFunc(func(ctx context.Context, m ent.Mutation) (ent.Value, error) {
			// Get the mutation
			t, ok := m.(tMutation)
			if !ok {
				return nil, fmt.Errorf("expected task mutation in schema hook, got: %+v", m)
			}

			// Set the new size
			output, _ := t.Output()
			t.SetOutputSize(len([]byte(output)))

			return next.Mutate(ctx, m)
		})
	}
}
