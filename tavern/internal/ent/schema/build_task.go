package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"realm.pub/tavern/internal/c2/c2pb"
)

// BuildTask holds the schema definition for the BuildTask entity.
type BuildTask struct {
	ent.Schema
}

// Fields of the BuildTask.
func (BuildTask) Fields() []ent.Field {
	return []ent.Field{
		field.Enum("target_os").
			GoType(c2pb.Host_Platform(0)).
			Annotations(
				entgql.Type("HostPlatform"),
				entgql.OrderField("TARGET_OS"),
			).
			Comment("The target operating system platform for this build."),
		field.String("build_image").
			NotEmpty().
			Comment("Docker container image name to use for the build."),
		field.Text("build_script").
			NotEmpty().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT",
			}).
			Comment("The script to execute inside the build container."),
		field.Time("claimed_at").
			Optional().
			Annotations(
				entgql.OrderField("CLAIMED_AT"),
			).
			Comment("Timestamp of when a builder claimed this task, null if unclaimed."),
		field.Time("started_at").
			Optional().
			Annotations(
				entgql.OrderField("STARTED_AT"),
			).
			Comment("Timestamp of when the build execution started, null if not yet started."),
		field.Time("finished_at").
			Optional().
			Annotations(
				entgql.OrderField("FINISHED_AT"),
			).
			Comment("Timestamp of when the build finished, null if not yet finished."),
		field.Text("output").
			Optional().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT",
			}).
			Comment("Output from the build execution."),
		field.Text("error").
			Optional().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT",
			}).
			Comment("Error message if the build failed."),
	}
}

// Edges of the BuildTask.
func (BuildTask) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("builder", Builder.Type).
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Required().
			Unique().
			Comment("The builder assigned to execute this build task."),
	}
}

// Annotations describes additional information for the ent.
func (BuildTask) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}

// Mixin defines common shared properties for the ent.
func (BuildTask) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}
