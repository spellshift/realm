package schema

import (
	"time"

	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/field"
)

// File holds the schema definition for the File entity.
type File struct {
	ent.Schema
}

// Fields of the File.
func (File) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			NotEmpty().
			Unique().
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("The name of the file, used to reference it for downloads"),
		field.Int("size").
			Default(0).
			Min(0).
			Annotations(
				entgql.OrderField("SIZE"),
			).
			Comment("The size of the file in bytes"),
		field.String("hash").
			MaxLen(100).
			Comment("A SHA3 digest of the content field"),
		field.Time("createdAt").
			Default(time.Now).
			Annotations(
				entgql.OrderField("CREATED_AT"),
			).
			Comment("The timestamp for when the File was created"),
		field.Time("lastModifiedAt").
			Default(time.Now).
			Annotations(
				entgql.OrderField("LAST_MODIFIED_AT"),
			).
			Comment("The timestamp for when the File was last modified"),

		field.Bytes("content").
			Annotations(
				entgql.Skip(), // Don't return file content in GraphQL queries
			).
			Comment("The content of the file"),
	}
}

// Edges of the File.
func (File) Edges() []ent.Edge {
	return []ent.Edge{}
}

// Annotations describes additional information for the ent.
func (File) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.QueryField(),
	}
}
