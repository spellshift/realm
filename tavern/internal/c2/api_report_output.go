package c2

import (
	"context"
	"fmt"
	"time"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
)

func (srv *Server) ReportOutput(ctx context.Context, req *c2pb.ReportOutputRequest) (*c2pb.ReportOutputResponse, error) {
	switch msg := req.Message.(type) {
	case *c2pb.ReportOutputRequest_TaskOutput:
		// Handle Task Output
		taskOutputMsg := msg.TaskOutput
		if taskOutputMsg.Context == nil {
			return nil, status.Errorf(codes.InvalidArgument, "missing task context")
		}
		beaconID, err := srv.ValidateJWT(taskOutputMsg.Context.Jwt)
		if err != nil {
			return nil, err
		}

		output := taskOutputMsg.Output
		if output == nil || output.Id <= 0 {
			return nil, status.Errorf(codes.InvalidArgument, "must provide task id")
		}
		// When the context references a task, it must be the same task the
		// output reports on, so a valid token cannot be paired with an
		// unrelated task id.
		if taskOutputMsg.Context.TaskId != 0 && taskOutputMsg.Context.TaskId != output.Id {
			return nil, status.Errorf(codes.InvalidArgument, "task context id does not match output task id")
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

		// Load only a task owned by the JWT's beacon.
		t, err := srv.loadTaskForBeacon(ctx, int(output.Id), beaconID)
		if err != nil {
			return nil, err
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
		beaconID, err := srv.ValidateJWT(shellTaskOutputMsg.Context.Jwt)
		if err != nil {
			return nil, err
		}

		output := shellTaskOutputMsg.Output
		if output == nil || output.Id <= 0 {
			return nil, status.Errorf(codes.InvalidArgument, "must provide shell task id")
		}
		// When the context references a shell task, it must be the same shell
		// task the output reports on, so a valid token cannot be paired with
		// an unrelated shell task id.
		if shellTaskOutputMsg.Context.ShellTaskId != 0 && shellTaskOutputMsg.Context.ShellTaskId != output.Id {
			return nil, status.Errorf(codes.InvalidArgument, "shell task context id does not match output shell task id")
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

		// Load only a shell task owned by the JWT's beacon.
		t, err := srv.loadShellTaskForBeacon(ctx, int(output.Id), beaconID)
		if err != nil {
			return nil, err
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
