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
func (r *mutationResolver) CreateJob(ctx context.Context, targets []int, input ent.CreateJobInput) (*ent.Job, error) {
	// 1. Load Tome
	jobTome, err := r.client.Tome.Get(ctx, input.TomeID)
	if err != nil {
		return nil, fmt.Errorf("failed to load tome: %w", err)
	}

	// 2. Load Tome Files (ordered so that hashing is always the same)
	bundleFiles, err := jobTome.QueryFiles().
		Order(ent.Asc(file.FieldID)).
		All(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to load tome files: %w", err)
	}

	// 3. Create bundle (if tome has files)
	var bundleID *int
	if bundleFiles != nil {
		bundle, err := createBundle(ctx, r.client, bundleFiles)
		if err != nil || bundle == nil {
			return nil, fmt.Errorf("failed to create bundle: %w", err)
		}
		bundleID = &bundle.ID
	}

	// 4. Create Job
	job, err := r.client.Job.Create().
		SetInput(input).
		SetNillableBundleID(bundleID).
		SetTome(jobTome).
		Save(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to create job: %w", err)
	}

	// 5. Create tasks for each target
	for _, tid := range targets {
		_, err := r.client.Task.Create().
			SetJob(job).
			SetTargetID(tid).
			Save(ctx)
		if err != nil {
			return nil, fmt.Errorf("failed to create task for target (%q): %w", tid, err)
		}
	}

	return job, nil
}

// UpdateUser is the resolver for the updateUser field.
func (r *mutationResolver) UpdateUser(ctx context.Context, id int, input ent.UpdateUserInput) (*ent.User, error) {
	return r.client.User.UpdateOneID(id).SetInput(input).Save(ctx)
}

// Mutation returns generated.MutationResolver implementation.
func (r *Resolver) Mutation() generated.MutationResolver { return &mutationResolver{r} }

type mutationResolver struct{ *Resolver }
