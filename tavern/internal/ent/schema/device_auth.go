package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// DeviceAuth holds the schema definition for the DeviceAuth entity.
type DeviceAuth struct {
	ent.Schema
}

// Fields of the DeviceAuth.
func (DeviceAuth) Fields() []ent.Field {
	return []ent.Field{
		field.String("user_code").
			NotEmpty().
			Unique().
			Annotations(
				entgql.OrderField("USER_CODE"),
			).
			Comment("Short code displayed to the user for auth."),
		field.String("device_code").
			NotEmpty().
			Unique().
			Sensitive().
			Comment("Long code used by the device to poll for auth status."),
		field.Enum("status").
			Values("PENDING", "APPROVED", "DENIED").
			Default("PENDING").
			Comment("Status of the device authentication request."),
		field.Time("expires_at").
			Comment("When this device auth request expires."),
	}
}

// Edges of the DeviceAuth.
func (DeviceAuth) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("user", User.Type).
			Unique().
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput, entgql.SkipMutationUpdateInput),
			).
			Comment("User who approved the device auth (may be null if pending)."),
	}
}

// Annotations describes additional information for the ent.
func (DeviceAuth) Annotations() []schema.Annotation {
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
func (DeviceAuth) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // createdAt, lastModifiedAt
	}
}
