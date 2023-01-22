package schema

import (
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"io"

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
		field.String("principal").
			Optional().
			NotEmpty().
			Comment("The identity the session is authenticated as (e.g. 'root')"),
		field.String("hostname").
			Optional().
			NotEmpty().
			Comment("The hostname of the system the session is running on."),
		field.String("identifier").
			DefaultFunc(newRandomIdentifier).
			NotEmpty().
			Unique().
			Comment("Unique identifier for the session. Unique to each instance of the session."),
		field.String("agentIdentifier").
			Optional().
			NotEmpty().
			Comment("Identifies the agent that the session is running as (e.g. 'imix')."),
		field.String("hostIdentifier").
			Optional().
			NotEmpty().
			Comment("Unique identifier for the host the session is running on."),
		field.Time("lastSeenAt").
			Optional().
			Annotations(
				entgql.OrderField("LAST_SEEN_AT"),
			).
			Comment("Timestamp of when a task was last claimed or updated for a target"),
	}
}

// Edges of the Target.
func (Session) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("tags", Tag.Type),
		edge.From("tasks", Task.Type).
			Ref("session"),
	}
}

// Annotations describes additional information for the ent.
func (Session) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.QueryField(),
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
