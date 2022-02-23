package schema

import (
	"entgo.io/ent"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// ImplantCallbackConfig holds the schema definition for the ImplantCallbackConfig entity.
type ImplantCallbackConfig struct {
	ent.Schema
}

// Fields of the ImplantCallbackConfig.
func (ImplantCallbackConfig) Fields() []ent.Field {
	return []ent.Field{
		field.String("uri").
			NotEmpty().
			Comment("URI the implant should use for checking in."),
		field.Int("priority").
			Default(10).
			Min(0).
			Comment("The priority of using this callback config (lower is better)."),
		field.Int("timeout").
			Default(5).
			Positive().
			Comment("The maximum time to wait before attempting the next callback config (seconds)."),
		field.Int("interval").
			Default(60).
			Positive().
			Comment("The maximum duration between callbacks (seconds)."),
		field.Int("jitter").
			Default(0).
			Min(0).
			Comment("The maximum random splay to subtract from the callback interval (seconds)."),
	}
}

// Edges of the ImplantCallbackConfig.
func (ImplantCallbackConfig) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("implantConfigs", ImplantConfig.Type).
			Ref("callbackConfigs").
			Comment("Implant configs that use this callback config"),
	}
}
