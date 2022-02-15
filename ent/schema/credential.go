package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Credential holds the schema definition for the Credential entity.
type Credential struct {
	ent.Schema
}

// Fields of the Credential.
func (Credential) Fields() []ent.Field {
	return []ent.Field{
		field.String("principal").
			Comment("The principal for the Credential (e.g. username)").
			NotEmpty().
			Annotations(entgql.OrderField("PRINCIPAL")),
		field.Text("secret").
			Comment("The secret for the Credential (e.g. password)").
			NotEmpty().
			MaxLen(3000).
			Annotations(entgql.OrderField("SECRET")),
		field.Enum("kind").
			Comment("The kind of the credential (password, key, etc)").
			NamedValues(
				"Password", "PASSWORD",
				"Key", "KEY",
				"Certificate", "CERTIFICATE",
			).
			Annotations(entgql.OrderField("KIND")),
	}
}

// Edges of the Credential.
func (Credential) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("target", Target.Type).
			Comment("The target that this credential was created for").
			Ref("credentials").
			Annotations(entgql.MapsTo("target")).
			Unique(),
	}
}
