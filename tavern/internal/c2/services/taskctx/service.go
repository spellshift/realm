package taskctx

import (
	"context"
	"log/slog"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/ent"
)

// Service handles loading context entities.
type Service struct {
	client *ent.Client
}

// New creates a new Context Service.
func New(client *ent.Client) *Service {
	return &Service{client: client}
}

// TaskContext holds the related entities for a task.
type TaskContext struct {
	Task    *ent.Task
	Beacon  *ent.Beacon
	Quest   *ent.Quest
	Creator *ent.User
}

// LoadTaskContext loads the task and its related entities (Beacon, Quest, Creator).
func (s *Service) LoadTaskContext(ctx context.Context, taskID int) (*TaskContext, error) {
	task, err := s.client.Task.Get(ctx, taskID)
	if err != nil {
		if ent.IsNotFound(err) {
			slog.ErrorContext(ctx, "failed: associated task does not exist", "task_id", taskID, "error", err)
			return nil, status.Errorf(codes.NotFound, "task does not exist (task_id=%d)", taskID)
		}
		slog.ErrorContext(ctx, "failed: could not load associated task", "task_id", taskID, "error", err)
		return nil, status.Errorf(codes.Internal, "failed to load task ent (task_id=%d): %v", taskID, err)
	}

	beacon, err := task.Beacon(ctx)
	if err != nil {
		slog.ErrorContext(ctx, "failed: could not load associated beacon", "task_id", taskID, "error", err)
		return nil, status.Errorf(codes.Internal, "failed to load beacon ent (task_id=%d): %v", taskID, err)
	}

	quest, err := task.Quest(ctx)
	if err != nil {
		slog.ErrorContext(ctx, "failed: could not load associated quest", "task_id", taskID, "error", err)
		return nil, status.Errorf(codes.Internal, "failed to load quest ent (task_id=%d): %v", taskID, err)
	}

	creator, err := quest.Creator(ctx)
	if err != nil {
		slog.ErrorContext(ctx, "failed: could not load associated quest creator", "task_id", taskID, "error", err)
		return nil, status.Errorf(codes.Internal, "failed to load quest creator (task_id=%d): %v", taskID, err)
	}

	return &TaskContext{
		Task:    task,
		Beacon:  beacon,
		Quest:   quest,
		Creator: creator,
	}, nil
}
