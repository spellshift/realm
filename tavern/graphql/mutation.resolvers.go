package graphql

// This file will be automatically regenerated based on the schema, any resolver implementations
// will be copied through when generating and any unknown code will be moved to the end.

import (
	"context"
	"fmt"

	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/ent/file"
	"github.com/kcarretto/realm/tavern/graphql/generated"
)

// CreateJob is the resolver for the createJob field.
func (r *mutationResolver) CreateJob(ctx context.Context, targetIDs []int, input ent.CreateJobInput) (*ent.Job, error) {
	tx, err := r.client.Tx(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to initialize transaction: %w", err)
	}

	// 1. Load Tome
	jobTome, err := tx.Tome.Get(ctx, input.TomeID)
	if err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to load tome: %w", err))
	}

	// 2. Load Tome Files (ordered so that hashing is always the same)
	bundleFiles, err := jobTome.QueryFiles().
		Order(ent.Asc(file.FieldID)).
		All(ctx)
	if err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to load tome files: %w", err))
	}

	// 3. Create bundle (if tome has files)
	var bundleID *int
	if len(bundleFiles) > 0 {
		bundle, err := createBundle(ctx, tx, bundleFiles)
		if err != nil || bundle == nil {
			return nil, rollback(tx, fmt.Errorf("failed to create bundle: %w", err))
		}
		bundleID = &bundle.ID
	}

	// 4. Create Job
	job, err := tx.Job.Create().
		SetInput(input).
		SetNillableBundleID(bundleID).
		SetTome(jobTome).
		Save(ctx)
	if err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to create job: %w", err))
	}

	// 5. Create tasks for each target
	for _, tid := range targetIDs {
		_, err := tx.Task.Create().
			SetJob(job).
			SetTargetID(tid).
			Save(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to create task for target (%q): %w", tid, err))
		}
	}

	// 6. Commit the transaction
	if err := tx.Commit(); err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to commit transaction: %w", err))
	}

	return job, nil
}

// ClaimTasks is the resolver for the claimTasks field.
func (r *mutationResolver) ClaimTasks(ctx context.Context, targetID int) ([]*ent.Task, error) {
	panic(fmt.Errorf("not implemented: ClaimTasks - claimTasks"))
}

// UpdateUser is the resolver for the updateUser field.
func (r *mutationResolver) UpdateUser(ctx context.Context, userID int, input ent.UpdateUserInput) (*ent.User, error) {
	return r.client.User.UpdateOneID(userID).SetInput(input).Save(ctx)
}

// Mutation returns generated.MutationResolver implementation.
func (r *Resolver) Mutation() generated.MutationResolver { return &mutationResolver{r} }

type mutationResolver struct{ *Resolver }
