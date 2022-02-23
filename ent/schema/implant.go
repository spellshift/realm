package schema

import (
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"io"

	"entgo.io/ent"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Implant holds the schema definition for the Implant entity.
type Implant struct {
	ent.Schema
}

// Fields of the Implant.
func (Implant) Fields() []ent.Field {
	return []ent.Field{
		field.String("sessionID").
			NotEmpty().
			Comment("Unique identifier for this instance of the implant (if it's running)."),
		field.String("processName").
			Optional().
			Comment("Name of the process this implant is running as."),
	}
}

// Edges of the Implant.
func (Implant) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("target", Target.Type).
			Unique().
			Required().
			Comment("The target this implant is (or will be) installed on."),
		edge.To("config", ImplantConfig.Type).
			Unique().
			Required().
			Comment("Configuration for this implant"),
	}
}

func newSessionID() string {
	buf := make([]byte, 32)
	_, err := io.ReadFull(rand.Reader, buf)
	if err != nil {
		panic(fmt.Errorf("failed to generate authToken: %w", err))
	}
	return base64.StdEncoding.EncodeToString(buf)
}
