package schema

import (
	"context"
	"fmt"

	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"realm.pub/tavern/internal/ent/hook"
	"golang.org/x/crypto/sha3"
)

// Tome holds the schema definition for the Tome entity.
type Tome struct {
	ent.Schema
}

// Fields of the Tome.
func (Tome) Fields() []ent.Field {
	return []ent.Field{
		field.String("name").
			NotEmpty().
			Unique().
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("Name of the tome"),
		field.String("description").
			Comment("Information about the tome"),
		field.String("param_defs").
			Optional().
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT", // Override MySQL, improve length maximum
			}).
			Comment("JSON string describing what parameters are used with the tome"),
		field.String("hash").
			MaxLen(100).
			Annotations(
				entgql.Skip(entgql.SkipAll),
			).
			Comment("A SHA3 digest of the eldritch field"),
		field.String("eldritch").
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT", // Override MySQL, improve length maximum
			}).
			Comment("Eldritch script that will be executed when the tome is run"),
	}
}

// Edges of the Tome.
func (Tome) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("files", File.Type).
			Comment("Any files required for tome execution that will be bundled and provided to the agent for download"),
	}
}

// Annotations describes additional information for the ent.
func (Tome) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.Mutations(
			entgql.MutationCreate(),
		),
	}
}

// Mixin defines common shared properties for the ent.
func (Tome) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // createdAt, lastModifiedAt
	}
}

// Hooks defines middleware for mutations for the ent.
func (Tome) Hooks() []ent.Hook {
	return []ent.Hook{
		hook.On(HookDeriveTomeInfo(), ent.OpCreate|ent.OpUpdate|ent.OpUpdateOne),
	}
}

// HookDeriveTomeInfo will update tome info (e.g. hash) whenever it is mutated.
func HookDeriveTomeInfo() ent.Hook {
	// Get the relevant methods from the Tome Mutation
	// See this example: https://github.com/ent/ent/blob/master/entc/integration/hooks/ent/schema/user.go#L98
	type tMutation interface {
		Eldritch() (string, bool)
		SetHash(s string)
	}

	return func(next ent.Mutator) ent.Mutator {
		return ent.MutateFunc(func(ctx context.Context, m ent.Mutation) (ent.Value, error) {
			// Get the tome mutation
			t, ok := m.(tMutation)
			if !ok {
				return nil, fmt.Errorf("expected tome mutation in schema hook, got: %+v", m)
			}

			// Set the new hash
			eldritch, _ := t.Eldritch()
			t.SetHash(fmt.Sprintf("%x", sha3.Sum256([]byte(eldritch))))

			return next.Mutate(ctx, m)
		})
	}
}
