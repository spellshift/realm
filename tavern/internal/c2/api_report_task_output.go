package c2

import (
	"context"
	"fmt"
	"time"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
)

func (srv *Server) ReportTaskOutput(ctx context.Context, req *c2pb.ReportTaskOutputRequest) (*c2pb.ReportTaskOutputResponse, error) {
	err := srv.ValidateJWT(req.GetContext().GetJwt())
	if err != nil {
		return nil, err
	}

	if req.ShellTaskOutput != nil {
		if req.ShellTaskOutput.Id == 0 {
			return nil, status.Errorf(codes.InvalidArgument, "must provide shell task id")
		}

		var (
			execStartedAt  *time.Time
			execFinishedAt *time.Time
			shellTaskErr   *string
		)

		if req.ShellTaskOutput.ExecStartedAt != nil {
			timestamp := req.ShellTaskOutput.ExecStartedAt.AsTime()
			execStartedAt = &timestamp
		}
		if req.ShellTaskOutput.ExecFinishedAt != nil {
			timestamp := req.ShellTaskOutput.ExecFinishedAt.AsTime()
			execFinishedAt = &timestamp
		}

		// Load ShellTask
		t, err := srv.graph.ShellTask.Get(ctx, int(req.ShellTaskOutput.Id))
		if ent.IsNotFound(err) {
			return nil, status.Errorf(codes.NotFound, "no shell task found (id=%d): %v", req.ShellTaskOutput.Id, err)
		}
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to submit shell task result (id=%d): %v", req.ShellTaskOutput.Id, err)
		}

		if req.ShellTaskOutput.Error != nil {
			e := fmt.Sprintf("%s%s", t.Error, req.ShellTaskOutput.Error.Msg)
			shellTaskErr = &e
		}

		// Update ShellTask
		update := t.Update().
			SetNillableExecStartedAt(execStartedAt).
			SetOutput(fmt.Sprintf("%s%s", t.Output, req.ShellTaskOutput.Output)).
			SetNillableExecFinishedAt(execFinishedAt)

		if shellTaskErr != nil {
			update.SetError(*shellTaskErr)
		}

		_, err = update.Save(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to save submitted shell task result (id=%d): %v", t.ID, err)
		}

		return &c2pb.ReportTaskOutputResponse{}, nil
	}

	// Validate Input for regular Task
	if req.Output == nil || req.Output.Id == 0 {
		return nil, status.Errorf(codes.InvalidArgument, "must provide task id")
	}

	// Parse Input
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

	// Load Task
	t, err := srv.graph.Task.Get(ctx, int(req.Output.Id))
	if ent.IsNotFound(err) {
		return nil, status.Errorf(codes.NotFound, "no task found (id=%d): %v", req.Output.Id, err)
	}
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to submit task result (id=%d): %v", req.Output.Id, err)
	}

	if req.Output.Error != nil {
		e := fmt.Sprintf("%s%s", t.Error, req.Output.Error.Msg)
		taskErr = &e
	}

	// Update Task
	_, err = t.Update().
		SetNillableExecStartedAt(execStartedAt).
		SetOutput(fmt.Sprintf("%s%s", t.Output, req.Output.Output)).
		SetNillableExecFinishedAt(execFinishedAt).
		SetNillableError(taskErr).
		Save(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to save submitted task result (id=%d): %v", t.ID, err)
	}

	return &c2pb.ReportTaskOutputResponse{}, nil
}
