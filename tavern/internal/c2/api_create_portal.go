package c2

import (
	"context"
	"log/slog"
	"sync"
	"time"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/portals"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/tracepb"
)

func (srv *Server) CreatePortal(gstream c2pb.C2_CreatePortalServer) error {
	// Setup Context
	ctx := gstream.Context()

	// Get initial message for registration
	registerMsg, err := gstream.Recv()
	if err != nil {
		return status.Errorf(codes.Internal, "failed to receive registration message: %v", err)
	}

	taskID := int(registerMsg.GetTaskId())
	if taskID <= 0 {
		return status.Errorf(codes.InvalidArgument, "invalid task ID: %d", taskID)
	}

	portalID, cleanup, err := srv.portalMux.CreatePortal(ctx, srv.graph, taskID)
	if err != nil {
		slog.ErrorContext(ctx, "failed to create portal", "task_id", taskID, "error", err)
		return status.Errorf(codes.Internal, "failed to create portal: %v", err)
	}
	defer cleanup()

	portalInTopic := srv.portalMux.TopicIn(portalID)
	recv, cleanup := srv.portalMux.Subscribe(portalInTopic)
	defer cleanup()

	// Send CLOSE
	defer sendPortalClose(ctx, srv.portalMux, portalID)

	// Start goroutine to subscribe to portal input and send to gRPC stream
	ctx, cancel := context.WithCancel(ctx)
	var wg sync.WaitGroup
	wg.Add(1)
	go func(ctx context.Context) {
		defer wg.Done()
		sendPortalInput(ctx, portalID, gstream, recv)
	}(ctx)

	// Send portal output from gRPC stream to portal output topic
	sendPortalOutput(ctx, portalID, gstream, srv.portalMux)

	// Cleanup
	cancel()
	wg.Wait()

	return nil
}

func sendPortalOutput(ctx context.Context, portalID int, gstream c2pb.C2_CreatePortalServer, mux *mux.Mux) {
	portalOutTopic := mux.TopicOut(portalID)

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

		// TRACE: Server Agent Recv
		if err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_AGENT_RECV); err != nil {
			slog.ErrorContext(ctx, "failed to add trace event (Server Agent Recv)", "error", err)
		}

		// TRACE: Server Agent Pub
		if err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_AGENT_PUB); err != nil {
			slog.ErrorContext(ctx, "failed to add trace event (Server Agent Pub)", "error", err)
		}

		// Publish to portal output topic
		if err := mux.Publish(ctx, portalOutTopic, mote); err != nil {
			slog.ErrorContext(ctx, "failed to publish mote to portal output topic",
				"portal_id", portalID,
				"error", err,
			)
		}
	}
}

func sendPortalInput(ctx context.Context, portalID int, gstream c2pb.C2_CreatePortalServer, recv <-chan *portalpb.Mote) {
	ticker := time.NewTicker(keepAlivePingInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if err := gstream.Send(&c2pb.CreatePortalResponse{
				Mote: &portalpb.Mote{
					Payload: &portalpb.Mote_Bytes{
						Bytes: &portalpb.BytesPayload{
							Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_KEEPALIVE,
						},
					},
				},
			}); err != nil {
				slog.ErrorContext(ctx, "portal failed to send gRPC keep alive ping", "error", err)
			}
		case mote := <-recv:
			// TRACE: Server Agent Sub
			if err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_AGENT_SUB); err != nil {
				slog.ErrorContext(ctx, "failed to add trace event (Server Agent Sub)", "error", err)
			}

			// TRACE: Server Agent Send
			if err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_AGENT_SEND); err != nil {
				slog.ErrorContext(ctx, "failed to add trace event (Server Agent Send)", "error", err)
			}

			if err := gstream.Send(&c2pb.CreatePortalResponse{
				Mote: mote,
			}); err != nil {
				slog.ErrorContext(ctx, "failed to send portal input message",
					"portal_id", portalID,
					"error", err,
				)
			}
		}
	}
}

func sendPortalClose(ctx context.Context, mux *mux.Mux, portalID int) {
	portalOutTopic := mux.TopicOut(portalID)
	if err := mux.Publish(ctx, portalOutTopic, &portalpb.Mote{
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Data: []byte("portal closed"),
				Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE,
			},
		},
	}); err != nil {
		slog.ErrorContext(ctx, "failed to notify subscribers that portal closed",
			"portal_id", portalID,
			"error", err,
		)
	}
}
