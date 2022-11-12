package schema

import (
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"io"

	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/field"
)

// User holds the schema definition for the User entity.
type User struct {
	ent.Schema
}

// Fields of the User.
func (User) Fields() []ent.Field {
	return []ent.Field{
		field.String("Name").
			MinLen(3).
			MaxLen(25).
			Comment("The name displayed for the user"),
		field.String("OAuthID").
			Sensitive().
			Unique().
			Immutable().
			Comment("OAuth Subject ID of the user"),
		field.String("PhotoURL").
			Comment("URL to the user's profile photo."),
		field.String("SessionToken").
			DefaultFunc(newSessionToken).
			Unique().
			Sensitive().
			MaxLen(1000).
			Comment("The session token currently authenticating the user"),
		field.Bool("IsActivated").
			Default(false).
			Comment("True if the user is active and able to authenticate"),
		field.Bool("IsAdmin").
			Default(false).
			Comment("True if the user is an Admin"),
	}
}

// Edges of the User.
func (User) Edges() []ent.Edge {
	return nil
}

// Annotations describes additional information for the ent.
func (User) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.QueryField(),
		entgql.Mutations(
			entgql.MutationCreate(),
			entgql.MutationUpdate(),
		),
	}
}

func newSessionToken() string {
	buf := make([]byte, 64)
	_, err := io.ReadFull(rand.Reader, buf)
	if err != nil {
		panic(fmt.Errorf("failed to generate session token: %w", err))
	}
	return base64.StdEncoding.EncodeToString(buf)
}
