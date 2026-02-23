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
	switch msg := req.Message.(type) {
	case *c2pb.ReportOutputRequest_TaskOutput:
		// Handle Task Output
		taskOutputMsg := msg.TaskOutput
		if taskOutputMsg.Context == nil {
			return nil, status.Errorf(codes.InvalidArgument, "missing task context")
		}
		if err := srv.ValidateJWT(taskOutputMsg.Context.Jwt); err != nil {
			return nil, err
		}

		output := taskOutputMsg.Output
		if output == nil || output.Id == 0 {
			return nil, status.Errorf(codes.InvalidArgument, "must provide task id")
		}

		var (
			execStartedAt  *time.Time
			execFinishedAt *time.Time
			taskErr        *string
		)
		if output.ExecStartedAt != nil {
			timestamp := output.ExecStartedAt.AsTime()
			execStartedAt = &timestamp
		}
		if output.ExecFinishedAt != nil {
			timestamp := output.ExecFinishedAt.AsTime()
			execFinishedAt = &timestamp
		}

		// Load Task
		t, err := srv.graph.Task.Get(ctx, int(output.Id))
		if ent.IsNotFound(err) {
			return nil, status.Errorf(codes.NotFound, "no task found (id=%d): %v", output.Id, err)
		}
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to submit task result (id=%d): %v", output.Id, err)
		}

		if output.Error != nil {
			e := fmt.Sprintf("%s%s", t.Error, output.Error.Msg)
			taskErr = &e
		}

		// Update Task
		_, err = t.Update().
			SetNillableExecStartedAt(execStartedAt).
			SetOutput(fmt.Sprintf("%s%s", t.Output, output.Output)).
			SetNillableExecFinishedAt(execFinishedAt).
			SetNillableError(taskErr).
			Save(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to save submitted task result (id=%d): %v", t.ID, err)
		}

	case *c2pb.ReportOutputRequest_ShellTaskOutput:
		// Handle Shell Task Output
		shellTaskOutputMsg := msg.ShellTaskOutput
		if shellTaskOutputMsg.Context == nil {
			return nil, status.Errorf(codes.InvalidArgument, "missing shell task context")
		}
		if err := srv.ValidateJWT(shellTaskOutputMsg.Context.Jwt); err != nil {
			return nil, err
		}

		output := shellTaskOutputMsg.Output
		if output == nil || output.Id == 0 {
			return nil, status.Errorf(codes.InvalidArgument, "must provide shell task id")
		}

		var (
			execStartedAt  *time.Time
			execFinishedAt *time.Time
			shellTaskErr   *string
		)

		if output.ExecStartedAt != nil {
			timestamp := output.ExecStartedAt.AsTime()
			execStartedAt = &timestamp
		}
		if output.ExecFinishedAt != nil {
			timestamp := output.ExecFinishedAt.AsTime()
			execFinishedAt = &timestamp
		}

		// Load ShellTask
		t, err := srv.graph.ShellTask.Get(ctx, int(output.Id))
		if ent.IsNotFound(err) {
			return nil, status.Errorf(codes.NotFound, "no shell task found (id=%d): %v", output.Id, err)
		}
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to submit shell task result (id=%d): %v", output.Id, err)
		}

		if output.Error != nil {
			e := fmt.Sprintf("%s%s", t.Error, output.Error.Msg)
			shellTaskErr = &e
		}

		// Update ShellTask
		update := t.Update().
			SetNillableExecStartedAt(execStartedAt).
			SetOutput(fmt.Sprintf("%s%s", t.Output, output.Output)).
			SetNillableExecFinishedAt(execFinishedAt)

		if shellTaskErr != nil {
			update.SetError(*shellTaskErr)
		}

		_, err = update.Save(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to save submitted shell task result (id=%d): %v", t.ID, err)
		}

	default:
		return nil, status.Errorf(codes.InvalidArgument, "invalid or missing message type")
	}

	return &c2pb.ReportOutputResponse{}, nil
}
