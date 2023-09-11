package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Tag holds the schema definition for the Tag entity.
type Tag struct {
	ent.Schema
}

// Fields of the Tag.
func (Tag) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			NotEmpty().
			Unique().
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("Name of the tag"),
		field.Enum("kind").
			Values("group", "service").
			Comment("Describes the type of tag this is"),
	}
}

// Edges of the Tag.
func (Tag) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("beacons", Beacon.Type).
			Ref("tags"),
	}
}

// Annotations describes additional information for the ent.
func (Tag) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.Mutations(
			entgql.MutationCreate(),
			entgql.MutationUpdate(),
		),
	}
}
