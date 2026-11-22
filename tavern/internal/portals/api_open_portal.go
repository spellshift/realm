package portals

import (
	"context"
	"fmt"
	"io"
	"log/slog"
	"strconv"
	"strings"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/tracepb"
)

func isAccessDeniedError(err error) bool {
	if err == nil {
		return false
	}
	code := status.Code(err)
	if code == codes.PermissionDenied || code == codes.Unauthenticated {
		return true
	}
	lower := strings.ToLower(err.Error())
	return strings.Contains(lower, "privacy") ||
		strings.Contains(lower, "permission denied") ||
		strings.Contains(lower, "deny rule") ||
		strings.Contains(lower, "unauthorized") ||
		strings.Contains(lower, "unauthenticated")
}

func (srv *Server) OpenPortal(gstream portalpb.Portal_OpenPortalServer) error {
	// Setup Context
	ctx := gstream.Context()
	if logging.EntrypointFromContext(ctx) == "" {
		ctx = logging.WithEntrypoint(ctx, "grpc")
	}
	ctx, _, _ = logging.EnsureTrace(ctx)

	if u := auth.UserFromContext(ctx); u != nil {
		if cur := logging.ActorFromContext(ctx); cur.Kind == logging.ActorUnknown || cur.Kind == logging.ActorAnonymous {
			ctx = logging.WithActor(ctx, logging.Actor{
				Kind:    logging.ActorUser,
				ID:      strconv.Itoa(u.ID),
				Name:    u.Name,
				IsAdmin: u.IsAdmin,
			})
		}
	}

	// Get initial message for registration
	registerMsg, err := gstream.Recv()
	if err != nil {
		log.WarnContext(ctx, "failed to receive open portal registration message", "error", err)
		return status.Errorf(codes.Internal, "failed to receive registration message: %v", err)
	}

	portalID := int(registerMsg.GetPortalId())
	if portalID <= 0 {
		log.WarnContext(ctx, "invalid portal ID in open portal request", "portal_id", portalID)
		return status.Errorf(codes.InvalidArgument, "invalid portal ID: %d", portalID)
	}

	p, err := srv.graph.Portal.Get(ctx, portalID)
	if err != nil {
		if isAccessDeniedError(err) {
			log.WarnContext(ctx, "operator denied access to portal", "portal_id", portalID, "error", err)
			deniedArgs := append(
				logging.TargetIntAttrs("portal", portalID, ""),
				slog.Int("portal_id", portalID),
				slog.String("error", err.Error()),
			)
			logging.AuditDenied(ctx, "portal", "portal.opened", "operator denied portal stream", deniedArgs...)
			return status.Errorf(codes.PermissionDenied, "access denied to portal %d: %v", portalID, err)
		}
		log.WarnContext(ctx, "failed to load portal for open request", "portal_id", portalID, "error", err)
		return status.Errorf(codes.InvalidArgument, "error loading portal ID: %d %v", portalID, err)
	}
	if !p.ClosedAt.IsZero() {
		log.WarnContext(ctx, "attempted to open closed portal", "portal_id", portalID)
		return status.Errorf(codes.InvalidArgument, "portal %d is closed", portalID)
	}

	var beaconID, hostID int
	if b, bErr := p.QueryBeacon().WithHost().Only(ctx); bErr == nil {
		beaconID = b.ID
		if b.Edges.Host != nil {
			hostID = b.Edges.Host.ID
		}
	} else {
		log.WarnContext(ctx, "failed to query beacon for portal open", "portal_id", portalID, "error", bErr)
	}

	streamAttrs := []slog.Attr{slog.Int("portal_id", portalID)}
	if beaconID > 0 {
		streamAttrs = append(streamAttrs, slog.Int("beacon_id", beaconID))
	}
	if hostID > 0 {
		streamAttrs = append(streamAttrs, slog.Int("host_id", hostID))
	}
	ctx = logging.WithStreamAttrs(ctx, streamAttrs...)

	portalOutTopic := srv.mux.TopicOut(portalID)
	recv, subCleanup := srv.mux.Subscribe(portalOutTopic)
	defer subCleanup()

	cleanup, err := srv.mux.OpenPortal(ctx, portalID)
	if err != nil {
		log.ErrorContext(ctx, "failed to open portal in mux", "error", err)
		failArgs := append(
			logging.TargetIntAttrs("portal", portalID, ""),
			logging.PortalAttrs(portalID, beaconID, hostID)...,
		)
		failArgs = append(failArgs, slog.String("error", err.Error()))
		logging.AuditFailure(ctx, "portal", "portal.opened", "failed to open operator portal stream", failArgs...)
		return status.Errorf(codes.Internal, "failed to open portal: %v", err)
	}
	defer cleanup()

	auditArgs := append(
		logging.TargetIntAttrs("portal", portalID, ""),
		logging.PortalAttrs(portalID, beaconID, hostID)...,
	)
	log.DebugContext(ctx, "operator opened portal stream")
	logging.Audit(ctx, "portal", "portal.opened", "operator opened portal stream", auditArgs...)

	defer func() {
		disconnectCtx := logging.DetachContext(ctx)
		log.DebugContext(disconnectCtx, "operator disconnected from portal stream")
		logging.Audit(disconnectCtx, "portal", "portal.disconnected", "operator disconnected from portal stream", auditArgs...)
	}()

	done := make(chan struct{}, 2)

	// Start goroutine to subscribe to portal output and send to gRPC stream
	go func() {
		sendPortalOutput(ctx, portalID, gstream, recv)
		done <- struct{}{}
	}()

	// Send portal input from gRPC stream to portal input topic
	go func() {
		sendPortalInput(ctx, portalID, gstream, srv.mux)
		done <- struct{}{}
	}()

	select {
	case <-ctx.Done():
		return nil
	case <-done:
		return fmt.Errorf("portal closed")
	}
}

