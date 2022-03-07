package graphql

// This file will be automatically regenerated based on the schema, any resolver implementations
// will be copied through when generating and any unknown code will be moved to the end.

import (
	"context"

	"github.com/kcarretto/realm/tavern/ent"
)

func (r *queryResolver) Node(ctx context.Context, id int) (ent.Noder, error) {
	return r.client.Noder(ctx, id)
}

func (r *queryResolver) Nodes(ctx context.Context, ids []int) ([]ent.Noder, error) {
	return r.client.Noders(ctx, ids)
}

func (r *queryResolver) Targets(ctx context.Context, after *ent.Cursor, first *int, before *ent.Cursor, last *int, orderBy *ent.TargetOrder, where *ent.TargetWhereInput) (*ent.TargetConnection, error) {
	return r.client.Target.Query().
		Paginate(ctx, after, first, before, last,
			ent.WithTargetOrder(orderBy),
			ent.WithTargetFilter(where.Filter),
		)
}

func (r *queryResolver) Credentials(ctx context.Context, after *ent.Cursor, first *int, before *ent.Cursor, last *int, orderBy *ent.CredentialOrder, where *ent.CredentialWhereInput) (*ent.CredentialConnection, error) {
	return r.client.Credential.Query().
		Paginate(ctx, after, first, before, last,
			ent.WithCredentialOrder(orderBy),
			ent.WithCredentialFilter(where.Filter),
		)
}

func (r *queryResolver) Files(ctx context.Context, after *ent.Cursor, first *int, before *ent.Cursor, last *int, orderBy *ent.FileOrder, where *ent.FileWhereInput) (*ent.FileConnection, error) {
	return r.client.File.Query().
		Paginate(ctx, after, first, before, last,
			ent.WithFileOrder(orderBy),
			ent.WithFileFilter(where.Filter),
		)
}

// Query returns QueryResolver implementation.
func (r *Resolver) Query() QueryResolver { return &queryResolver{r} }

type queryResolver struct{ *Resolver }
