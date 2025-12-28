package portal

import (
	"context"
	"fmt"
	"io"
	"log/slog"
	"sync"
	"time"

	"gocloud.dev/pubsub"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/portal/portalpb"
)

func (srv *Server) InvokePortal(gstream portalpb.Portal_InvokePortalServer) error {
	// Setup Context
	ctx := gstream.Context()

	// Get initial message for registration
	registerMsg, err := gstream.Recv()
	if err != nil {
		return status.Errorf(codes.Internal, "failed to receive registration message: %v", err)
	}

	// Load the Portal Entity
	portal, err := srv.graph.Portal.Get(ctx, int(registerMsg.GetPortalId()))
	if err != nil {
		slog.ErrorContext(ctx, "Invoke portal failed: could not locate entity", "portal_id", registerMsg.GetPortalId(), "error", err)
		return status.Errorf(codes.Internal, "failed to locate portal: %v", err)
	}
	portalID := portal.ID

	// Log Portal Session
	slog.InfoContext(ctx, "new gRPC portal session ✨",
		"portal_id", portalID,
	)
	defer func(start time.Time) {
		slog.InfoContext(ctx, "closed gRPC portal session",
			"started_at", start.String(),
			"ended_at", time.Now().String(),
			"duration", time.Since(start).String(),
			"portal_id", portalID,
		)
	}(time.Now())

	// Create new Stream
	pubsubStream := stream.New(fmt.Sprintf("%d", portalID))

	// Register stream with Mux
	srv.portalClientMux.Register(pubsubStream)
	defer srv.portalClientMux.Unregister(pubsubStream)

	// WaitGroup to manage tasks
	var wg sync.WaitGroup

	// Send Keep Alives
	wg.Add(1)
	go func() {
		defer wg.Done()
		sendPortalKeepAlives(ctx, gstream)
	}()

	// Send Input (to portal)
	wg.Add(1)
	go func() {
		defer wg.Done()
		sendPortalInput(ctx, portalID, gstream, pubsubStream)
	}()

	// Send Output (to pubsub)
	err = sendPortalOutput(ctx, portalID, gstream, pubsubStream, srv.portalClientMux)

	wg.Wait()

	return err
}

func sendPortalInput(ctx context.Context, portalID int, gstream portalpb.Portal_InvokePortalServer, pubsubStream *stream.Stream) {
	for {
		select {
		case <-ctx.Done():
			return
		case msg := <-pubsubStream.Messages():
			payload := &portalpb.Payload{}
			if err := proto.Unmarshal(msg.Body, payload); err != nil {
				slog.ErrorContext(ctx, "failed to unmarshal portal input message",
					"portal_id", portalID,
					"error", err,
				)
				continue
			}
			msgLen := len(msg.Body)
			if err := gstream.Send(&portalpb.InvokePortalResponse{
				Payload: payload,
			}); err != nil {
				slog.ErrorContext(ctx, "failed to send input through portal",
					"portal_id", portalID,
					"msg_len", msgLen,
					"error", err,
				)
				return
			}
			slog.DebugContext(ctx, "input sent through portal via gRPC",
				"portal_id", portalID,
				"msg_len", msgLen,
			)
		}
	}
}

func sendPortalOutput(ctx context.Context, portalID int, gstream portalpb.Portal_InvokePortalServer, pubsubStream *stream.Stream, mux *stream.Mux) error {
	for {
		req, err := gstream.Recv()
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to receive portal request: %v", err)
		}
		if req.Payload == nil {
			continue
		}

		// Marshal Payload
		data, err := proto.Marshal(req.Payload)
		if err != nil {
			return status.Errorf(codes.Internal, "failed to marshal portal payload: %v", err)
		}

		// Send Pubsub Message
		msgLen := len(data)
		if err := pubsubStream.SendMessage(ctx, &pubsub.Message{
			Body: data,
		}, mux); err != nil {
			slog.ErrorContext(ctx, "portal failed to publish output",
				"portal_id", portalID,
				"msg_len", msgLen,
				"error", err,
			)
			return status.Errorf(codes.Internal, "failed to publish message: %v", err)
		}
		slog.DebugContext(ctx, "portal published output",
			"portal_id", portalID,
			"msg_len", msgLen,
		)
	}
}

func sendPortalKeepAlives(ctx context.Context, gstream portalpb.Portal_InvokePortalServer) {
	ticker := time.NewTicker(30 * time.Second) // TODO: #m elmos_magic_numbers define this elsewhere
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if err := gstream.Send(&portalpb.InvokePortalResponse{
				Payload: &portalpb.Payload{
					Payload: &portalpb.Payload_Bytes{
						Bytes: &portalpb.BytesMessage{
							Kind: portalpb.BytesMessageKind_BYTES_MESSAGE_KIND_PING,
							Data: []byte(time.Now().UTC().Format(time.RFC3339Nano)),
						},
					},
				},
			}); err != nil {
				slog.ErrorContext(ctx, "portal failed to send gRPC keep alive ping", "error", err)
			}
		}
	}
}
