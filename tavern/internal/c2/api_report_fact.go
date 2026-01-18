package c2

import (
	"context"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
)

func (srv *Server) ReportFact(ctx context.Context, req *c2pb.ReportFactRequest) (*c2pb.ReportFactResponse, error) {
	// Validate Arguments
	if req.GetContext().GetTaskId() == 0 {
		return nil, status.Errorf(codes.InvalidArgument, "must provide task id")
	}
	if req.Fact == nil {
		return nil, status.Errorf(codes.InvalidArgument, "must provide fact")
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

	// Create Fact
	if _, err := srv.graph.HostFact.Create().
		SetHost(host).
		SetTask(task).
		SetName(req.Fact.Name).
		SetValue(req.Fact.Value).
		Save(ctx); err != nil {
		return nil, status.Errorf(codes.Internal, "failed to save fact")
	}

	return &c2pb.ReportFactResponse{}, nil
}
