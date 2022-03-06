package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Target holds the schema definition for the Target entity.
type Target struct {
	ent.Schema
}

// Fields of the Target.
func (Target) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			Unique().
			NotEmpty().
			MinLen(3).
			MaxLen(50).
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("A human readable identifier for the target system."),
		field.String("forwardConnectIP").
			Unique().
			NotEmpty().
			Annotations(
				entgql.OrderField("FORWARD_CONNECT_IP"),
			).
			Comment("The IP Address that can be used to connect to the target using a protocol like SSH."),
	}
}

// Edges of the Target.
func (Target) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("credentials", Credential.Type).
			Annotations(entgql.Bind()).
			Comment("A Target can have many credentials connected to it"),
		edge.From("implants", Implant.Type).
			Ref("target").
			Comment("Implants that are (or will be) installed on the target."),
		edge.From("deployments", Deployment.Type).
			Ref("target").
			Comment("Deployments that occurred on this target."),
	}
}
