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

func (srv *Server) ReportOutput(ctx context.Context, req *c2pb.ReportOutputRequest) (*c2pb.ReportOutputResponse, error) {
	// Extract JWT
	var jwtToken string
	switch c := req.Context.(type) {
	case *c2pb.ReportOutputRequest_TaskContext:
		jwtToken = c.TaskContext.Jwt
	case *c2pb.ReportOutputRequest_ShellContext:
		jwtToken = c.ShellContext.Jwt
	default:
		return nil, status.Errorf(codes.InvalidArgument, "must provide context")
	}

	err := srv.ValidateJWT(jwtToken)
	if err != nil {
		return nil, err
	}

	// Handle Output
	switch o := req.Output.(type) {
	case *c2pb.ReportOutputRequest_ShellTaskOutput:
		shellTaskOutput := o.ShellTaskOutput
		if shellTaskOutput.Id == 0 {
			return nil, status.Errorf(codes.InvalidArgument, "must provide shell task id")
		}

		var (
			execStartedAt  *time.Time
			execFinishedAt *time.Time
			shellTaskErr   *string
		)

		if shellTaskOutput.ExecStartedAt != nil {
			timestamp := shellTaskOutput.ExecStartedAt.AsTime()
			execStartedAt = &timestamp
		}
		if shellTaskOutput.ExecFinishedAt != nil {
			timestamp := shellTaskOutput.ExecFinishedAt.AsTime()
			execFinishedAt = &timestamp
		}

		// Load ShellTask
		t, err := srv.graph.ShellTask.Get(ctx, int(shellTaskOutput.Id))
		if ent.IsNotFound(err) {
			return nil, status.Errorf(codes.NotFound, "no shell task found (id=%d): %v", shellTaskOutput.Id, err)
		}
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to submit shell task result (id=%d): %v", shellTaskOutput.Id, err)
		}

		if shellTaskOutput.Error != nil {
			e := fmt.Sprintf("%s%s", t.Error, shellTaskOutput.Error.Msg)
			shellTaskErr = &e
		}

		// Update ShellTask
		update := t.Update().
			SetNillableExecStartedAt(execStartedAt).
			SetOutput(fmt.Sprintf("%s%s", t.Output, shellTaskOutput.Output)).
			SetNillableExecFinishedAt(execFinishedAt)

		if shellTaskErr != nil {
			update.SetError(*shellTaskErr)
		}

		_, err = update.Save(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to save submitted shell task result (id=%d): %v", t.ID, err)
		}

	case *c2pb.ReportOutputRequest_TaskOutput:
		taskOutput := o.TaskOutput
		if taskOutput.Id == 0 {
			return nil, status.Errorf(codes.InvalidArgument, "must provide task id")
		}

		// Parse Input
		var (
			execStartedAt  *time.Time
			execFinishedAt *time.Time
			taskErr        *string
		)
		if taskOutput.ExecStartedAt != nil {
			timestamp := taskOutput.ExecStartedAt.AsTime()
			execStartedAt = &timestamp
		}
		if taskOutput.ExecFinishedAt != nil {
			timestamp := taskOutput.ExecFinishedAt.AsTime()
			execFinishedAt = &timestamp
		}

		// Load Task
		t, err := srv.graph.Task.Get(ctx, int(taskOutput.Id))
		if ent.IsNotFound(err) {
			return nil, status.Errorf(codes.NotFound, "no task found (id=%d): %v", taskOutput.Id, err)
		}
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to submit task result (id=%d): %v", taskOutput.Id, err)
		}

		if taskOutput.Error != nil {
			e := fmt.Sprintf("%s%s", t.Error, taskOutput.Error.Msg)
			taskErr = &e
		}

		// Update Task
		_, err = t.Update().
			SetNillableExecStartedAt(execStartedAt).
			SetOutput(fmt.Sprintf("%s%s", t.Output, taskOutput.Output)).
			SetNillableExecFinishedAt(execFinishedAt).
			SetNillableError(taskErr).
			Save(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to save submitted task result (id=%d): %v", t.ID, err)
		}

	default:
		return nil, status.Errorf(codes.InvalidArgument, "must provide output")
	}

	return &c2pb.ReportOutputResponse{}, nil
}