func sendPortalInput(ctx context.Context, portalID int, gstream portalpb.Portal_OpenPortalServer, mux *mux.Mux) {
	portalInTopic := mux.TopicIn(portalID)
	log.DebugContext(ctx, "starting operator portal input pump", "topic", portalInTopic)
	defer log.DebugContext(ctx, "stopped operator portal input pump", "topic", portalInTopic)

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
			if err == io.EOF {
				log.DebugContext(ctx, "portal gRPC stream ended (EOF)")
			} else if status.Code(err) == codes.Canceled || ctx.Err() != nil {
				log.DebugContext(ctx, "portal gRPC stream canceled")
			} else {
				log.WarnContext(ctx, "portal gRPC stream error", "error", err)
			}
			break
		}
		mote := req.GetMote()
		if mote == nil {
			continue
		}

		// Skip keepalive motes
		if bytesMote := mote.GetBytes(); bytesMote != nil && bytesMote.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_KEEPALIVE {
			log.TraceContext(ctx, "received keepalive mote from operator")
			continue
		}

		// TRACE: Server User Recv
		if traced, err := AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_RECV); err != nil {
			log.ErrorContext(ctx, "failed to add trace event (Server User Recv)", "error", err)
		} else {
			mote = traced
		}

		// TRACE: Server User Pub
		if traced, err := AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_PUB); err != nil {
			log.ErrorContext(ctx, "failed to add trace event (Server User Pub)", "error", err)
		} else {
			mote = traced
		}

		// Publish to portal input topic
		log.TraceContext(ctx, "publishing operator mote to portal input topic", "topic", portalInTopic, "stream_id", mote.GetStreamId(), "seq_id", mote.GetSeqId())
		if err := mux.Publish(ctx, portalInTopic, mote); err != nil {
			log.ErrorContext(ctx, "failed to publish mote to portal input topic",
				"topic", portalInTopic,
				"error", err,
			)
		}
	}
}

func sendPortalOutput(ctx context.Context, portalID int, gstream portalpb.Portal_OpenPortalServer, recv <-chan *portalpb.Mote) {
	log.DebugContext(ctx, "starting operator portal output pump")
	defer log.DebugContext(ctx, "stopped operator portal output pump")

	for {
		select {
		case <-ctx.Done():
			log.DebugContext(ctx, "portal output loop context done", "error", ctx.Err())
			return
		case mote, ok := <-recv:
			if !ok {
				log.DebugContext(ctx, "portal output channel closed")
				return
			}

			// TRACE: Server User Sub
			if traced, err := AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_SUB); err != nil {
				log.ErrorContext(ctx, "failed to add trace event (Server User Sub)", "error", err)
			} else {
				mote = traced
			}

			// TRACE: Server User Send
			if traced, err := AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_SEND); err != nil {
				log.ErrorContext(ctx, "failed to add trace event (Server User Send)", "error", err)
			} else {
				mote = traced
			}

			log.TraceContext(ctx, "sending portal output mote to operator", "stream_id", mote.GetStreamId(), "seq_id", mote.GetSeqId())
			if err := gstream.Send(&portalpb.OpenPortalResponse{
				Mote: mote,
			}); err != nil && ctx.Err() == nil {
				log.ErrorContext(ctx, "failed to send portal output message", "error", err)
			}

			if payload := mote.GetBytes(); payload != nil && payload.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE && mote.StreamId == "" {
				log.DebugContext(ctx, "received portal close mote, disconnecting client", "reason", string(payload.Data))
				return
			}
		}
	}
}
