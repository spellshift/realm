package c2

import (
	"context"
	"fmt"
	"io"
	"log"
	"sync"
	"time"

	"gocloud.dev/pubsub"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/http/stream"
)

// keepAlivePingInterval defines the frequency to send no-op ping messages to the stream,
// this is useful because imix relies on an input after the "exit" command to perform cleanup.
const (
	keepAlivePingInterval = 5 * time.Second
)

func (srv *Server) ReverseShell(gstream c2pb.C2_ReverseShellServer) error {
	// Setup Context
	ctx := gstream.Context()

	// Create the Shell Entity
	shell, err := srv.graph.Shell.Create().
		SetData([]byte{}).
		Save(ctx)
	if err != nil {
		return fmt.Errorf("failed to create shell: %w", err)
	}
	shellID := shell.ID

	// Log Shell Session
	log.Printf("[gRPC] Reverse Shell Started (shell_id=%d)", shellID)
	defer log.Printf("[gRPC] Reverse Shell Closed (shell_id=%d)", shellID)

	// Create new Stream
	pubsubStream := stream.New(fmt.Sprintf("%d", shellID))

	// Cleanup
	defer func() {
		closedAt := time.Now()

		// Prepare New Context
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()

		// Notify Subscribers that the stream is closed
		log.Printf("[gRPC][ReverseShell] Sending stream close message")
		if err := pubsubStream.SendMessage(ctx, &pubsub.Message{
			Metadata: map[string]string{
				stream.MetadataStreamClose: fmt.Sprintf("%d", shellID),
			},
		}, srv.mux); err != nil {
			log.Printf("[gRPC][ReverseShell][ERROR] Failed to notify subscribers that shell was closed: %v", err)
		}

		// Update Ent
		shell, err := srv.graph.Shell.Get(ctx, shellID)
		if err != nil {
			log.Printf("[gRPC][ReverseShell][ERROR] Failed to retrieve shell ent to update it as closed: %v", err)
			return
		}
		if _, err := shell.Update().
			SetClosedAt(closedAt).
			Save(ctx); err != nil {
			log.Printf("[gRPC][ReverseShell][ERROR] Failed to update shell ent as closed: %v", err)
		}
	}()

	// Register stream with Mux
	srv.mux.Register(pubsubStream)
	defer srv.mux.Unregister(pubsubStream)

	// WaitGroup to manage tasks
	var wg sync.WaitGroup

	// Send Keep Alives
	wg.Add(1)
	go func() {
		defer wg.Done()
		sendKeepAlives(ctx, gstream)
	}()

	// Send Input (to shell)
	wg.Add(1)
	go func() {
		defer wg.Done()
		sendShellInput(ctx, gstream, pubsubStream)
	}()

	// Send Output (to pubsub)
	err = sendShellOutput(ctx, gstream, pubsubStream, srv.mux)

	wg.Wait()

	return err
}

func sendShellInput(ctx context.Context, gstream c2pb.C2_ReverseShellServer, pubsubStream *stream.Stream) {
	for {
		select {
		case <-ctx.Done():
			return
		case msg := <-pubsubStream.Messages():
			if err := gstream.Send(&c2pb.ReverseShellResponse{
				Kind: c2pb.ReverseShellMessageKind_REVERSE_SHELL_MESSAGE_KIND_DATA,
				Data: msg.Body,
			}); err != nil {
				log.Printf("[ERROR] Failed to send gRPC input: %v", err)
				return
			}
		}
	}
}

func sendShellOutput(ctx context.Context, gstream c2pb.C2_ReverseShellServer, pubsubStream *stream.Stream, mux *stream.Mux) error {
	for {
		req, err := gstream.Recv()
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to receive shell request: %v", err)
		}

		// Ping events are no-ops
		if req.Kind == c2pb.ReverseShellMessageKind_REVERSE_SHELL_MESSAGE_KIND_PING {
			continue
		}

		// Send Pubsub Message
		if err := pubsubStream.SendMessage(ctx, &pubsub.Message{
			Body: req.Data,
		}, mux); err != nil {
			return status.Errorf(codes.Internal, "failed to publish message: %v", err)
		}
	}
}

func sendKeepAlives(ctx context.Context, gstream c2pb.C2_ReverseShellServer) {
	ticker := time.NewTicker(keepAlivePingInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if err := gstream.Send(&c2pb.ReverseShellResponse{
				Kind: c2pb.ReverseShellMessageKind_REVERSE_SHELL_MESSAGE_KIND_PING,
			}); err != nil {
				log.Printf("[ERROR] Failed to send gRPC ping: %v", err)
			}
		}
	}
}
