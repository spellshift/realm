package graphql

// This file will be automatically regenerated based on the schema, any resolver implementations
// will be copied through when generating and any unknown code will be moved to the end.

import (
	"context"

	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/graphql/generated"
)

// Node is the resolver for the node field.
func (r *queryResolver) Node(ctx context.Context, id int) (ent.Noder, error) {
	return r.client.Noder(ctx, id)
}

// Nodes is the resolver for the nodes field.
func (r *queryResolver) Nodes(ctx context.Context, ids []int) ([]ent.Noder, error) {
	return r.client.Noders(ctx, ids)
}

// Files is the resolver for the files field.
func (r *queryResolver) Files(ctx context.Context) ([]*ent.File, error) {
	return r.client.File.Query().All(ctx)
}

// Jobs is the resolver for the jobs field.
func (r *queryResolver) Jobs(ctx context.Context) ([]*ent.Job, error) {
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

// Query returns generated.QueryResolver implementation.
func (r *Resolver) Query() generated.QueryResolver { return &queryResolver{r} }

type queryResolver struct{ *Resolver }
