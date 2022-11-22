package schema

import (
	"time"

	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema/field"
	"entgo.io/ent/schema/mixin"
)

// MixinHistory defines common fields and edges for use in ents that require
// tracking of creation and modification details.
type MixinHistory struct {
	mixin.Schema
}

func (MixinHistory) Fields() []ent.Field {
	return []ent.Field{
		field.Time("createdAt").
			Comment("Timestamp of when this ent was created").
			Immutable().
			Default(time.Now).
			Annotations(
				entgql.OrderField("CREATED_AT"),
				entgql.Skip(entgql.SkipMutationCreateInput),
			),
		field.Time("lastModifiedAt").
			Comment("Timestamp of when this ent was last updated").
			Default(time.Now).
			UpdateDefault(time.Now).
			Annotations(
				entgql.OrderField("LAST_MODIFIED_AT"),
				entgql.Skip(entgql.SkipMutationCreateInput),
			),
	}
}
