package schema

import (
	"entgo.io/ent"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// DeploymentConfig holds the schema definition for the DeploymentConfig entity.
type DeploymentConfig struct {
	ent.Schema
}

// Fields of the DeploymentConfig.
func (DeploymentConfig) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			Unique().
			NotEmpty().
			Comment("Human-readable name assigned to this deployment config (e.g. 'Deploy Imix')."),
		field.String("cmd").
			Default("").
			Comment("Command to execute on target during deployment. It may contain templated vars such as {{B64_IMPLANT_CFG}}."),
		field.Bool("startCmd").
			Default(false).
			Comment("If set to true, start the cmd and disown it. No cmd output will be available."),
		field.String("fileDst").
			Default("").
			Comment("If there is a file associated with this deployment config, it will be deployed to this location on the target system."),
	}
}

// Edges of the DeploymentConfig.
func (DeploymentConfig) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("deployments", Deployment.Type).
			Ref("config").
			Comment("Deployments referencing this config."),
		edge.To("file", File.Type).
			Unique().
			Comment("A file to deploy to the target system."),
		edge.To("implantConfig", ImplantConfig.Type).
			Unique().
			Comment("Implant configuration to provide, available via the {{B64_IMPLANT_CGF}} cmd template var."),
	}
}
