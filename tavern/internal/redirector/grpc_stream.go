package redirector

import (
	"context"

	"google.golang.org/grpc"
)

// streamConfig represents gRPC stream configuration
type streamConfig struct {
	StreamName    string
	MethodPath    string
	ServerStreams bool
	ClientStreams bool
}

// Common stream configurations
var (
	fetchAssetStream = streamConfig{
		StreamName:    "FetchAsset",
		MethodPath:    "/c2.C2/FetchAsset",
		ServerStreams: true,
		ClientStreams: false,
	}

	reportFileStream = streamConfig{
		StreamName:    "ReportFile",
		MethodPath:    "/c2.C2/ReportFile",
		ServerStreams: false,
		ClientStreams: true,
	}
)

// createStream creates a gRPC stream with the given configuration
func createStream(ctx context.Context, conn *grpc.ClientConn, cfg streamConfig) (grpc.ClientStream, error) {
	return conn.NewStream(
		ctx,
		&grpc.StreamDesc{
			StreamName:    cfg.StreamName,
			ServerStreams: cfg.ServerStreams,
			ClientStreams: cfg.ClientStreams,
		},
		cfg.MethodPath,
		grpc.CallContentSubtype("raw"),
	)
}
