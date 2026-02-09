package schema

import (
	"math/rand"
	"time"

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

func randomShortPath() string {
	const charset = "abcdefghijklmnopqrstuvwxyz0123456789"
	const length = 6
	result := make([]byte, length)
	for i := range result {
		result[i] = charset[rand.Intn(len(charset))]
	}
	return string(result)
}

// Fields of the Link.
func (Link) Fields() []ent.Field {
	return []ent.Field{
		field.String("path").
			NotEmpty().
			Unique().
			DefaultFunc(randomShortPath).
			Annotations(
				entgql.OrderField("PATH"),
			).
			Comment("Unique path for accessing the asset via the CDN"),
		field.Time("expires_at").
			Default(time.Now().Add(24 * time.Hour)).
			Annotations(
				entgql.OrderField("EXPIRES_AT"),
			).
			Comment("Timestamp before which the link is active. Default is MySQL minimum datetime (1000-01-01)"),
		field.Int("download_limit").
			Optional().
			Nillable().
			Min(0).
			Annotations(
				entgql.OrderField("DOWNLOAD_LIMIT"),
			).
			Comment("Maximum number of times this link can be clicked before it becomes inactive (if set)"),
		field.Int("downloads").
			Default(0).
			Min(0).
			Annotations(
				entgql.OrderField("DOWNLOADS"),
			).
			Comment("Number of times the asset has been downloaded using this Link."),
	}
}

// Edges of the Link.
func (Link) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("asset", Asset.Type).
			Unique().
			Required().
			Annotations(
				entgql.Skip(entgql.SkipMutationUpdateInput), // Don't allow changing the asset after creation
			).
			Comment("The asset that this link points to"),
		edge.To("creator", User.Type).
			Unique().
			Annotations(
				entgql.Skip(entgql.SkipMutationCreateInput),
			).
			Comment("User that created the Link if available."),
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
