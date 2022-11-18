package schema

import (
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

// Mixin defines common shared properties for the ent.
func (File) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // createdAt, lastModifiedAt, createdBy
	}
}

// Hooks of the File.
// func (File) Hooks() []ent.Hook {
// 	return []ent.Hook{
// 		// First hook.
// 		hook.On(
// 			func(next ent.Mutator) ent.Mutator {
// 				return hook.FileFunc(func(ctx context.Context, m *gen.FileMutation) (ent.Value, error) {
// 					// if num, ok := m.Number(); ok && len(num) < 10 {
// 					// 	return nil, fmt.Errorf("card number is too short")
// 					// }
// 					return next.Mutate(ctx, m)
// 				})
// 			},
// 			// Limit the hook only for these operations.
// 			ent.OpCreate|ent.OpUpdate|ent.OpUpdateOne,
// 		),
// 	}
// }
