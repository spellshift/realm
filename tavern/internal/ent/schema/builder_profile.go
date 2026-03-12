package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/field"
	"realm.pub/tavern/internal/builder/builderpb"
)

// BuilderProfile holds the schema definition for the BuilderProfile entity.
type BuilderProfile struct {
	ent.Schema
}

// Fields of the BuilderProfile.
func (BuilderProfile) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			NotEmpty().
			Unique().
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("Name of the builder profile."),
		field.String("description").
			Optional().
			Comment("Description of the builder profile."),
		field.String("pre_build_script").
			Optional().
			Comment("Bash script to run before compilation."),
		field.String("post_build_script").
			Optional().
			Comment("Bash script to run after build is complete."),
		field.JSON("transports", []builderpb.BuildTaskTransport{}).
			Optional().
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput, entgql.SkipMutationUpdateInput),
			).
			Comment("List of transport configurations for the IMIX agent."),
	}
}

// Edges of the BuilderProfile.
func (BuilderProfile) Edges() []ent.Edge {
	return []ent.Edge{}
}

// Annotations describes additional information for the ent.
func (BuilderProfile) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.QueryField(),
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
func (BuilderProfile) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
