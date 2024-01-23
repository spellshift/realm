package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Host holds the schema definition for the Host entity.
type Host struct {
	ent.Schema
}

// Fields of the Host.
func (Host) Fields() []ent.Field {
	return []ent.Field{
		field.String("identifier").
			NotEmpty().
			Unique().
			Annotations(
				entgql.Skip(entgql.SkipMutationUpdateInput),
			).
			Comment("Unique identifier for the host. Unique to each host."),
		field.String("name").
			Optional().
			NotEmpty().
			Comment("A human readable identifier for the host."),
		field.String("primary_ip").
			Optional().
			Annotations(
				entgql.Skip(entgql.SkipMutationUpdateInput),
			).
			Comment("Primary interface IP address reported by the agent."),
		field.Enum("platform").
			Values("Windows", "Linux", "MacOS", "BSD", "Unknown").
			Default("Unknown").
			Annotations(
				entgql.Skip(entgql.SkipMutationUpdateInput),
			).
			Comment("Platform the agent is operating on."),
		field.Time("last_seen_at").
			Optional().
			Annotations(
				entgql.OrderField("LAST_SEEN_AT"),
				entgql.Skip(entgql.SkipMutationUpdateInput),
			).
			Comment("Timestamp of when a task was last claimed or updated for the host."),
	}
}

// Edges of the Host.
func (Host) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("tags", Tag.Type).
			Comment("Tags used to group this host with other hosts."),
		edge.From("beacons", Beacon.Type).
			Ref("host").
			Comment("Beacons that are present on this host system."),
		edge.To("processes", Process.Type).
			Comment("Processes reported as running on this host system."),
	}
}

// Annotations describes additional information for the ent.
func (Host) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.Mutations(
			entgql.MutationUpdate(),
		),
	}
}

// Mixin defines common shared properties for the ent.
func (Host) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
