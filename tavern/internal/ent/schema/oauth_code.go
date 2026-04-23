package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// OAuthCode holds the schema definition for a short-lived OAuth 2.0 authorization code.
type OAuthCode struct {
	ent.Schema
}

// Fields of the OAuthCode.
func (OAuthCode) Fields() []ent.Field {
	return []ent.Field{
		field.String("code").
			Unique().
			Sensitive().
			NotEmpty().
			Comment("The authorization code value"),
		field.String("redirect_uri").
			NotEmpty().
			Comment("The redirect URI this code is bound to"),
		field.String("code_challenge").
			NotEmpty().
			Comment("PKCE code challenge"),
		field.String("code_challenge_method").
			Default("S256").
			Comment("PKCE code challenge method (S256 or plain)"),
		field.Time("expires_at").
			Comment("When this authorization code expires"),
		field.Bool("claimed").
			Default(false).
			Comment("Whether this code has already been exchanged for a token"),
	}
}

// Edges of the OAuthCode.
func (OAuthCode) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("client", OAuthClient.Type).
			Ref("oauth_codes").
			Unique().
			Required().
			Annotations(
				entgql.Skip(),
			).
			Comment("The OAuth client that requested this code"),
		edge.To("user", User.Type).
			Unique().
			Required().
			Annotations(
				entgql.Skip(),
			).
			Comment("The user who approved this authorization request"),
	}
}

// Annotations describes additional information for the ent.
func (OAuthCode) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.Skip(),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}
