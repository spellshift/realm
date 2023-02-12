package graphql

// This file will be automatically regenerated based on the schema, any resolver implementations
// will be copied through when generating and any unknown code will be moved to the end.

import (
	"context"
	"fmt"
	"time"

	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/ent/file"
	"github.com/kcarretto/realm/tavern/ent/session"
	"github.com/kcarretto/realm/tavern/ent/task"
	"github.com/kcarretto/realm/tavern/graphql/generated"
	"github.com/kcarretto/realm/tavern/graphql/models"
)

// CreateJob is the resolver for the createJob field.
func (r *mutationResolver) CreateJob(ctx context.Context, sessionIDs []int, input ent.CreateJobInput) (*ent.Job, error) {
	// 1. Begin Transaction
	tx, err := r.client.Tx(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to initialize transaction: %w", err)
	}
	client := tx.Client()

	// 2. Rollback transaction if we panic
	defer func() {
		if v := recover(); v != nil {
			tx.Rollback()
			panic(v)
		}
	}()

	// 3. Load Tome
	jobTome, err := client.Tome.Get(ctx, input.TomeID)
	if err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to load tome: %w", err))
	}

	// 4. Load Tome Files (ordered so that hashing is always the same)
	bundleFiles, err := jobTome.QueryFiles().
		Order(ent.Asc(file.FieldID)).
		All(ctx)
	if err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to load tome files: %w", err))
	}

	// 5. Create bundle (if tome has files)
	var bundleID *int
	if len(bundleFiles) > 0 {
		bundle, err := createBundle(ctx, client, bundleFiles)
		if err != nil || bundle == nil {
			return nil, rollback(tx, fmt.Errorf("failed to create bundle: %w", err))
		}
		bundleID = &bundle.ID
	}

	// 6. Create Job
	job, err := client.Job.Create().
		SetInput(input).
		SetNillableBundleID(bundleID).
		SetTome(jobTome).
		Save(ctx)
	if err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to create job: %w", err))
	}

	// 7. Create tasks for each session
	for _, sid := range sessionIDs {
		_, err := client.Task.Create().
			SetJob(job).
			SetSessionID(sid).
			Save(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to create task for session (%q): %w", sid, err))
		}
	}

	// 8. Commit the transaction
	if err := tx.Commit(); err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to commit transaction: %w", err))
	}

	// 9. Load the job with our non transactional client (cannot use transaction after commit)
	job, err = r.client.Job.Get(ctx, job.ID)
	if err != nil {
		return nil, fmt.Errorf("failed to load created job: %w", err)
	}

	return job, nil
}

// UpdateSession is the resolver for the updateSession field.
func (r *mutationResolver) UpdateSession(ctx context.Context, sessionID int, input ent.UpdateSessionInput) (*ent.Session, error) {
	return r.client.Session.UpdateOneID(sessionID).SetInput(input).Save(ctx)
}

// CreateTag is the resolver for the createTag field.
func (r *mutationResolver) CreateTag(ctx context.Context, input ent.CreateTagInput) (*ent.Tag, error) {
	return r.client.Tag.Create().SetInput(input).Save(ctx)
}

// UpdateTag is the resolver for the updateTag field.
func (r *mutationResolver) UpdateTag(ctx context.Context, tagID int, input ent.UpdateTagInput) (*ent.Tag, error) {
	return r.client.Tag.UpdateOneID(tagID).SetInput(input).Save(ctx)
}

