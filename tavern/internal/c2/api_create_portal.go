package c2

import (
	"context"
	"io"
	"log/slog"
	"strconv"
	"time"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/cryptocodec"
	"realm.pub/tavern/internal/ent"
	entbeacon "realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/ent/portal"
	"realm.pub/tavern/internal/ent/shellpivot"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/portals"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/portals/ssh"
	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/tracepb"
)

var portalLog = logging.For("portals.c2")

// keepAlivePingInterval defines the frequency to send no-op ping messages to the stream,
// to keep it alive while waiting for input.
const (
	keepAlivePingInterval = 5 * time.Second
)

func (srv *Server) CreatePortal(gstream c2pb.C2_CreatePortalServer) error {
	// Setup Context
	ctx := gstream.Context()
	if logging.EntrypointFromContext(ctx) == "" {
		ctx = logging.WithEntrypoint(ctx, "grpc")
	}
	ctx, _, _ = logging.EnsureTrace(ctx)

	// Get initial message for registration
	registerMsg, err := gstream.Recv()
	if err != nil {
		portalLog.WarnContext(ctx, "failed to receive portal registration message", "error", err)
		return status.Errorf(codes.Internal, "failed to receive registration message: %v", err)
	}

	var taskID int
	var shellTaskID int
	var beaconID int
	if tc := registerMsg.GetTaskContext(); tc != nil {
		beaconID, err = srv.ValidateJWT(tc.GetJwt())
		if err != nil {
			portalLog.WarnContext(ctx, "portal task context JWT validation failed", "error", err)
			return err
		}
		taskID = int(tc.GetTaskId())
	} else if stc := registerMsg.GetShellTaskContext(); stc != nil {
		beaconID, err = srv.ValidateJWT(stc.GetJwt())
		if err != nil {
			portalLog.WarnContext(ctx, "portal shell task context JWT validation failed", "error", err)
			return err
		}
		shellTaskID = int(stc.GetShellTaskId())
	} else {
		portalLog.WarnContext(ctx, "portal registration missing task or shell task context")
		return status.Errorf(codes.InvalidArgument, "missing context")
	}

	if taskID <= 0 && shellTaskID <= 0 {
		portalLog.WarnContext(ctx, "portal registration has invalid task and shell task IDs", "beacon_id", beaconID, "task_id", taskID, "shell_task_id", shellTaskID)
		return status.Errorf(codes.InvalidArgument, "invalid task ID or shell task ID")
	}

	var hostID int
	var beaconName string
	if b, bErr := srv.graph.Beacon.Query().Where(entbeacon.ID(beaconID)).WithHost().Only(ctx); bErr == nil {
		beaconName = b.Name
		if beaconName == "" {
			beaconName = b.Identifier
		}
		if b.Edges.Host != nil {
			hostID = b.Edges.Host.ID
		}
	}
	if cur := logging.ActorFromContext(ctx); cur.Kind == logging.ActorUnknown || cur.Kind == logging.ActorAnonymous {
		ctx = logging.WithActor(ctx, logging.Actor{
			Kind:       logging.ActorBeacon,
			ID:         strconv.Itoa(beaconID),
			Name:       beaconName,
			AuthMethod: "jwt",
		})
	}

	portalID, cleanup, err := srv.portalMux.CreatePortal(ctx, srv.graph, taskID, shellTaskID, beaconID)
	if err != nil {
		portalLog.ErrorContext(ctx, "failed to create portal", "beacon_id", beaconID, "task_id", taskID, "shell_task_id", shellTaskID, "error", err)
		return status.Errorf(codes.Internal, "failed to create portal: %v", err)
	}
	defer cleanup()

	portalInTopic := srv.portalMux.TopicIn(portalID)
	recv, subCleanup := srv.portalMux.Subscribe(portalInTopic)
	defer subCleanup()

	streamAttrs := []slog.Attr{
		slog.Int("portal_id", portalID),
		slog.Int("beacon_id", beaconID),
	}
	if hostID > 0 {
		streamAttrs = append(streamAttrs, slog.Int("host_id", hostID))
	}
	ctx = logging.WithStreamAttrs(ctx, streamAttrs...)

	portalAuditArgs := append(
		logging.TargetIntAttrs("portal", portalID, ""),
		logging.PortalAttrs(portalID, beaconID, hostID)...,
	)
	if taskID > 0 {
		portalAuditArgs = append(portalAuditArgs, slog.Int("task_id", taskID))
	}
	if shellTaskID > 0 {
		portalAuditArgs = append(portalAuditArgs, slog.Int("shell_task_id", shellTaskID))
	}

	portalLog.DebugContext(ctx, "agent created portal", "task_id", taskID, "shell_task_id", shellTaskID)
	logging.Audit(ctx, "portal", "portal.created", "agent created portal", portalAuditArgs...)

	// Send CLOSE when stream terminates (detached context preserves trace, actor, and stream attrs)
	closeCtx := logging.DetachContext(ctx)
	defer sendPortalClose(closeCtx, srv.graph, srv.portalMux, portalID, portalAuditArgs)

	// Start goroutine to subscribe to portal input and send to gRPC stream
	ctx, cancel := context.WithCancel(ctx)

	// Capture client public key for encrypting outbound gRPC stream messages
	clientPubKey, _ := cryptocodec.GetSessionPubKey()

	// Dedicated channel for sending motes to the agent over the gRPC stream.
	// Using a channel and sending from the main CreatePortal handler goroutine ensures:
	// 1. Thread-safety: gRPC stream.Send is only called by one goroutine.
	// 2. Crypto session continuity: cryptocodec encryption derives the key from this goroutine.
	agentSendCh := make(chan *portalpb.Mote, 1024)
	sendToAgent := func(mote *portalpb.Mote) error {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case agentSendCh <- mote:
			return nil
		}
	}

	sshMux := ssh.New(portalID, srv.portalMux, sendToAgent)
	defer sshMux.Close()

	done := make(chan struct{}, 2)

	go func() {
		sendPortalInput(ctx, portalID, sendToAgent, sshMux, recv)
		done <- struct{}{}
	}()

	// Send portal output from gRPC stream to portal output topic
	go func() {
		sendPortalOutput(ctx, portalID, gstream, srv.portalMux, sshMux)
		done <- struct{}{}
	}()

	// Main sender loop on this goroutine
	for {
		if len(clientPubKey) > 0 {
			cryptocodec.RegisterCurrentGoroutine(clientPubKey)
		}

		select {
		case <-ctx.Done():
			return nil
		case <-done:
			cancel()
			return nil
		case mote := <-agentSendCh:
			portalLog.TraceContext(ctx, "sending mote to agent over gRPC", "stream_id", mote.GetStreamId(), "seq_id", mote.GetSeqId())
			if err := gstream.Send(&c2pb.CreatePortalResponse{
				Mote: mote,
			}); err != nil {
				portalLog.ErrorContext(ctx, "failed to send mote to agent over gRPC", "error", err)
				cancel()
				return err
			}
		}
	}
}

