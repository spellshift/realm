package schema

import (
	"time"

	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/field"
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
		field.String("parameters").
			Comment("JSON string describing what parameters are used with the tome"),
		field.Int("size").
			Default(0).
			Min(0).
			Annotations(
				entgql.OrderField("SIZE"),
			).
			Comment("The size of the tome in bytes"),
		field.String("hash").
			MaxLen(100).
			Comment("A SHA3 digest of the content field"),
		field.Time("createdAt").
			Default(time.Now).
			Annotations(
				entgql.OrderField("CREATED_AT"),
			).
			Comment("The timestamp for when the Tome was created"),
		field.Time("lastModifiedAt").
			Default(time.Now).
			Annotations(
				entgql.OrderField("LAST_MODIFIED_AT"),
			).
			Comment("The timestamp for when the Tome was last modified"),
		field.Bytes("content").
			Annotations(
				entgql.Skip(), // Don't return tome content in GraphQL queries
			).
			Comment("The content of the tome"),
	}
}

// Edges of the Tome.
func (Tome) Edges() []ent.Edge {
	return []ent.Edge{}
}

// Annotations describes additional information for the ent.
func (Tome) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.QueryField(),
	}
}
