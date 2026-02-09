package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/field"
	"realm.pub/tavern/internal/c2/c2pb"
)

// Builder holds the schema definition for the Builder entity.
type Builder struct {
	ent.Schema
}

// Fields of the Builder.
func (Builder) Fields() []ent.Field {
	return []ent.Field{
		field.JSON("supported_targets", []c2pb.Host_Platform{}).
			Annotations(
				entgql.Type("[HostPlatform!]"),
			).
			Comment("The platforms this builder can build agents for."),
		field.String("upstream").
			Comment("The server address that the builder should connect to."),
	}
}

// Edges of the Builder.
func (Builder) Edges() []ent.Edge {
	return nil
}

// Annotations describes additional information for the ent.
func (Builder) Annotations() []schema.Annotation {
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
func (Builder) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