// ClaimTasks is the resolver for the claimTasks field.
func (r *mutationResolver) ClaimTasks(ctx context.Context, input models.ClaimTasksInput) ([]*ent.Task, error) {
	// 1. Check if session already exists
	agentSession, err := r.client.Session.Query().
		Where(session.Identifier(input.SessionIdentifier)).
		Only(ctx)
	if err != nil && !ent.IsNotFound(err) {
		return nil, fmt.Errorf("failed to query sessions: %w", err)
	}

	// 2. Create session if it didn't already exist
	if ent.IsNotFound(err) {
		_, err = r.client.Session.Create().
			SetPrincipal(input.Principal).
			SetHostname(input.Hostname).
			SetIdentifier(input.SessionIdentifier).
			SetAgentIdentifier(input.AgentIdentifier).
			SetHostIdentifier(input.HostIdentifier).
			SetLastSeenAt(time.Now()).
			Save(ctx)
		if err != nil {
			return nil, fmt.Errorf("failed to create new session: %w", err)
		}

		// New sessions won't have any tasks yet, so just return an empty list
		return []*ent.Task{}, nil
	}

	// 3. Update the existing session
	agentSession, err = agentSession.Update().
		SetPrincipal(input.Principal).
		SetHostname(input.Hostname).
		SetIdentifier(input.SessionIdentifier).
		SetAgentIdentifier(input.AgentIdentifier).
		SetHostIdentifier(input.HostIdentifier).
		SetLastSeenAt(time.Now()).
		Save(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to update existing session: %w", err)
	}

	// 4. Load any queued tasks for the session
	tasks, err := agentSession.QueryTasks().
		Where(task.ClaimedAtIsNil()).
		All(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to query tasks: %w", err)
	}

	// 5. Prepare Transaction for Claiming Tasks
	tx, err := r.client.Tx(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to initialize transaction: %w", err)
	}
	client := tx.Client()

	// 6. Rollback transaction if we panic
	defer func() {
		if v := recover(); v != nil {
			tx.Rollback()
			panic(v)
		}
	}()

	// 7. Update all ClaimedAt timestamps to claim tasks
	// ** Note: If one fails to update, we roll back the transaction and return the error
	taskIDs := make([]int, 0, len(tasks))
	for _, t := range tasks {
		_, err := client.Task.UpdateOne(t).
			SetClaimedAt(time.Now()).
			Save(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to update task %d: %w", t.ID, err))
		}
		taskIDs = append(taskIDs, t.ID)
	}

	// 8. Commit the transaction
	if err := tx.Commit(); err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to commit transaction: %w", err))
	}

	// 9. Load the tasks with our non transactional client (cannot use transaction after commit)
	result := make([]*ent.Task, 0, len(taskIDs))
	for _, taskID := range taskIDs {
		updatedTask, err := r.client.Task.Get(ctx, taskID)
		if err != nil {
			return nil, fmt.Errorf("failed to load updated task (but they were still updated) %d: %w", taskID, err)
		}
		result = append(result, updatedTask)
	}

	// 10. Return claimed tasks
	return result, nil
}

// SubmitTaskResult is the resolver for the submitTaskResult field.
func (r *mutationResolver) SubmitTaskResult(ctx context.Context, input models.SubmitTaskResultInput) (*ent.Task, error) {
	// 1. Load the task
	t, err := r.client.Task.Get(ctx, input.TaskID)
	if err != nil {
		return nil, fmt.Errorf("failed to submit task result: %w", err)
	}

	t, err = t.Update().
		SetExecStartedAt(input.ExecStartedAt).
		SetOutput(fmt.Sprintf("%s%s", t.Output, input.Output)).
		SetNillableExecFinishedAt(input.ExecFinishedAt).
		SetNillableError(input.Error).
		Save(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to save submitted task result: %w", err)
	}

	return t, nil
}

// CreateTome is the resolver for the createTome field.
func (r *mutationResolver) CreateTome(ctx context.Context, input ent.CreateTomeInput) (*ent.Tome, error) {
	return r.client.Tome.Create().SetInput(input).Save(ctx)
}

// UpdateUser is the resolver for the updateUser field.
func (r *mutationResolver) UpdateUser(ctx context.Context, userID int, input ent.UpdateUserInput) (*ent.User, error) {
	return r.client.User.UpdateOneID(userID).SetInput(input).Save(ctx)
}

// Mutation returns generated.MutationResolver implementation.
func (r *Resolver) Mutation() generated.MutationResolver { return &mutationResolver{r} }

type mutationResolver struct{ *Resolver }
