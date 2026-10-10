package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"realm.pub/tavern/internal/namegen"
)

// Adventure holds the schema definition for the Adventure entity.
type Adventure struct {
	ent.Schema
}

// Fields of the Adventure.
func (Adventure) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			NotEmpty().
			DefaultFunc(namegen.New).
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("Name of the adventure"),
	}
}

// Edges of the Adventure.
func (Adventure) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("quests", Quest.Type).
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput),
				entgql.RelayConnection(),
				entgql.MultiOrder(),
			).
			Comment("Quests that belong to this adventure"),
	}
}

// Annotations describes additional information for the ent.
func (Adventure) Annotations() []schema.Annotation {
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
func (Adventure) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