func sendPortalOutput(ctx context.Context, portalID int, gstream c2pb.C2_CreatePortalServer, mux *mux.Mux, sshMux *ssh.Mux) {
	portalOutTopic := mux.TopicOut(portalID)
	portalLog.DebugContext(ctx, "starting agent portal output pump", "topic", portalOutTopic)
	defer portalLog.DebugContext(ctx, "stopped agent portal output pump", "topic", portalOutTopic)

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
			if err == io.EOF || status.Code(err) == codes.Canceled || ctx.Err() != nil {
				portalLog.DebugContext(ctx, "agent portal gRPC stream closed", "error", err)
			} else {
				portalLog.WarnContext(ctx, "agent portal gRPC stream recv error", "error", err)
			}
			break
		}
		mote := req.GetMote()
		if mote == nil {
			continue
		}

		// Skip keepalive motes
		if bytesMote := mote.GetBytes(); bytesMote != nil && bytesMote.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_KEEPALIVE {
			portalLog.TraceContext(ctx, "received keepalive mote from agent")
			continue
		}

		// Route agent TCP/Close motes belonging to gRPC-side SSH sessions directly to the SSH mux
		if sshMux != nil && sshMux.RouteAgentMote(mote) {
			portalLog.TraceContext(ctx, "routed agent mote to SSH mux", "stream_id", mote.GetStreamId(), "seq_id", mote.GetSeqId())
			continue
		}

		// TRACE: Server Agent Recv
		if traced, err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_AGENT_RECV); err != nil {
			portalLog.ErrorContext(ctx, "failed to add trace event (Server Agent Recv)", "error", err)
		} else {
			mote = traced
		}

		// TRACE: Server Agent Pub
		if traced, err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_AGENT_PUB); err != nil {
			portalLog.ErrorContext(ctx, "failed to add trace event (Server Agent Pub)", "error", err)
		} else {
			mote = traced
		}

		// Publish to portal output topic
		portalLog.TraceContext(ctx, "publishing agent mote to portal output topic", "topic", portalOutTopic, "stream_id", mote.GetStreamId(), "seq_id", mote.GetSeqId())
		if err := mux.Publish(ctx, portalOutTopic, mote); err != nil {
			portalLog.ErrorContext(ctx, "failed to publish mote to portal output topic",
				"topic", portalOutTopic,
				"error", err,
			)
		}
	}
}

