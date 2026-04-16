package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// ShellPivot holds the schema definition for the entity.
type ShellPivot struct {
	ent.Schema
}

// Fields of the ent.
func (ShellPivot) Fields() []ent.Field {
	return []ent.Field{
		field.Time("closed_at").
			Optional().
			Annotations(
				entgql.OrderField("CLOSED_AT"),
				entgql.Skip(entgql.SkipMutationCreateInput),
			).
			Comment("Timestamp of when this pivot was closed"),
		field.String("stream_id").
			NotEmpty().
			Comment("Stream ID of the portal connection"),
		field.Enum("kind").
			Values("ssh", "pty").
			Comment("Kind of pivot connection"),
		field.String("destination").
			NotEmpty().
			Comment("Destination address of the pivot"),
		field.Int("port").
			Comment("Destination port of the pivot"),
		field.String("data").
			Optional().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT",
			}).
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput),
			).
			Comment("Pivot data stream buffer"),
	}
}

// Edges of the ent.
func (ShellPivot) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("shell", Shell.Type).
			Unique().
			Comment("Shell associated with this pivot"),
		edge.To("portal", Portal.Type).
			Unique().
			Comment("Portal associated with this pivot"),
		edge.To("credential", HostCredential.Type).
			Unique().
			Comment("Credential used for this pivot"),
	}
}

// Annotations describes additional information for the ent.
func (ShellPivot) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entgql.Mutations(
			entgql.MutationCreate(),
		),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}

// Mixin defines common shared properties for the ent.
func (ShellPivot) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
