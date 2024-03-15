package c2

import (
	"bytes"
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
// entBufFlushThreshold is the minimum number of output bytes received before syncing data to the database.
// entBuffFlushMaxDelay is the maximum amount of time before a buffer flush will occur (the next time new data is available).
const (
	keepAlivePingInterval = 5 * time.Second
	entBufFlushThreshold  = 1024 * 1024 // 1MB
	entBuffFlushMaxDelay  = 10 * time.Second
)

func (srv *Server) ReverseShell(gstream c2pb.C2_ReverseShellServer) error {
	ctx := gstream.Context()
	var shellID int
	defer func() {
		log.Printf("[gRPC] Reverse Shell Closed (shell_id=%d)", shellID)
	}()

	// Create the Shell Entity
	shell, err := srv.graph.Shell.Create().
		SetData([]byte{}).
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

	defer func() {
		closedAt := time.Now()

		// Notify Subscribers that the stream is closed
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()

		// Notify Subscribers
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

	// Send Input
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()

		for msg := range pubsubStream.Messages() {
			if err := gstream.Send(&c2pb.ReverseShellResponse{
				Kind: c2pb.ReverseShellMessageKind_REVERSE_SHELL_MESSAGE_KIND_DATA,
				Data: msg.Body,
			}); err != nil {
				log.Printf("[ERROR] Failed to send gRPC input: %v", err)
				return
			}
		}
	}()

	// Send Keep Alives
	wg.Add(1)
	go func() {
		defer wg.Done()

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
	}()

	// Store Stream Data
	streamDataCh := make(chan []byte, 1024*1024)
	wg.Add(1)
	go func() {
		defer wg.Done()

		var buf bytes.Buffer
		lastFlush := time.Now()
		for msg := range streamDataCh {
			if len(msg) < 1 {
				continue
			}

			n, err := buf.Write(msg)
			if err != nil {
				log.Printf("[gRPC][ReverseShell][ERROR] failed to write stream data to buffer (shell_id=%d): %v", shellID, err)
			}
			if n != len(msg) {
				log.Printf("[gRPC][ReverseShell][ERROR] data lost while writing stream to buffer (shell_id=%d): want %d, got %d", shellID, len(msg), n)
			}

			// Flush buffer to ent
			if buf.Len() > entBufFlushThreshold || time.Since(lastFlush) > entBuffFlushMaxDelay {
				shell, err := srv.graph.Shell.Get(ctx, shellID)
				if err != nil {
					log.Printf("[gRPC][ReverseShell][ERROR] failed to get underlying shell ent: %v", err)
					continue
				}

				if _, err := shell.Update().
					SetData(append(shell.Data, buf.Bytes()...)).
					Save(ctx); err != nil {
					log.Printf("[gRPC][ReverseShell][ERROR] failed to write output to ent: %v", err)
					continue
				}

				lastFlush = time.Now()
				buf.Reset()
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

		// Send Pubsub Message
		if err := pubsubStream.SendMessage(ctx, &pubsub.Message{
			Body: req.Data,
		}, srv.mux); err != nil {
			return status.Errorf(codes.Internal, "failed to publish message: %v", err)
		}

		// Update Ent (async)
		streamDataCh <- req.Data
	}

	log.Printf("waiting for threads")
	close(streamDataCh)
	wg.Wait()

	return nil
}
