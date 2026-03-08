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
	if req.List == nil || len(req.List.List) < 1 {
		return nil, status.Errorf(codes.InvalidArgument, "must provide process list")
	}

	var host *ent.Host
	var task *ent.Task
	var shellTask *ent.ShellTask

	if tc := req.GetTaskContext(); tc != nil {
		if err := srv.ValidateJWT(tc.GetJwt()); err != nil {
			return nil, err
		}
		t, err := srv.graph.Task.Get(ctx, int(tc.GetTaskId()))
		if err != nil {
			return nil, status.Errorf(codes.NotFound, "task not found: %v", err)
		}
		task = t
		h, err := t.QueryBeacon().QueryHost().Only(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to load host from task: %v", err)
		}
		host = h
	} else if stc := req.GetShellTaskContext(); stc != nil {
		if err := srv.ValidateJWT(stc.GetJwt()); err != nil {
			return nil, err
		}
		st, err := srv.graph.ShellTask.Get(ctx, int(stc.GetShellTaskId()))
		if err != nil {
			return nil, status.Errorf(codes.NotFound, "shell task not found: %v", err)
		}
		shellTask = st
		h, err := st.QueryShell().QueryBeacon().QueryHost().Only(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to load host from shell task: %v", err)
		}
		host = h
	} else {
		return nil, status.Errorf(codes.InvalidArgument, "missing context")
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
		builder := txGraph.HostProcess.Create().
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
			builder.SetTaskID(task.ID)
		}
		if shellTask != nil {
			builder.SetShellTaskID(shellTask.ID)
		}
		builders = append(builders, builder)
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
