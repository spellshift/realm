package builder

import (
	"context"
	"log/slog"

	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/ent"
)

// Server implements the Builder gRPC service.
type Server struct {
	graph *ent.Client
	builderpb.UnimplementedBuilderServer
}

// New creates a new Builder gRPC server.
func New(graph *ent.Client) *Server {
	return &Server{
		graph: graph,
	}
}

// Ping is a simple health check endpoint.
func (s *Server) Ping(ctx context.Context, req *builderpb.PingRequest) (*builderpb.PingResponse, error) {
	slog.Info("ping!")
	return &builderpb.PingResponse{}, nil
}
