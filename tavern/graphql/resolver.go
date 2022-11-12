package graphql

import (
	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/graphql/generated"

	"github.com/99designs/gqlgen/graphql"
)

// Resolver is the resolver root.
type Resolver struct{ client *ent.Client }

// NewSchema creates a graphql executable schema.
func NewSchema(client *ent.Client) graphql.ExecutableSchema {
	return generated.NewExecutableSchema(generated.Config{
		Resolvers: &Resolver{client},
	})
}
