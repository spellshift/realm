package c2

import (
	"fmt"
	"io"
	"log"
	"sync"

	"gocloud.dev/pubsub"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/http/stream"
)

func (srv *Server) ReverseShell(gstream c2pb.C2_ReverseShellServer) error {
	ctx := gstream.Context()

	// Create the Shell Entity
	shell, err := srv.graph.Shell.Create().
		Save(ctx)
	if err != nil {
		return fmt.Errorf("failed to create shell: %w", err)
	}
	log.Printf("[gRPC] Reverse Shell Started (shell_id=%d)", shell.ID)

	// Register a Receiver with the stream.Mux
	recv := stream.NewReceiver(fmt.Sprintf("%d", shell.ID))
	srv.mux.Register(recv)
	defer srv.mux.Unregister(recv)

	// Send Input
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		for msg := range recv.Messages() {
			// TODO: Update Shell Ent

			if err := gstream.Send(&c2pb.ReverseShellResponse{
				Data: msg.Body,
			}); err != nil {
				// TODO: Handle error
				return
			}

		}
	}()

	// Publish Output
	for {
		req, err := gstream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to receive shell request: %v", err)
		}

		// TODO: Update Ent

		srv.mux.Send(ctx, &pubsub.Message{
			Body: req.Data,
			Metadata: map[string]string{
				"id":   fmt.Sprintf("%d", shell.ID),
				"size": fmt.Sprintf("%d", len(req.Data)),
			},
		})
	}

	return nil
}
