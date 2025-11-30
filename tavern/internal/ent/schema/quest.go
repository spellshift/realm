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

// Quest holds the schema definition for the Quest entity.
type Quest struct {
	ent.Schema
}

// Fields of the Quest.
func (Quest) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			NotEmpty().
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("Name of the quest"),
		field.String("parameters").
			Validate(validators.NewJSONStringString()).
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT", // Override MySQL, improve length maximum
			}).
			Optional().
			Comment("Value of parameters that were specified for the quest (as a JSON string)."),
		field.String("param_defs_at_creation").
			Immutable().
			Validate(validators.NewTomeParameterDefinitions()).
			Optional().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT", // Override MySQL, improve length maximum
			}).
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput),
			).
			Comment("JSON string describing what parameters are used with the tome at the time of this quest creation. Requires a list of JSON objects, one for each parameter."),
		field.String("eldritch_at_creation").
			Immutable().
			Optional().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT", // Override MySQL, improve length maximum
			}).
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput),
			).
			Comment("Eldritch script that was evaluated at the time of this quest creation."),
	}
}

// Edges of the Quest.
func (Quest) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("tome", Tome.Type).
			Comment("Tome that this quest will be executing").
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
				entgql.RelayConnection(),
				entgql.MultiOrder(),
			).
			Comment("Tasks tracking the status and output of individual tome execution on targets"),
		edge.To("creator", User.Type).
			Unique().
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput),
			).
			Comment("User that created the quest if available."),
	}
}

// Annotations describes additional information for the ent.
func (Quest) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entgql.Mutations(
			entgql.MutationCreate(),
		),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}

// Mixin defines common shared properties for the ent.
func (Quest) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
