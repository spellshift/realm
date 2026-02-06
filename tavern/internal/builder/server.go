package builder

import (
	"context"
	"fmt"
	"io"
	"log/slog"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
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

// BuildAgent initiates a new agent build with the given configuration.
func (srv *Server) BuildAgent(ctx context.Context, req *builderpb.BuildAgentRequest) (*builderpb.BuildAgentResponse, error) {
	slog.InfoContext(ctx, "build agent requested",
		"name", req.GetName(),
		"target_os", req.GetTargetOs(),
		"target_arch", req.GetTargetArch(),
		"imix_config", req.GetImixConfig(),
	)

	// TODO: Implement agent build logic
	return nil, status.Errorf(codes.Unimplemented, "BuildAgent not yet implemented")
}

// ReportBuildStatus reports the current status of an ongoing build.
func (srv *Server) ReportBuildStatus(ctx context.Context, req *builderpb.ReportBuildStatusRequest) (*builderpb.ReportBuildStatusResponse, error) {
	slog.InfoContext(ctx, "build status reported",
		"build_id", req.GetBuildId(),
		"status", req.GetStatus().String(),
		"stdout", req.GetStdout(),
		"stderr", req.GetStderr(),
	)

	// TODO: Implement build status reporting logic
	return nil, status.Errorf(codes.Unimplemented, "ReportBuildStatus not yet implemented")
}

// ReportBuildArtifact receives build artifact data in chunks via a client stream.
func (srv *Server) ReportBuildArtifact(stream builderpb.Builder_ReportBuildArtifactServer) error {
	var (
		buildID      string
		artifactName string
		totalBytes   int
	)

	for {
		req, err := stream.Recv()
		if err == io.EOF {
			slog.Info("build artifact upload complete",
				"build_id", buildID,
				"artifact_name", artifactName,
				"total_bytes", totalBytes,
			)
			// TODO: Implement artifact storage and checksum calculation
			return stream.SendAndClose(&builderpb.ReportBuildArtifactResponse{
				Sha3_256Checksum: fmt.Sprintf("placeholder-checksum-%s", buildID),
			})
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to receive artifact chunk: %v", err)
		}

		buildID = req.GetBuildId()
		artifactName = req.GetArtifactName()
		totalBytes += len(req.GetChunk())
	}
}
