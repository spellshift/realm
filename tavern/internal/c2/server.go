package c2

import (
	"context"
	"fmt"
	"time"

	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
)

type Server struct {
	MaxFileChunkSize uint64
	graph            *ent.Client
	c2pb.UnimplementedC2Server
}

func New(graph *ent.Client) *Server {
	return &Server{
		MaxFileChunkSize: 1024 * 1024, // 1 MB
		graph:            graph,
	}
}

func (srv *Server) ReportTaskOutput(ctx context.Context, req *c2pb.ReportTaskOutputRequest) (*c2pb.ReportTaskOutputResponse, error) {
	// 1. Parse Input
	var (
		execStartedAt  *time.Time
		execFinishedAt *time.Time
		taskErr        *string
	)
	if req.Output.ExecStartedAt != nil {
		timestamp := req.Output.ExecStartedAt.AsTime()
		execStartedAt = &timestamp
	}
	if req.Output.ExecFinishedAt != nil {
		timestamp := req.Output.ExecFinishedAt.AsTime()
		execFinishedAt = &timestamp
	}
	if req.Output.Error != nil {
		taskErr = &req.Output.Error.Msg
	}

	// 2. Load the task
	t, err := srv.graph.Task.Get(ctx, int(req.Output.Id))
	if err != nil {
		return nil, fmt.Errorf("failed to submit task result (id=%d): %w", req.Output.Id, err)
	}

	// 3. Update task info
	_, err = t.Update().
		SetNillableExecStartedAt(execStartedAt).
		SetOutput(fmt.Sprintf("%s%s", t.Output, req.Output.Output)).
		SetNillableExecFinishedAt(execFinishedAt).
		SetNillableError(taskErr).
		Save(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to save submitted task result (id=%d): %w", t.ID, err)
	}

	return &c2pb.ReportTaskOutputResponse{}, nil
}
