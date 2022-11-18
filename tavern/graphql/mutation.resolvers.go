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

	// 2. Load Tome Files
	bundleFiles, err := jobTome.QueryFiles().
		Order(ent.Asc(file.FieldID)).
		All(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to load tome files: %w", err)
	}

	// 3. Calculate Bundle Hash
	bundleHash := newBundleHashDigest(bundleFiles...)
	bundleName := fmt.Sprintf("Bundle-%s", bundleHash)

	// 4. Check if bundle exists
	bundle, err := r.client.File.Query().
		Where(file.Name(bundleName)).
		First(ctx)
	if err != nil && !ent.IsNotFound(err) {
		return nil, fmt.Errorf("failed to query tome bundle: %w", err)
	}

	// 5. Create a new bundle if it doesn't yet exist
	if bundle == nil || ent.IsNotFound(err) {
		bundleContent, err := newBundle(bundleFiles...)
		if err != nil {
			return nil, fmt.Errorf("failed to encode tome bundle: %w", err)
		}

		bundle, err = r.client.File.Create().
			SetName(bundleName).
			SetContent(bundleContent).
			Save(ctx)
		if err != nil || bundle == nil {
			return nil, fmt.Errorf("failed to create tome bundle: %w", err)
		}
	}

	// 6. Create Job
	job, err := r.client.Job.Create().
		SetInput(input).
		SetBundle(bundle).
		SetTome(jobTome).
		Save(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to create job: %w", err)
	}
	_ = job.ID

	// 7. Create tasks for each target
	tasks := make([]*ent.Task, 0, len(targets))
	for _, tid := range targets {
		t, err := r.client.Task.Create().
			SetJob(job).
			SetTargetID(tid).
			Save(ctx)
		if err != nil {
			return nil, fmt.Errorf("failed to create task for target (%q): %w", tid, err)
		}
		tasks = append(tasks, t)
	}

	return job, nil
}

// CreateUser is the resolver for the createUser field.
func (r *mutationResolver) CreateUser(ctx context.Context, input ent.CreateUserInput) (*ent.User, error) {
	return r.client.User.Create().SetInput(input).Save(ctx)
}

// UpdateUser is the resolver for the updateUser field.
func (r *mutationResolver) UpdateUser(ctx context.Context, id int, input ent.UpdateUserInput) (*ent.User, error) {
	return r.client.User.UpdateOneID(id).SetInput(input).Save(ctx)
}

// Mutation returns generated.MutationResolver implementation.
func (r *Resolver) Mutation() generated.MutationResolver { return &mutationResolver{r} }

type mutationResolver struct{ *Resolver }
