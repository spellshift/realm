package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// OperatorNote holds the schema definition for the OperatorNote entity.
type OperatorNote struct {
	ent.Schema
}

// Fields of the OperatorNote.
func (OperatorNote) Fields() []ent.Field {
	return []ent.Field{
		field.Text("content").
			Optional().
			Default("").
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT",
			}).
			Comment("Content of the operator note."),
	}
}

// Edges of the OperatorNote.
func (OperatorNote) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("author", User.Type).
			Unique().
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput, entgql.SkipMutationUpdateInput),
			).
			Comment("User who authored the note."),
		edge.To("shells", Shell.Type).
			Annotations(
				entgql.RelayConnection(),
				entgql.MultiOrder(),
			).
			Comment("Shells from which this note has been modified."),
	}
}

// Annotations describes additional information for the ent.
func (OperatorNote) Annotations() []schema.Annotation {
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
func (OperatorNote) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}

