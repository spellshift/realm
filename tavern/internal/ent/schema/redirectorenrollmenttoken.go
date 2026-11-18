package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/field"
)

// RedirectorEnrollmentToken stores only the hash of a single-use token.
type RedirectorEnrollmentToken struct{ ent.Schema }

func (RedirectorEnrollmentToken) Fields() []ent.Field {
	return []ent.Field{
		field.String("token_hash").Unique().Immutable().Sensitive(),
		field.Bool("used").Default(false),
	}
}

func (RedirectorEnrollmentToken) Annotations() []schema.Annotation {
	return []schema.Annotation{entgql.Skip(entgql.SkipType)}
}
