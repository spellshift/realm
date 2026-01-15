package c2

import (
	"context"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
)

func (srv *Server) ReportCredential(ctx context.Context, req *c2pb.ReportCredentialRequest) (*c2pb.ReportCredentialResponse, error) {
	// Validate Arguments
	if req.GetContext().GetTaskId() == 0 {
		return nil, status.Errorf(codes.InvalidArgument, "must provide task id")
	}
	if req.Credential == nil {
		return nil, status.Errorf(codes.InvalidArgument, "must provide credential")
	}
	err := srv.ValidateJWT(req.GetContext().GetJwt())
	if err != nil {
		return nil, err
	}


	// Load Task
	task, err := srv.graph.Task.Get(ctx, int(req.GetContext().GetTaskId()))
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

	// Create Credential
	if _, err := srv.graph.HostCredential.Create().
		SetHost(host).
		SetTask(task).
		SetPrincipal(req.Credential.Principal).
		SetSecret(req.Credential.Secret).
		SetKind(req.Credential.Kind).
		Save(ctx); err != nil {
		return nil, status.Errorf(codes.Internal, "failed to save credential")
	}

	return &c2pb.ReportCredentialResponse{}, nil
}
