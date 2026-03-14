package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"realm.pub/tavern/internal/ent/schema/validators"
)

// ScheduledTask holds the schema definition for the ScheduledTask entity.
type ScheduledTask struct {
	ent.Schema
}

// Fields of the ScheduledTask.
func (ScheduledTask) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			NotEmpty().
			Unique().
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("Name of the scheduled task"),
		field.String("description").
			Comment("Information about the scheduled task"),
		field.Bool("run_on_new_beacon_callback").
			Default(false).
			Comment("If true, this tome will automatically be queued for all new Beacon callbacks."),
		field.Bool("run_on_first_host_callback").
			Default(false).
			Comment("If true, this tome will automatically be queued for the first new callback on a Host."),
		field.String("parameters").
			Validate(validators.NewJSONStringString()).
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT",
			}).
			Optional().
			Comment("Value of parameters that will be used when creating quests from this scheduled task (as a JSON string)."),
		field.String("run_on_schedule").
			Default("").
			Comment("Cron-like schedule for this tome to be automatically queued."),
	}
}

// Edges of the ScheduledTask.
func (ScheduledTask) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("tome", Tome.Type).
			Unique().
			Required().
			Annotations(
				entgql.Skip(entgql.SkipMutationUpdateInput),
			).
			Comment("The Tome that this ScheduledTask will run."),
		edge.To("scheduled_hosts", Host.Type).
			Annotations(
				entgql.RelayConnection(),
				entgql.MultiOrder(),
			).
			Comment("If a schedule is configured for this tome, you may limit which hosts it runs on using this field. If a schedule is configured but no hosts are set, the tome will run on all hosts."),
	}
}

// Annotations describes additional information for the ent.
func (ScheduledTask) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entgql.Mutations(
			entgql.MutationCreate(),
			entgql.MutationUpdate(),
		),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}

// Mixin defines common shared properties for the ent.
func (ScheduledTask) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // createdAt, lastModifiedAt
	}
}

// Hooks defines middleware for mutations for the ent.
func (ScheduledTask) Hooks() []ent.Hook {
	return []ent.Hook{}
}
