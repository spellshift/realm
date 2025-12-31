package c2

import (
	"context"
	"fmt"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
)

func (srv *Server) ReportProcessList(ctx context.Context, req *c2pb.ReportProcessListRequest) (*c2pb.ReportProcessListResponse, error) {
	// Validate Arguments
	if req.TaskId == 0 {
		return nil, status.Errorf(codes.InvalidArgument, "must provide task id")
	}
	if req.List == nil || len(req.List.List) < 1 {
		return nil, status.Errorf(codes.InvalidArgument, "must provide process list")
	}

	// Load Task
	task, err := srv.graph.Task.Get(ctx, int(req.TaskId))
	if ent.IsNotFound(err) {
		return nil, status.Errorf(codes.NotFound, "no task found")
	}
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to load task")
	}

	// Load Host
	host, err := task.QueryBeacon().QueryHost().Only(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to load host")
	}

	// Run Transaction
	if err := transaction.Run(ctx, srv.graph, func(tx *ent.Tx) error {
		txGraph := tx.Client()

		// Create Processes
		builders := make([]*ent.HostProcessCreate, 0, len(req.List.List))
		for _, proc := range req.List.List {
			builders = append(builders,
				txGraph.HostProcess.Create().
					SetHostID(host.ID).
					SetTaskID(task.ID).
					SetPid(proc.Pid).
					SetPpid(proc.Ppid).
					SetName(proc.Name).
					SetPrincipal(proc.Principal).
					SetPath(proc.Path).
					SetCmd(proc.Cmd).
					SetEnv(proc.Env).
					SetCwd(proc.Cwd).
					SetStatus(proc.Status),
			)
		}
		processList, err := txGraph.HostProcess.CreateBulk(builders...).Save(ctx)
		if err != nil {
			return fmt.Errorf("failed to create process list: %w", err)
		}

		// Set new process list for host
		_, err = txGraph.Host.UpdateOne(host).
			ClearProcesses().
			AddProcesses(processList...).
			Save(ctx)
		if err != nil {
			return fmt.Errorf("failed to set new host process list: %w", err)
		}

		return nil
	}); err != nil {
		return nil, err
	}

	return &c2pb.ReportProcessListResponse{}, nil
}
