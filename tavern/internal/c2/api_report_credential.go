package c2

import (
	"context"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
)

func (srv *Server) ReportCredential(ctx context.Context, req *c2pb.ReportCredentialRequest) (*c2pb.ReportCredentialResponse, error) {
	if req.Credential == nil {
		return nil, status.Errorf(codes.InvalidArgument, "must provide credential")
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

	// Create Credential
	builder := srv.graph.HostCredential.Create().
		SetHost(host).
		SetPrincipal(req.Credential.Principal).
		SetSecret(req.Credential.Secret).
		SetKind(req.Credential.Kind)

	if task != nil {
		builder.SetTask(task)
	}
	if shellTask != nil {
		builder.SetShellTask(shellTask)
	}

	if _, err := builder.Save(ctx); err != nil {
		return nil, status.Errorf(codes.Internal, "failed to save credential: %v", err)
	}

	return &c2pb.ReportCredentialResponse{}, nil
}
