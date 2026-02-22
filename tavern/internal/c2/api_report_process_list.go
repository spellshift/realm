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
	var taskID int64
	var shellTaskID int64
	var jwtToken string

	switch c := req.Context.(type) {
	case *c2pb.ReportProcessListRequest_TaskContext:
		jwtToken = c.TaskContext.Jwt
		taskID = c.TaskContext.TaskId
	case *c2pb.ReportProcessListRequest_ShellContext:
		jwtToken = c.ShellContext.Jwt
		shellTaskID = c.ShellContext.TaskId
	default:
		return nil, status.Errorf(codes.InvalidArgument, "must provide context")
	}

	// Validate Arguments
	if taskID == 0 && shellTaskID == 0 {
		return nil, status.Errorf(codes.InvalidArgument, "must provide task id or shell task id")
	}
	if req.List == nil || len(req.List.List) < 1 {
		return nil, status.Errorf(codes.InvalidArgument, "must provide process list")
	}
	err := srv.ValidateJWT(jwtToken)
	if err != nil {
		return nil, err
	}

	var host *ent.Host
	var task *ent.Task
	var shellTask *ent.ShellTask
	var shell *ent.Shell

	if taskID != 0 {
		// Load Task
		task, err = srv.graph.Task.Get(ctx, int(taskID))
		if ent.IsNotFound(err) {
			return nil, status.Errorf(codes.NotFound, "no task found")
		}
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to load task")
		}

		// Load Host
		host, err = task.QueryBeacon().QueryHost().Only(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to load host")
		}
	} else {
		// Load ShellTask
		shellTask, err = srv.graph.ShellTask.Get(ctx, int(shellTaskID))
		if ent.IsNotFound(err) {
			return nil, status.Errorf(codes.NotFound, "no shell task found")
		}
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to load shell task")
		}

		// Load Shell
		shell, err = shellTask.QueryShell().Only(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to load shell")
		}

		// Load Host
		host, err = shell.QueryBeacon().QueryHost().Only(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to load host")
		}
	}

	// 	Prepare Transaction
	tx, err := srv.graph.Tx(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to initialize transaction: %v", err)
	}
	txGraph := tx.Client()

	// Rollback transaction if we panic
	defer func() {
		if v := recover(); v != nil {
			tx.Rollback()
			panic(v)
		}
	}()

	// Create Processes
	builders := make([]*ent.HostProcessCreate, 0, len(req.List.List))
	for _, proc := range req.List.List {
		b := txGraph.HostProcess.Create().
			SetHostID(host.ID).
			SetPid(proc.Pid).
			SetPpid(proc.Ppid).
			SetName(proc.Name).
			SetPrincipal(proc.Principal).
			SetPath(proc.Path).
			SetCmd(proc.Cmd).
			SetEnv(proc.Env).
			SetCwd(proc.Cwd).
			SetStatus(proc.Status)

		if task != nil {
			b.SetTask(task)
		}
		if shell != nil {
			b.SetShell(shell)
		}
		if shellTask != nil {
			b.SetShellTask(shellTask)
		}
		builders = append(builders, b)
	}
	processList, err := txGraph.HostProcess.CreateBulk(builders...).Save(ctx)
	if err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to create process list: %w", err))
	}

	// Set new process list for host
	_, err = txGraph.Host.UpdateOne(host).
		ClearProcesses().
		AddProcesses(processList...).
		Save(ctx)
	if err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to set new host process list: %w", err))
	}

	// Commit the transaction
	if err := tx.Commit(); err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to commit transaction: %w", err))
	}

	return &c2pb.ReportProcessListResponse{}, nil
}
