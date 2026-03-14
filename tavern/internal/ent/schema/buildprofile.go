package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"realm.pub/tavern/internal/builder"
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
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("The build profiles name"),
		field.String("description").
			Comment("A user facing build profile description."),
		field.JSON("transports", []builderpb.BuildProfileTransport{}).
			Default(builder.DefaultTransports).
			Comment("The transports builds should use in order of priority."),
		field.String("build_image").
			NotEmpty().
			Default(builder.DefaultBuildImage).
			Comment("Docker container image name to use for the build."),
		field.String("prebuildscript").
			Default("echo 'no prebuild set'").
			Comment("Bash script to run before build command"),
		field.String("postbuildscript").
			Default("echo 'no postbuild set'").
			Comment("Bash script to run after build command"),
		field.JSON("tomes", []builderpb.BuildProfileTome{}).
			Optional().
			Annotations(
				entgql.Type("[BuildProfileTome!]"),
			).
			Comment("The tomes to include in builds using this profile."),
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

func (BuildProfile) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}
