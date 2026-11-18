package schema

import (
	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/field"
	"github.com/google/uuid"
)

// Redirector holds the stable deployment identity and configuration metadata.
type Redirector struct{ ent.Schema }

func (Redirector) Fields() []ent.Field {
	return []ent.Field{
		field.String("identifier").DefaultFunc(uuid.NewString).NotEmpty().Unique().Immutable(),
		field.Enum("transport").Values("http1", "grpc", "dns", "icmp", "quic"),
		field.JSON("domains", []string{}),
		field.Bool("enabled").Default(true),
		field.Time("last_seen_at").Optional().Nillable(),
	}
}

func (Redirector) Annotations() []schema.Annotation {
	return []schema.Annotation{entgql.Skip(entgql.SkipType)}
}

func (Redirector) Mixin() []ent.Mixin {
	return []ent.Mixin{MixinHistory{}}
}
