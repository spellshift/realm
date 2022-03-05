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

// ImplantConfig holds the schema definition for the ImplantConfig entity.
type ImplantConfig struct {
	ent.Schema
}

// Fields of the ImplantConfig.
func (ImplantConfig) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			Unique().
			NotEmpty().
			Comment("Human-readable name assigned to this implant config (e.g. imix-rhel)."),
		field.String("authToken").
			Unique().
			Immutable().
			DefaultFunc(newAuthToken).
			Comment("Authentication token used by the implant to communicate with the GraphQL API."),
	}
}

// Edges of the Implant.
func (ImplantConfig) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("implants", Implant.Type).
			Ref("config").
			Comment("Implants that are using this config"),
		edge.To("serviceConfigs", ImplantServiceConfig.Type).
			Comment("Service configurations used for implant deployment"),
		edge.To("callbackConfigs", ImplantCallbackConfig.Type).
			Comment("implants that use this callback config"),
	}
}

func newAuthToken() string {
	buf := make([]byte, 32)
	_, err := io.ReadFull(rand.Reader, buf)
	if err != nil {
		panic(fmt.Errorf("failed to generate authToken: %w", err))
	}
	return base64.StdEncoding.EncodeToString(buf)
}
