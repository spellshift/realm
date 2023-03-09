package schema

import (
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"io"

	"github.com/kcarretto/realm/tavern/namegen"

	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Session holds the schema definition for the Session entity.
type Session struct {
	ent.Schema
}

// Fields of the Session.
func (Session) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			NotEmpty().
			Unique().
			DefaultFunc(namegen.GetRandomName).
			Comment("A human readable identifier for the session."),
		field.String("principal").
			Optional().
			NotEmpty().
			Annotations(
				entgql.Skip(entgql.SkipMutationUpdateInput),
			).
			Comment("The identity the session is authenticated as (e.g. 'root')"),
		field.String("hostname").
			Optional().
			NotEmpty().
			Comment("The hostname of the system the session is running on."),
		field.String("identifier").
			DefaultFunc(newRandomIdentifier).
			NotEmpty().
			Unique().
			Annotations(
				entgql.Skip(entgql.SkipMutationUpdateInput),
			).
			Comment("Unique identifier for the session. Unique to each instance of the session."),
		field.String("agentIdentifier").
			Optional().
			NotEmpty().
			Annotations(
				entgql.Skip(entgql.SkipMutationUpdateInput),
			).
			Comment("Identifies the agent that the session is running as (e.g. 'imix')."),
		field.String("hostIdentifier").
			Optional().
			NotEmpty().
			Annotations(
				entgql.Skip(entgql.SkipMutationUpdateInput),
			).
			Comment("Unique identifier for the host the session is running on."),
		field.Time("lastSeenAt").
			Optional().
			Annotations(
				entgql.OrderField("LAST_SEEN_AT"),
				entgql.Skip(entgql.SkipMutationUpdateInput),
			).
			Comment("Timestamp of when a task was last claimed or updated for a target"),
	}
}

// Edges of the Target.
func (Session) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("tags", Tag.Type).
			Comment("Tags used to group the session with other sessions"),
		edge.From("tasks", Task.Type).
			Annotations(
				entgql.Skip(entgql.SkipMutationUpdateInput),
			).
			Ref("session").
			Comment("Tasks that have been assigned to the session"),
	}
}

// Annotations describes additional information for the ent.
func (Session) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.Mutations(
			entgql.MutationUpdate(),
		),
	}
}

func newRandomIdentifier() string {
	buf := make([]byte, 64)
	_, err := io.ReadFull(rand.Reader, buf)
	if err != nil {
		panic(fmt.Errorf("failed to generate random identifier: %w", err))
	}
	return base64.StdEncoding.EncodeToString(buf)
}
