package c2

import (
	"bytes"
	"fmt"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/file"
)

func (srv *Server) DownloadFile(req *c2pb.DownloadFileRequest, stream c2pb.C2_DownloadFileServer) error {
	ctx := stream.Context()

	// Load File
	name := req.GetName()
	f, err := srv.graph.File.Query().
		Where(file.Name(name)).
		Only(ctx)
	if ent.IsNotFound(err) {
		return status.Errorf(codes.NotFound, "%v", err)
	}
	if err != nil {
		return status.Errorf(codes.Internal, "failed to query file (%q): %v", name, err)
	}

	// Set Header Metadata
	stream.SetHeader(metadata.Pairs(
		"sha3-checksum", f.Hash,
		"file-size", fmt.Sprintf("%d", f.Size),
	))

	// Send File Chunks
	buf := bytes.NewBuffer(f.Content)
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
		sendErr := stream.Send(&c2pb.DownloadFileResponse{
			Chunk: chunk,
		})
		if sendErr != nil {
			return status.Errorf(codes.Internal, "failed to send file content: %v", err)
		}
	}
}
