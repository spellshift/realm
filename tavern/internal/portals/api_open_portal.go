package portals

import (
	"context"
	"log/slog"
	"sync"

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
	// portalOutSub := srv.mux.SubName(portalOutTopic)
	recv, cleanup := srv.mux.Subscribe(portalOutTopic)
	defer cleanup()

	// Start goroutine to subscribe to portal output and send to gRPC stream
	ctx, cancel := context.WithCancel(ctx)
	var wg sync.WaitGroup
	wg.Add(1)
	go func(ctx context.Context) {
		defer wg.Done()
		sendPortalOutput(ctx, portalID, gstream, recv)
	}(ctx)

	// Send portal input from gRPC stream to portal input topic
	sendPortalInput(ctx, portalID, gstream, srv.mux)

	// Cleanup
	cancel()
	wg.Wait()

	return nil
}

func sendPortalInput(ctx context.Context, portalID int, gstream portalpb.Portal_OpenPortalServer, mux *mux.Mux) {
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
		if err := AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_RECV); err != nil {
			slog.ErrorContext(ctx, "failed to add trace event (Server User Recv)", "error", err)
		}

		// TRACE: Server User Pub
		if err := AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_PUB); err != nil {
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

func sendPortalOutput(ctx context.Context, portalID int, gstream portalpb.Portal_OpenPortalServer, recv <-chan *portalpb.Mote) {
	for {
		select {
		case <-ctx.Done():
			return
		case mote := <-recv:
			// TRACE: Server User Sub
			if err := AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_SUB); err != nil {
				slog.ErrorContext(ctx, "failed to add trace event (Server User Sub)", "error", err)
			}

			// TRACE: Server User Send
			if err := AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_SEND); err != nil {
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
		}
	}
}

// func sendPortalInput(ctx context.Context, portalID int, gstream portalpb.Portal_InvokePortalServer, pubsubStream *stream.Stream) {
// 	for {
// 		select {
// 		case <-ctx.Done():
// 			return
// 		case msg := <-pubsubStream.Messages():
// 			payload := &portalpb.Payload{}
// 			if err := proto.Unmarshal(msg.Body, payload); err != nil {
// 				slog.ErrorContext(ctx, "failed to unmarshal portal input message",
// 					"portal_id", portalID,
// 					"error", err,
// 				)
// 				continue
// 			}
// 			msgLen := len(msg.Body)
// 			if err := gstream.Send(&portalpb.InvokePortalResponse{
// 				Payload: payload,
// 			}); err != nil {
// 				slog.ErrorContext(ctx, "failed to send input through portal",
// 					"portal_id", portalID,
// 					"msg_len", msgLen,
// 					"error", err,
// 				)
// 				return
// 			}
// 			slog.DebugContext(ctx, "input sent through portal via gRPC",
// 				"portal_id", portalID,
// 				"msg_len", msgLen,
// 			)
// 		}
// 	}
// }

// func sendPortalOutput(ctx context.Context, portalID int, gstream portalpb.Portal_InvokePortalServer, pubsubStream *stream.Stream, mux *stream.Mux) error {
// 	for {
// 		req, err := gstream.Recv()
// 		if err == io.EOF {
// 			return nil
// 		}
// 		if err != nil {
// 			return status.Errorf(codes.Internal, "failed to receive portal request: %v", err)
// 		}
// 		if req.Payload == nil {
// 			continue
// 		}

// 		// Marshal Payload
// 		data, err := proto.Marshal(req.Payload)
// 		if err != nil {
// 			return status.Errorf(codes.Internal, "failed to marshal portal payload: %v", err)
// 		}

// 		// Send Pubsub Message
// 		msgLen := len(data)
// 		if err := pubsubStream.SendMessage(ctx, &pubsub.Message{
// 			Body: data,
// 		}, mux); err != nil {
// 			slog.ErrorContext(ctx, "portal failed to publish output",
// 				"portal_id", portalID,
// 				"msg_len", msgLen,
// 				"error", err,
// 			)
// 			return status.Errorf(codes.Internal, "failed to publish message: %v", err)
// 		}
// 		slog.DebugContext(ctx, "portal published output",
// 			"portal_id", portalID,
// 			"msg_len", msgLen,
// 		)
// 	}
// }

// func sendPortalKeepAlives(ctx context.Context, gstream portalpb.Portal_InvokePortalServer) {
// 	ticker := time.NewTicker(30 * time.Second) // TODO: #m elmos_magic_numbers define this elsewhere
// 	defer ticker.Stop()

// 	for {
// 		select {
// 		case <-ctx.Done():
// 			return
// 		case <-ticker.C:
// 			if err := gstream.Send(&portalpb.InvokePortalResponse{
// 				Payload: &portalpb.Payload{
// 					Payload: &portalpb.Payload_Bytes{
// 						Bytes: &portalpb.BytesMessage{
// 							Kind: portalpb.BytesMessageKind_BYTES_MESSAGE_KIND_PING,
// 							Data: []byte(time.Now().UTC().Format(time.RFC3339Nano)),
// 						},
// 					},
// 				},
// 			}); err != nil {
// 				slog.ErrorContext(ctx, "portal failed to send gRPC keep alive ping", "error", err)
// 			}
// 		}
// 	}
// }
