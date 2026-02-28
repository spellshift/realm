package c2

import (
	"bytes"
	"fmt"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
)

func (srv *Server) FetchAsset(req *c2pb.FetchAssetRequest, stream c2pb.C2_FetchAssetServer) error {
	ctx := stream.Context()

	var jwt string
	if tc := req.GetTaskContext(); tc != nil {
		jwt = tc.GetJwt()
	} else if stc := req.GetShellTaskContext(); stc != nil {
		jwt = stc.GetJwt()
	} else {
		return status.Errorf(codes.InvalidArgument, "missing context")
	}

	err := srv.ValidateJWT(jwt)
	if err != nil {
		return err
	}

	// Load Asset
	name := req.GetName()
	a, err := srv.graph.Asset.Query().
		Where(asset.Name(name)).
		Only(ctx)
	if ent.IsNotFound(err) {
		return status.Errorf(codes.NotFound, "%v", err)
	}
	if err != nil {
		return status.Errorf(codes.Internal, "failed to query asset (%q): %v", name, err)
	}

	// Set Header Metadata
	stream.SetHeader(metadata.Pairs(
		"sha3-256-checksum", a.Hash,
		"file-size", fmt.Sprintf("%d", a.Size),
	))

	// Send Asset Chunks
	buf := bytes.NewBuffer(a.Content)
	for {
		// Check Empty Buffer
		if buf.Len() < 1 {
			return nil
		}

		// Determine Chunk Size
		chunkLen := srv.MaxFileChunkSize
		if uint64(buf.Len()) < chunkLen {
			chunkLen = uint64(buf.Len())
		}

		// Read Chunk
		chunk := make([]byte, chunkLen)
		if _, err := buf.Read(chunk); err != nil {
			return status.Errorf(codes.Internal, "failed to read file content: %v", err)
		}

		// Send Chunk
		sendErr := stream.Send(&c2pb.FetchAssetResponse{
			Chunk: chunk,
		})
		if sendErr != nil {
			return status.Errorf(codes.Internal, "failed to send file content: %v", err)
		}
	}
}
