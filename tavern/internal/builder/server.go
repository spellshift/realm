package builder

import (
	"context"
	"log/slog"
	"time"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/ent"
	entbuilder "realm.pub/tavern/internal/ent/builder"
	"realm.pub/tavern/internal/ent/buildtask"
)

// Server implements the Builder gRPC service.
type Server struct {
	graph *ent.Client
	builderpb.UnimplementedBuilderServer
}

// New creates a new Builder gRPC server.
func New(graph *ent.Client) *Server {
	return &Server{
		graph: graph,
	}
}

// Ping is a simple health check endpoint.
func (s *Server) Ping(ctx context.Context, req *builderpb.PingRequest) (*builderpb.PingResponse, error) {
	slog.Info("ping!")
	return &builderpb.PingResponse{}, nil
}

// ClaimBuildTasks returns unclaimed build tasks assigned to the authenticated builder and marks them as claimed.
func (s *Server) ClaimBuildTasks(ctx context.Context, req *builderpb.ClaimBuildTasksRequest) (*builderpb.ClaimBuildTasksResponse, error) {
	now := time.Now()

	// 1. Extract authenticated builder from context
	b, ok := BuilderFromContext(ctx)
	if !ok {
		return nil, status.Error(codes.Unauthenticated, "builder not authenticated")
	}

	// 2. Query unclaimed build tasks assigned to this builder
	tasks, err := s.graph.BuildTask.Query().
		Where(
			buildtask.HasBuilderWith(entbuilder.ID(b.ID)),
			buildtask.ClaimedAtIsNil(),
		).
		All(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to query build tasks: %v", err)
	}

	if len(tasks) == 0 {
		// Log for debugging: count total tasks (including claimed) for this builder
		total, _ := s.graph.BuildTask.Query().
			Where(buildtask.HasBuilderWith(entbuilder.ID(b.ID))).
			Count(ctx)
		slog.InfoContext(ctx, "no unclaimed build tasks found",
			"builder_id", b.ID,
			"total_tasks_for_builder", total,
		)
		return &builderpb.ClaimBuildTasksResponse{}, nil
	}

	// 3. Begin transaction to claim tasks atomically
	tx, err := s.graph.Tx(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to start transaction: %v", err)
	}
	defer func() {
		if v := recover(); v != nil {
			tx.Rollback()
			panic(v)
		}
	}()

	client := tx.Client()
	taskIDs := make([]int, 0, len(tasks))
	for _, t := range tasks {
		_, err := client.BuildTask.UpdateOne(t).
			SetClaimedAt(now).
			SetStartedAt(now).
			Save(ctx)
		if err != nil {
			tx.Rollback()
			return nil, status.Errorf(codes.Internal, "failed to claim build task %d: %v", t.ID, err)
		}
		taskIDs = append(taskIDs, t.ID)
	}

	if err := tx.Commit(); err != nil {
		return nil, status.Errorf(codes.Internal, "failed to commit transaction: %v", err)
	}

	// 4. Build response from claimed tasks
	resp := &builderpb.ClaimBuildTasksResponse{}
	for _, t := range tasks {
		resp.Tasks = append(resp.Tasks, &builderpb.BuildTaskSpec{
			Id:          int64(t.ID),
			TargetOs:    t.TargetOs.String(),
			BuildImage:  t.BuildImage,
			BuildScript: t.BuildScript,
		})
	}

	slog.InfoContext(ctx, "builder claimed build tasks",
		"builder_id", b.ID,
		"task_count", len(resp.Tasks),
	)

	return resp, nil
}

// SubmitBuildTaskOutput records the output of a completed build task.
func (s *Server) SubmitBuildTaskOutput(ctx context.Context, req *builderpb.SubmitBuildTaskOutputRequest) (*builderpb.SubmitBuildTaskOutputResponse, error) {
	now := time.Now()

	// 1. Validate the builder is authenticated
	b, ok := BuilderFromContext(ctx)
	if !ok {
		return nil, status.Error(codes.Unauthenticated, "builder not authenticated")
	}

	// 2. Load the build task and verify it belongs to this builder
	bt, err := s.graph.BuildTask.Get(ctx, int(req.TaskId))
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "build task %d not found: %v", req.TaskId, err)
	}

	btBuilder, err := bt.QueryBuilder().Only(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to query builder for task %d: %v", req.TaskId, err)
	}
	if btBuilder.ID != b.ID {
		return nil, status.Errorf(codes.PermissionDenied, "build task %d is not assigned to this builder", req.TaskId)
	}

	// 3. Update the build task with output and mark as finished
	update := s.graph.BuildTask.UpdateOne(bt).
		SetFinishedAt(now).
		SetOutput(req.Output)
	if req.Error != "" {
		update = update.SetError(req.Error)
	}

	if _, err := update.Save(ctx); err != nil {
		return nil, status.Errorf(codes.Internal, "failed to update build task %d: %v", req.TaskId, err)
	}

	slog.InfoContext(ctx, "build task output submitted",
		"task_id", req.TaskId,
		"builder_id", b.ID,
		"has_error", req.Error != "",
	)

	return &builderpb.SubmitBuildTaskOutputResponse{}, nil
}
