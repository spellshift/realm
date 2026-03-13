package schema

import (
	"entgo.io/ent"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"realm.pub/tavern/internal/builder/builderpb"
)

// BuildProfile holds the schema definition for the BuildProfile entity.
type BuildProfile struct {
	ent.Schema
}

// Fields of the BuildProfile.
func (BuildProfile) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			Comment("The build profiles name"),
		field.String("description").
			Comment("A user facing build profile description."),
		field.JSON("transports", []builderpb.BuildProfileTransport{}).
			Comment("The transports builds should use in order of priority."),
		field.String("prebuildscript").
			Comment("Bash script to run before build command"),
		field.String("postbuildscript").
			Comment("Bash script to run after build command"),
	}
}

// Edges of the BuildProfile.
func (BuildProfile) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("buildtasks", BuildTask.Type).
			Ref("profile").
			Comment("Build Tasks that have used this profile"),
	}
}
