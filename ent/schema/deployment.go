package schema

import (
	"time"

	"entgo.io/ent"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Deployment holds the schema definition for the Deployment entity.
type Deployment struct {
	ent.Schema
}

// Fields of the Deployment.
func (Deployment) Fields() []ent.Field {
	return []ent.Field{
		field.String("output").
			Default("").
			Comment("Output from the deployment."),
		field.String("error").
			Default("").
			Comment("Any errors that occurred during deployment."),
		field.Time("queuedAt").
			Default(time.Now).
			Comment("The timestamp for when the deployment was queued/created."),
		field.Time("lastModifiedAt").
			Default(time.Now).
			Comment("The timestamp for when the deployment was last updated."),
		field.Time("startedAt").
			Optional().
			Comment("The timestamp for when the deployment started."),
		field.Time("finishedAt").
			Optional().
			Comment("The timestamp for when the deployment finished."),
	}
}

// Edges of the Deployment.
func (Deployment) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("config", DeploymentConfig.Type).
			Unique().
			Required().
			Comment("Deployment Config used for this deployment."),
		edge.To("target", Target.Type).
			Unique().
			Required().
			Comment("Target of this deployment."),
	}
}
