package schema

import (
	"time"

	"github.com/google/uuid"
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Link holds the schema definition for the Link entity.
type Link struct {
	ent.Schema
}

// Fields of the Link.
func (Link) Fields() []ent.Field {
	return []ent.Field{
		field.String("path").
			NotEmpty().
			Unique().
			DefaultFunc(func() string {
				return uuid.New().String()
			}).
			Annotations(
				entgql.OrderField("PATH"),
			).
			Comment("Unique path for accessing the file via the CDN"),
		field.Time("active_before").
			Default(time.Unix(0, 0)).
			Annotations(
				entgql.OrderField("ACTIVE_BEFORE"),
			).
			Comment("Timestamp before which the link is active. Default is epoch 0"),
		field.Int("active_clicks").
			Default(0).
			Min(0).
			Annotations(
				entgql.OrderField("ACTIVE_CLICKS"),
			).
			Comment("Number of times this link can be clicked before it becomes inactive"),
	}
}

// Edges of the Link.
func (Link) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("file", File.Type).
			Unique().
			Required().
			Annotations(
				entgql.Skip(entgql.SkipMutationUpdateInput), // Don't allow changing the file after creation
			).
			Comment("The file that this link points to"),
	}
}

// Annotations describes additional information for the ent.
func (Link) Annotations() []schema.Annotation {
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
func (Link) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
