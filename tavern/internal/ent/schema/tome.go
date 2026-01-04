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
	"golang.org/x/crypto/sha3"
	"realm.pub/tavern/internal/ent/hook"
	"realm.pub/tavern/internal/ent/schema/validators"
)

// Tome holds the schema definition for the Tome entity.
type Tome struct {
	ent.Schema
}

// Fields of the Tome.
func (Tome) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			NotEmpty().
			Unique().
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("Name of the tome"),
		field.String("description").
			Comment("Information about the tome"),
		field.String("author").
			Comment("Name of the author who created the tome."),
		field.Enum("support_model").
			Values("UNSPECIFIED", "FIRST_PARTY", "COMMUNITY").
			Default("UNSPECIFIED").
			Comment("Information about the tomes support model."),
		field.Enum("tactic").
			Values(
				"UNSPECIFIED",
				"RECON",
				"RESOURCE_DEVELOPMENT",
				"INITIAL_ACCESS",
				"EXECUTION",
				"PERSISTENCE",
				"PRIVILEGE_ESCALATION",
				"DEFENSE_EVASION",
				"CREDENTIAL_ACCESS",
				"DISCOVERY",
				"LATERAL_MOVEMENT",
				"COLLECTION",
				"COMMAND_AND_CONTROL",
				"EXFILTRATION",
				"IMPACT",
			).
			Default("UNSPECIFIED").
			Comment("MITRE ATT&CK tactic provided by the tome."),
		field.Bool("run_on_new_beacon_callback").
			Default(false).
			Comment("If true, this tome will automatically be queued for all new Beacon callbacks."),
		field.Bool("run_on_first_host_callback").
			Default(false).
			Comment("If true, this tome will automatically be queued for the first new callback on a Host."),
		field.String("run_on_schedule").
			// Validate() // TODO: Cron schedule validation
			Default("").
			Comment("Cron-like schedule for this tome to be automatically queued."),
		field.String("param_defs").
			Validate(validators.NewTomeParameterDefinitions()).
			Optional().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT", // Override MySQL, improve length maximum
			}).
			Comment("JSON string describing what parameters are used with the tome. Requires a list of JSON objects, one for each parameter."),
		field.String("hash").
			MaxLen(100).
			Annotations(
				entgql.Skip(entgql.SkipAll),
			).
			Comment("A SHA3 digest of the eldritch field"),
		field.String("eldritch").
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT", // Override MySQL, improve length maximum
			}).
			Comment("Eldritch script that will be executed when the tome is run"),
	}
}

// Edges of the Tome.
func (Tome) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("files", File.Type).
			Annotations(
				entgql.RelayConnection(),
				entgql.MultiOrder(),
			).
			Comment("Any files required for tome execution that will be bundled and provided to the agent for download"),
		edge.To("uploader", User.Type).
			Unique().
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput, entgql.SkipMutationUpdateInput),
			).
			Comment("User who uploaded the tome (may be null)."),
		edge.To("repository", Repository.Type).
			Unique().
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput, entgql.SkipMutationUpdateInput),
			).
			Comment("Repository from which this Tome was imported (may be null)."),
		edge.To("scheduled_hosts", Host.Type).
			Annotations(
				entgql.RelayConnection(),
				entgql.MultiOrder(),
			).
			Comment("If a schedule is configured for this tome, you may limit which hosts it runs on using this field. If a schedule is configured but no hosts are set, the tome will run on all hosts."),
	}
}

// Annotations describes additional information for the ent.
func (Tome) Annotations() []schema.Annotation {
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
func (Tome) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // createdAt, lastModifiedAt
	}
}

// Hooks defines middleware for mutations for the ent.
func (Tome) Hooks() []ent.Hook {
	return []ent.Hook{
		hook.On(HookDeriveTomeInfo(), ent.OpCreate|ent.OpUpdate|ent.OpUpdateOne),
	}
}

// HookDeriveTomeInfo will update tome info (e.g. hash) whenever it is mutated.
func HookDeriveTomeInfo() ent.Hook {
	// Get the relevant methods from the Tome Mutation
	// See this example: https://github.com/ent/ent/blob/master/entc/integration/hooks/ent/schema/user.go#L98
	type tMutation interface {
		Eldritch() (string, bool)
		SetHash(s string)
	}

	return func(next ent.Mutator) ent.Mutator {
		return ent.MutateFunc(func(ctx context.Context, m ent.Mutation) (ent.Value, error) {
			// Get the tome mutation
			t, ok := m.(tMutation)
			if !ok {
				return nil, fmt.Errorf("expected tome mutation in schema hook, got: %+v", m)
			}

			// Set the new hash
			eldritch, _ := t.Eldritch()
			t.SetHash(fmt.Sprintf("%x", sha3.Sum256([]byte(eldritch))))

			return next.Mutate(ctx, m)
		})
	}
}
