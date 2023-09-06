package graphql

import (
	"context"
	"fmt"

	"github.com/kcarretto/realm/tavern/internal/auth"
	"github.com/kcarretto/realm/tavern/internal/ent"
	"github.com/kcarretto/realm/tavern/internal/graphql/generated"
	"github.com/kcarretto/realm/tavern/internal/graphql/models"

	"github.com/99designs/gqlgen/graphql"
)

// Resolver is the resolver root.
type Resolver struct{ client *ent.Client }

// NewSchema creates a graphql executable schema.
func NewSchema(client *ent.Client) graphql.ExecutableSchema {
	cfg := generated.Config{
		Resolvers: &Resolver{client},
	}
	cfg.Directives.RequireRole = func(ctx context.Context, obj interface{}, next graphql.Resolver, requiredRole models.Role) (interface{}, error) {
		// Allow unauthenticated contexts to continue for open endpoints
		if requiredRole != models.RoleAdmin && requiredRole != models.RoleUser {
			return next(ctx)
		}

		// Require all roles be authenticated
		if !auth.IsAuthenticatedContext(ctx) {
			return nil, fmt.Errorf("%w: unauthenticated", auth.ErrPermissionDenied)
		}

		// Require all roles be activated
		if !auth.IsActivatedContext(ctx) {
			return nil, fmt.Errorf("%w: unactivated", auth.ErrPermissionDenied)
		}

		// Require admin role have administrative privileges
		if requiredRole == models.RoleAdmin && !auth.IsAdminContext(ctx) {
			return nil, fmt.Errorf("%w: requires administrative privileges", auth.ErrPermissionDenied)
		}

		// Allow the request
		return next(ctx)
	}

	return generated.NewExecutableSchema(cfg)
}
