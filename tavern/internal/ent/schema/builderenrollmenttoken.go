package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/field"
)

// BuilderEnrollmentToken stores a one time enrollment token hash.
type BuilderEnrollmentToken struct{ ent.Schema }

func (BuilderEnrollmentToken) Fields() []ent.Field {
	return []ent.Field{
		field.String("token_hash").Unique().Immutable().Sensitive(),
		field.Bool("used").Default(false),
	}
}

// Annotations describes additional information for the ent. The entire type
// is skipped from the GraphQL schema: tokens are redeemed through the
// dedicated exchangeBuilderEnrollmentToken mutation, not through node/nodes.
func (BuilderEnrollmentToken) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.Skip(entgql.SkipType),
	}
}
