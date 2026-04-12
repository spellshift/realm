package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Event holds the schema definition for the Event entity.
type Event struct {
	ent.Schema
}

// Fields of the Event.
func (Event) Fields() []ent.Field {
	return []ent.Field{
		field.Int64("timestamp").
			Comment("Unix timestamp of the event").
			Annotations(
				entgql.OrderField("TIMESTAMP"),
			),
		field.Enum("kind").
			Values(
				"BEACON_LOST",
				"HOST_ACCESS_NEW",
				"HOST_ACCESS_RECOVERED",
				"HOST_ACCESS_LOST",
				"QUEST_COMPLETED",
			).
			Comment("Type of event"),
	}
}

// Edges of the Event.
func (Event) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("beacon", Beacon.Type).
			Ref("events").
			Unique().
			Comment("Beacon associated with this event"),
		edge.From("host", Host.Type).
			Ref("events").
			Unique().
			Comment("Host associated with this event"),
		edge.From("quest", Quest.Type).
			Ref("events").
			Unique().
			Comment("Quest associated with this event"),
		edge.From("notifications", Notification.Type).
			Ref("event").
			Annotations(
				entgql.RelayConnection(),
				entgql.MultiOrder(),
			).
			Comment("Notifications related to this event"),
	}
}

// Annotations describes additional information for the ent.
func (Event) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}

// Mixin defines common shared properties for the ent.
func (Event) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
