package portals

import (
	"context"
	"fmt"
	"log/slog"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/tracepb"
)

func (srv *Server) OpenPortal(gstream portalpb.Portal_OpenPortalServer) error {
	// Setup Context
	ctx := gstream.Context()

	// Get initial message for registration
	registerMsg, err := gstream.Recv()
	if err != nil {
		return status.Errorf(codes.Internal, "failed to receive registration message: %v", err)
	}

	portalID := int(registerMsg.GetPortalId())
	if portalID <= 0 {
		return status.Errorf(codes.InvalidArgument, "invalid portal ID: %d", portalID)
	}

	p, err := srv.graph.Portal.Get(ctx, portalID)
	if err != nil {
		return status.Errorf(codes.InvalidArgument, "error loading portal ID: %d %v", portalID, err)
	}
	if !p.ClosedAt.IsZero() {
		return status.Errorf(codes.InvalidArgument, "portal %d is closed", portalID)
	}

	cleanup, err := srv.mux.OpenPortal(ctx, portalID)
	if err != nil {
		slog.ErrorContext(ctx, "failed to open portal", "portal_id", portalID, "error", err)
		return status.Errorf(codes.Internal, "failed to open portal: %v", err)
	}
	defer cleanup()

	portalOutTopic := srv.mux.TopicOut(portalID)
	recv, cleanup := srv.mux.Subscribe(portalOutTopic)
	defer cleanup()

	done := make(chan struct{}, 2)

	// Start goroutine to subscribe to portal output and send to gRPC stream
	go func() {
		sendPortalOutput(ctx, portalID, gstream, recv, srv.serverID)
		done <- struct{}{}
	}()

	// Send portal input from gRPC stream to portal input topic
	go func() {
		sendPortalInput(ctx, portalID, gstream, srv.mux, srv.serverID)
		done <- struct{}{}
	}()

	select {
	case <-ctx.Done():
		return nil
	case <-done:
		return fmt.Errorf("portal closed")
	}

	// ctx, cancel := context.WithCancel(ctx)
	// var wg sync.WaitGroup
	// wg.Add(1)
	// go func(ctx context.Context) {
	// 	defer wg.Done()
	// 	defer cancel()
	// 	sendPortalOutput(ctx, portalID, gstream, recv)
	// }(ctx)

	// Send portal input from gRPC stream to portal input topic
	// sendPortalInput(ctx, portalID, gstream, srv.mux)

	// // Cleanup
	// cancel()
	// wg.Wait()

	// return nil
}

func sendPortalInput(ctx context.Context, portalID int, gstream portalpb.Portal_OpenPortalServer, mux *mux.Mux, serverID string) {
	portalInTopic := mux.TopicIn(portalID)

	for {
		// Return if context is done
		select {
		case <-ctx.Done():
			return
		default:
		}

		// Receive from gRPC stream
		req, err := gstream.Recv()
		if err != nil {
			slog.InfoContext(ctx, "portal gRPC stream closed", "portal_id", portalID, "error", err)
			break
		}
		mote := req.GetMote()
		if mote == nil {
			continue
		}

		// Skip keepalive motes
		if bytesMote := mote.GetBytes(); bytesMote != nil && bytesMote.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_KEEPALIVE {
			continue
		}

		// TRACE: Server User Recv
		if err := AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_RECV, serverID); err != nil {
			slog.ErrorContext(ctx, "failed to add trace event (Server User Recv)", "error", err)
		}

		// TRACE: Server User Pub
		if err := AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_PUB, serverID); err != nil {
			slog.ErrorContext(ctx, "failed to add trace event (Server User Pub)", "error", err)
		}

		// Publish to portal input topic
		if err := mux.Publish(ctx, portalInTopic, mote); err != nil {
			slog.ErrorContext(ctx, "failed to publish mote to portal input topic",
				"portal_id", portalID,
				"error", err,
			)
		}
	}
}

func sendPortalOutput(ctx context.Context, portalID int, gstream portalpb.Portal_OpenPortalServer, recv <-chan *portalpb.Mote, serverID string) {
	for {
		select {
		case <-ctx.Done():
			return
		case mote, ok := <-recv:
			if !ok {
				return
			}

			// TRACE: Server User Sub
			if err := AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_SUB, serverID); err != nil {
				slog.ErrorContext(ctx, "failed to add trace event (Server User Sub)", "error", err)
			}

			// TRACE: Server User Send
			if err := AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_SEND, serverID); err != nil {
				slog.ErrorContext(ctx, "failed to add trace event (Server User Send)", "error", err)
			}

			if err := gstream.Send(&portalpb.OpenPortalResponse{
				Mote: mote,
			}); err != nil {
				slog.ErrorContext(ctx, "failed to send portal output message",
					"portal_id", portalID,
					"error", err,
				)
			}

			if payload := mote.GetBytes(); payload != nil && payload.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
				slog.InfoContext(ctx, "received portal close, disconnecting client", "portal_id", portalID, "reason", string(payload.Data))
				return
			}
		}
	}
}
