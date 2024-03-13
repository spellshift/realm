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
	var shellID int
	defer func() {
		log.Printf("[gRPC] Reverse Shell Closed (shell_id=%d)", shellID)
	}()

	// Create the Shell Entity
	shell, err := srv.graph.Shell.Create().
		SetInput([]byte{}).
		SetOutput([]byte{}).
		Save(ctx)
	if err != nil {
		return fmt.Errorf("failed to create shell: %w", err)
	}
	shellID = shell.ID
	log.Printf("[gRPC] Reverse Shell Started (shell_id=%d)", shellID)

	// Send initial message
	if err := gstream.Send(&c2pb.ReverseShellResponse{
		Kind: c2pb.ReverseShellMessageKind_REVERSE_SHELL_MESSAGE_KIND_PING,
		Data: []byte{},
	}); err != nil {
		return err
	}

	// Register a Stream with the stream.Mux
	pubsubStream := stream.New(fmt.Sprintf("%d", shell.ID))
	srv.mux.Register(pubsubStream)
	defer srv.mux.Unregister(pubsubStream)

	// Send Input
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		for msg := range pubsubStream.Messages() {
			// TODO: Update Shell Ent

			if err := gstream.Send(&c2pb.ReverseShellResponse{
				Kind: c2pb.ReverseShellMessageKind_REVERSE_SHELL_MESSAGE_KIND_DATA,
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

		// Ping events are no-ops
		if req.Kind == c2pb.ReverseShellMessageKind_REVERSE_SHELL_MESSAGE_KIND_PING {
			continue
		}
		// TODO: Update Ent

		if err := pubsubStream.SendMessage(ctx, &pubsub.Message{
			Body: req.Data,
			// Metadata: map[string]string{
			// 	"id":        fmt.Sprintf("%d", shell.ID),
			// 	"size":      fmt.Sprintf("%d", len(req.Data)),
			// 	"order-key": fmt.Sprintf("%d", orderKey),
			// },
		}, srv.mux); err != nil {
			return status.Errorf(codes.Internal, "failed to publish message: %v", err)
		}
	}

	return nil
}
