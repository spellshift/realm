package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// NetworkHost holds the schema definition for the NetworkHost entity.
type NetworkHost struct {
	ent.Schema
}

// Fields of the NetworkHost.
func (NetworkHost) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			Optional().
			Default("").
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("Human readable name for the network host."),
		field.String("hostname").
			Optional().
			Default("").
			Annotations(
				entgql.OrderField("HOSTNAME"),
			).
			Comment("Hostname of the network host."),
		field.String("ip_address").
			NotEmpty().
			Annotations(
				entgql.OrderField("IP_ADDRESS"),
			).
			Comment("IP address of the network host."),
		field.String("subnet").
			Optional().
			Default("").
			Annotations(
				entgql.OrderField("SUBNET"),
			).
			Comment("Subnet CIDR of the network host (e.g. 10.0.0.0/24)."),
		field.Ints("open_ports").
			Optional().
			Comment("List of open ports discovered on the network host."),
	}
}

// Edges of the NetworkHost.
func (NetworkHost) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("host", Host.Type).
			Unique().
			Comment("Optional link to the detailed Host entity."),
		edge.To("networks", Network.Type).
			Annotations(
				entgql.RelayConnection(),
				entgql.MultiOrder(),
			).
			Comment("Networks that this network host belongs to."),
		edge.To("credentials", HostCredential.Type).
			Annotations(
				entgql.RelayConnection(),
				entgql.MultiOrder(),
			).
			Comment("Credentials associated with this network host."),
	}
}

// Annotations describes additional information for the ent.
func (NetworkHost) Annotations() []schema.Annotation {
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
func (NetworkHost) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}

