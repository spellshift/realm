package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// BeaconHistory holds the schema definition for the BeaconHistory entity.
type BeaconHistory struct {
	ent.Schema
}

// Fields of the BeaconHistory.
func (BeaconHistory) Fields() []ent.Field {
	return []ent.Field{
		field.Int64("latency").
			Comment("The number of milliseconds off of its expected callback time. Can be negative if it checked-in ahead of schedule."),
	}
}

// Edges of the BeaconHistory.
func (BeaconHistory) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("beacon", Beacon.Type).
			Required().
			Unique().
			Comment("The beacon this history belongs to."),
	}
}

// Annotations describes additional information for the ent.
func (BeaconHistory) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entgql.Mutations(
			entgql.MutationCreate(),
		),
	}
}

// Mixin defines common shared properties for the ent.
func (BeaconHistory) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
