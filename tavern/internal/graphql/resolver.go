package graphql

import (
	"context"
	"crypto/ed25519"
	"crypto/x509"
	"fmt"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/graphql/generated"
	"realm.pub/tavern/internal/graphql/models"

	"github.com/99designs/gqlgen/graphql"
)

// An option to configure the graphql resolver.
type Option func(*Resolver)

// A RepoImporter is responsible for importing tomes from the provided URL (filter based on provided filter options).
type RepoImporter interface {
	Import(ctx context.Context, repo *ent.Repository, filters ...func(path string) bool) error
}

// Resolver is the resolver root.
type Resolver struct {
	client       *ent.Client
	importer     RepoImporter
	builderCA    *x509.Certificate
	builderCAKey ed25519.PrivateKey
}

// NewSchema creates a graphql executable schema.
func NewSchema(client *ent.Client, importer RepoImporter, builderCA *x509.Certificate, builderCAKey ed25519.PrivateKey) graphql.ExecutableSchema {
	cfg := generated.Config{
		Resolvers: &Resolver{
			client:       client,
			importer:     importer,
			builderCA:    builderCA,
			builderCAKey: builderCAKey,
		},
	}
	for _, opt := range options {
      opt(resolver)
    }
	cfg := generated.Config{
		Resolvers: resolver,
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
