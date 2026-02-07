package schema

import (
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"io"

	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// User holds the schema definition for the User entity.
type User struct {
	ent.Schema
}

// Fields of the User.
func (User) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			MinLen(3).
			MaxLen(25).
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("The name displayed for the user"),
		field.String("oauth_id").
			Sensitive().
			Unique().
			Immutable().
			Comment("OAuth Subject ID of the user"),
		field.String("photo_url").
			Comment("URL to the user's profile photo.").
			SchemaType(map[string]string{
				dialect.MySQL: "MEDIUMTEXT", // Override MySQL, improve length maximum
			}),
		field.String("session_token").
			DefaultFunc(newSecureToken).
			Sensitive().
			MaxLen(200).
			Annotations(
				entgql.Skip(),
			).
			Comment("The session token currently authenticating the user"),
		field.String("access_token").
			DefaultFunc(newSecureToken).
			Sensitive().
			MaxLen(200).
			Annotations(
				entgql.Skip(),
			).
			Comment("The token used by applications to authenticate as the user"),
		field.Bool("is_activated").
			Default(false).
			Comment("True if the user is active and able to authenticate"),
		field.Bool("is_admin").
			Default(false).
			Comment("True if the user is an Admin"),
	}
}

// Edges of the User.
func (User) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("tomes", Tome.Type).
			Ref("uploader").
			Annotations(
				entgql.RelayConnection(),
				entgql.MultiOrder(),
			).
			Comment("Tomes uploaded by the user."),
		edge.From("active_shells", Shell.Type).
			Ref("active_users").
			Annotations(
				entgql.RelayConnection(),
				entgql.MultiOrder(),
			).
			Comment("Shells actively used by the user"),
		edge.From("assets", Asset.Type).
			Ref("creator").
			Annotations(
				entgql.RelayConnection(),
				entgql.MultiOrder(),
			).
			Comment("Assets uploaded by the user"),
	}
}

// Annotations describes additional information for the ent.
func (User) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entgql.Mutations(
			entgql.MutationUpdate(),
		),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}

func newSecureToken() string {
	buf := make([]byte, 64)
	_, err := io.ReadFull(rand.Reader, buf)
	if err != nil {
		panic(fmt.Errorf("failed to generate random token: %w", err))
	}
	return base64.StdEncoding.EncodeToString(buf)
}
