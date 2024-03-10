package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/field"
)

// Shell holds the schema definition for the entity.
type Shell struct {
	ent.Schema
}

// Fields of the ent.
func (Shell) Fields() []ent.Field {
	return []ent.Field{
		field.Bytes("input").
			SchemaType(map[string]string{
				dialect.MySQL: "LONGBLOB", // Override MySQL, improve length maximum
			}).
			Annotations(
				entgql.Skip(), // Don't return in GraphQL queries
			).
			Comment("Shell input stream"),
		field.Bytes("output").
			SchemaType(map[string]string{
				dialect.MySQL: "LONGBLOB", // Override MySQL, improve length maximum
			}).
			Annotations(
				entgql.Skip(), // Don't return in GraphQL queries
			).
			Comment("Shell output stream"),
	}
}

// Edges of the ent.
func (Shell) Edges() []ent.Edge {
	return []ent.Edge{}
}

// Annotations describes additional information for the ent.
func (Shell) Annotations() []schema.Annotation {
	return []schema.Annotation{}
}

// Mixin defines common shared properties for the ent.
func (Shell) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
