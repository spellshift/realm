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

// HostFact holds the schema definition for the entity.
type HostFact struct {
	ent.Schema
}

// Fields of the ent.
func (HostFact) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			NotEmpty().
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("Name of the fact."),
		field.String("value").
			NotEmpty().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT", // Override MySQL, improve length maximum
			}).
			Comment("Value of the fact."),
	}
}

// Edges of the ent.
func (HostFact) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("host", Host.Type).
			Required().
			Unique().
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Host the fact was reported on."),
		edge.From("task", Task.Type).
			Unique().
			Ref("reported_facts").
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Task that reported this fact."),
	}
}

// Annotations describes additional information for the ent.
func (HostFact) Annotations() []schema.Annotation {
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
func (HostFact) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}

// Hooks defines middleware for mutations for the ent.
func (HostFact) Hooks() []ent.Hook {
	return []ent.Hook{}
}
