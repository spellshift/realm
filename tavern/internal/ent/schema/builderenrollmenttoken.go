package schema

import (
	"entgo.io/ent"
	"entgo.io/ent/schema/field"
)

// BuilderEnrollmentToken stores a one time enrollment token hash.
type BuilderEnrollmentToken struct{ ent.Schema }

func (BuilderEnrollmentToken) Fields() []ent.Field {
	return []ent.Field{
		field.String("token_hash").Unique().Immutable(),
		field.Bool("used").Default(false),
	}
}
