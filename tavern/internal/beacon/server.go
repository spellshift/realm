package beacon

import (
	"context"

	"github.com/kcarretto/realm/tavern/internal/beacon/beaconpb"
	"google.golang.org/grpc"
)

type Server struct{}

func (srv *Server) ClaimTasks(ctx context.Context, in *beaconpb.ClaimTasksRequest, opts ...grpc.CallOption) (*beaconpb.ClaimTasksResponse, error) {
	return nil, nil
}
func (srv *Server) ReportTaskOutputs(ctx context.Context, in *beaconpb.ReportTaskOutputsRequest, opts ...grpc.CallOption) (*beaconpb.ReportTaskOutputsResponse, error) {
	return nil, nil
}
