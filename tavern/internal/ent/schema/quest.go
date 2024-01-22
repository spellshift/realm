package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
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
		entgql.Mutations(
			entgql.MutationCreate(),
		),
	}
}

// Mixin defines common shared properties for the ent.
func (Quest) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
