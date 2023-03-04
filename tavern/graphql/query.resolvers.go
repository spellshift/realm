package graphql

// This file will be automatically regenerated based on the schema, any resolver implementations
// will be copied through when generating and any unknown code will be moved to the end.

import (
	"context"
	"fmt"

	"github.com/kcarretto/realm/tavern/ent"
)

// Files is the resolver for the files field.
func (r *queryResolver) Files(ctx context.Context) ([]*ent.File, error) {
	return r.client.File.Query().All(ctx)
}

// Jobs is the resolver for the jobs field.
func (r *queryResolver) Jobs(ctx context.Context, where *ent.JobWhereInput) ([]*ent.Job, error) {
	if where != nil {
		query, err := where.Filter(r.client.Job.Query())
		if err != nil {
			return nil, fmt.Errorf("failed to apply filter: %w", err)
		}
		return query.All(ctx)
	}
	return r.client.Job.Query().All(ctx)
}

// Sessions is the resolver for the sessions field.
func (r *queryResolver) Sessions(ctx context.Context) ([]*ent.Session, error) {
	return r.client.Session.Query().All(ctx)
}

// Tags is the resolver for the tags field.
func (r *queryResolver) Tags(ctx context.Context) ([]*ent.Tag, error) {
	return r.client.Tag.Query().All(ctx)
}

// Tomes is the resolver for the tomes field.
func (r *queryResolver) Tomes(ctx context.Context) ([]*ent.Tome, error) {
	return r.client.Tome.Query().All(ctx)
}

// Users is the resolver for the users field.
func (r *queryResolver) Users(ctx context.Context) ([]*ent.User, error) {
	return r.client.User.Query().All(ctx)
}
