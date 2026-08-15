package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// OAuthClient holds the schema definition for a dynamically registered OAuth 2.0 client.
type OAuthClient struct {
	ent.Schema
}

// Fields of the OAuthClient.
func (OAuthClient) Fields() []ent.Field {
	return []ent.Field{
		field.String("client_id").
			Unique().
			Immutable().
			NotEmpty().
			Annotations(
				entgql.OrderField("CLIENT_ID"),
			).
			Comment("Unique identifier for the OAuth client"),
		field.String("client_name").
			Optional().
			Comment("Human-readable name of the OAuth client"),
		field.JSON("redirect_uris", []string{}).
			Comment("Allowed redirect URIs for the OAuth client"),
	}
}

// Edges of the OAuthClient.
func (OAuthClient) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("oauth_codes", OAuthCode.Type).
			Annotations(
				entgql.Skip(),
			).
			Comment("Authorization codes issued to this client"),
	}
}

// Annotations describes additional information for the ent.
func (OAuthClient) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}

// Mixin defines common shared properties for the ent.
func (OAuthClient) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{},
	}
}