func sendPortalInput(ctx context.Context, portalID int, sendToAgent ssh.AgentSenderFunc, sshMux *ssh.Mux, recv <-chan *portalpb.Mote) {
	portalLog.DebugContext(ctx, "starting agent portal input pump")
	defer portalLog.DebugContext(ctx, "stopped agent portal input pump")

	ticker := time.NewTicker(keepAlivePingInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			portalLog.TraceContext(ctx, "sending keepalive ping to agent")
			if err := sendToAgent(&portalpb.Mote{
				Payload: &portalpb.Mote_Bytes{
					Bytes: &portalpb.BytesPayload{
						Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_KEEPALIVE,
					},
				},
			}); err != nil && ctx.Err() == nil {
				portalLog.ErrorContext(ctx, "portal failed to send gRPC keep alive ping", "error", err)
			}
		case mote := <-recv:
			if mote == nil {
				continue
			}

			// Intercept SSH portal motes and negotiate/manage SSH on this gRPC Tavern instance
			if mote.GetSsh() != nil && sshMux != nil {
				portalLog.TraceContext(ctx, "intercepted SSH mote for gRPC SSH mux", "stream_id", mote.GetStreamId(), "seq_id", mote.GetSeqId())
				sshMux.HandleMote(ctx, mote)
				continue
			}

			// TRACE: Server Agent Sub
			if traced, err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_AGENT_SUB); err != nil {
				portalLog.ErrorContext(ctx, "failed to add trace event (Server Agent Sub)", "error", err)
			} else {
				mote = traced
			}

			// TRACE: Server Agent Send
			if traced, err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_AGENT_SEND); err != nil {
				portalLog.ErrorContext(ctx, "failed to add trace event (Server Agent Send)", "error", err)
			} else {
				mote = traced
			}

			portalLog.TraceContext(ctx, "queuing portal input mote for agent", "stream_id", mote.GetStreamId(), "seq_id", mote.GetSeqId())
			if err := sendToAgent(mote); err != nil && ctx.Err() == nil {
				portalLog.ErrorContext(ctx, "failed to send portal input message", "error", err)
			}

			// Check for close message indicating portal termination
			if payload := mote.GetBytes(); payload != nil && payload.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
				if mote.GetStreamId() == "" {
					portalLog.DebugContext(ctx, "received portal close mote, disconnecting agent")
					return
				}
			}
		}
	}
}

func sendPortalClose(ctx context.Context, graph *ent.Client, mux *mux.Mux, portalID int, auditArgs []any) {
	portalLog.DebugContext(ctx, "closing agent portal")
	logging.Audit(ctx, "portal", "portal.closed", "agent closed portal", auditArgs...)

	// Update DB to Closed
	if err := graph.Portal.UpdateOneID(portalID).
		SetClosedAt(time.Now()).
		Exec(ctx); err != nil {
		portalLog.ErrorContext(ctx, "failed to update portal closed_at", "error", err)
	}

	// Close all open ShellPivots associated with this portal
	now := time.Now()
	pivots, err := graph.ShellPivot.Query().
		Where(
			shellpivot.HasPortalWith(portal.ID(portalID)),
			shellpivot.ClosedAtIsNil(),
		).
		All(ctx)
	if err != nil {
		portalLog.ErrorContext(ctx, "failed to query open shell pivots for portal", "error", err)
	}

	portalOutTopic := mux.TopicOut(portalID)

	for _, p := range pivots {
		// Mark pivot as closed in DB
		if err := graph.ShellPivot.UpdateOneID(p.ID).
			SetClosedAt(now).
			Exec(ctx); err != nil {
			portalLog.ErrorContext(ctx, "failed to close shell pivot",
				"pivot_id", p.ID,
				"error", err,
			)
		}

		// Send a stream-specific CLOSE mote so the SSH/PTY session receives the shutdown signal
		if err := mux.Publish(ctx, portalOutTopic, &portalpb.Mote{
			StreamId: p.StreamID,
			Payload: &portalpb.Mote_Bytes{
				Bytes: &portalpb.BytesPayload{
					Data: []byte("portal closed"),
					Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE,
				},
			},
		}); err != nil {
			portalLog.ErrorContext(ctx, "failed to send close mote to shell pivot",
				"pivot_id", p.ID,
				"stream_id", p.StreamID,
				"error", err,
			)
		}
	}

	// Send portal-level CLOSE mote (no stream_id) to disconnect clients
	if err := mux.Publish(ctx, portalOutTopic, &portalpb.Mote{
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Data: []byte("portal closed"),
				Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE,
			},
		},
	}); err != nil {
		portalLog.ErrorContext(ctx, "failed to notify subscribers that portal closed", "error", err)
	}
}
