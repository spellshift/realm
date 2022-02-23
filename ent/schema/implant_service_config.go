package schema

import (
	"entgo.io/ent"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// ImplantServiceConfig holds the schema definition for the ImplantServiceConfig entity.
type ImplantServiceConfig struct {
	ent.Schema
}

// Fields of the ImplantServiceConfig.
func (ImplantServiceConfig) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			Unique().
			NotEmpty().
			Comment("The name of the service."),
		field.String("description").
			Default("System Service").
			Comment("A description of the service."),
		field.String("executablePath").
			NotEmpty().
			Comment("Path to the executable run by the service."),
	}
}

// Edges of the ImplantServiceConfig.
func (ImplantServiceConfig) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("implantConfigs", ImplantConfig.Type).
			Ref("serviceConfigs").
			Comment("Implants configs that use this service config."),
	}
}
