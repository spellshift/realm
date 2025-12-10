package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"realm.pub/tavern/internal/c2/epb"
)

// HostCredential holds the schema definition for the entity.
type HostCredential struct {
	ent.Schema
}

// Fields of the ent.
func (HostCredential) Fields() []ent.Field {
	return []ent.Field{
		field.String("principal").
			NotEmpty().
			Annotations(
				entgql.OrderField("PRINCIPAL"),
			).
			Comment("Identity associated with this credential (e.g. username)."),
		field.String("secret").
			NotEmpty().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT", // Override MySQL, improve length maximum
			}).
			Comment("Secret for this credential (e.g. password)."),
		field.Enum("kind").
			GoType(epb.Credential_Kind(0)).
			Comment("Kind of credential."),
	}
}

// Edges of the ent.
func (HostCredential) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("host", Host.Type).
			Required().
			Unique().
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Host the credential was reported on."),
		edge.From("task", Task.Type).
			Unique().
			Ref("reported_credentials").
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Task that reported this credential."),
	}
}

// Annotations describes additional information for the ent.
func (HostCredential) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entgql.Mutations(entgql.MutationCreate()),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}

// Mixin defines common shared properties for the ent.
func (HostCredential) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}

// Hooks defines middleware for mutations for the ent.
func (HostCredential) Hooks() []ent.Hook {
	return []ent.Hook{}
}
