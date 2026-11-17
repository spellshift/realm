package c2

import (
	"context"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/logging"
)

func (srv *Server) ReportCredential(ctx context.Context, req *c2pb.ReportCredentialRequest) (*c2pb.ReportCredentialResponse, error) {
	ctx, _ = enrichC2Context(ctx)

	if req.Credential == nil {
		return nil, status.Errorf(codes.InvalidArgument, "must provide credential")
	}

	var (
		beaconID  int
		err       error
		host      *ent.Host
		task      *ent.Task
		shellTask *ent.ShellTask
	)

	if tc := req.GetTaskContext(); tc != nil {
		ctx, beaconID, err = srv.ValidateJWTContext(ctx, tc.GetJwt())
		if err != nil {
			return nil, err
		}
		t, err := srv.loadTaskForBeacon(ctx, int(tc.GetTaskId()), beaconID)
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
		ctx, beaconID, err = srv.ValidateJWTContext(ctx, stc.GetJwt())
		if err != nil {
			return nil, err
		}
		st, err := srv.loadShellTaskForBeacon(ctx, int(stc.GetShellTaskId()), beaconID)
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

	// NEVER log req.Credential.Secret!
	auditAttrs := []any{
		"host_id", host.ID,
		"beacon_id", beaconID,
		"principal", req.Credential.Principal,
		"kind", req.Credential.Kind.String(),
	}
	if task != nil {
		auditAttrs = append(auditAttrs, "task_id", task.ID)
	}
	if shellTask != nil {
		auditAttrs = append(auditAttrs, "shell_task_id", shellTask.ID)
	}
	logging.Audit(ctx, "c2", "c2.credential_reported", "beacon reported credential", auditAttrs...)

	return &c2pb.ReportCredentialResponse{}, nil
}
