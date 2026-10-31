package c2

import (
	"context"
	"log/slog"
	"sync"
	"time"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/portal"
	"realm.pub/tavern/internal/ent/shellpivot"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/portals"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/portals/ssh"
	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/tracepb"
)

// keepAlivePingInterval defines the frequency to send no-op ping messages to the stream,
// to keep it alive while waiting for input.
const (
	keepAlivePingInterval = 5 * time.Second
)

func (srv *Server) CreatePortal(gstream c2pb.C2_CreatePortalServer) error {
	// Setup Context
	ctx := gstream.Context()

	// Get initial message for registration
	registerMsg, err := gstream.Recv()
	if err != nil {
		return status.Errorf(codes.Internal, "failed to receive registration message: %v", err)
	}

	var taskID int
	var shellTaskID int
	if tc := registerMsg.GetTaskContext(); tc != nil {
		beaconID, err := srv.ValidateJWT(tc.GetJwt())
		if err != nil {
			return err
		}
		if tc.GetTaskId() <= 0 {
			return status.Errorf(codes.InvalidArgument, "invalid task id")
		}
		taskID = int(tc.GetTaskId())
		t, err := srv.graph.Task.Get(ctx, taskID)
		if ent.IsNotFound(err) {
			return status.Errorf(codes.NotFound, "task not found: %v", err)
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to load task: %v", err)
		}
		if err := srv.authorizeTaskForBeacon(ctx, t, beaconID); err != nil {
			return err
		}
	} else if stc := registerMsg.GetShellTaskContext(); stc != nil {
		beaconID, err := srv.ValidateJWT(stc.GetJwt())
		if err != nil {
			return err
		}
		if stc.GetShellTaskId() <= 0 {
			return status.Errorf(codes.InvalidArgument, "invalid shell task id")
		}
		shellTaskID = int(stc.GetShellTaskId())
		st, err := srv.graph.ShellTask.Get(ctx, shellTaskID)
		if ent.IsNotFound(err) {
			return status.Errorf(codes.NotFound, "shell task not found: %v", err)
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to load shell task: %v", err)
		}
		if err := srv.authorizeShellTaskForBeacon(ctx, st, beaconID); err != nil {
			return err
		}
	} else {
		return status.Errorf(codes.InvalidArgument, "missing context")
	}

	portalID, cleanup, err := srv.portalMux.CreatePortal(ctx, srv.graph, taskID, shellTaskID)
	if err != nil {
		slog.ErrorContext(ctx, "failed to create portal", "task_id", taskID, "shell_task_id", shellTaskID, "error", err)
		return status.Errorf(codes.Internal, "failed to create portal: %v", err)
	}
	defer cleanup()

	portalInTopic := srv.portalMux.TopicIn(portalID)
	recv, cleanup := srv.portalMux.Subscribe(portalInTopic)
	defer cleanup()

	// Send CLOSE
	defer sendPortalClose(context.Background(), srv.graph, srv.portalMux, portalID)

	// Start goroutine to subscribe to portal input and send to gRPC stream
	ctx, cancel := context.WithCancel(ctx)

	var sendMu sync.Mutex
	sendToAgent := func(mote *portalpb.Mote) error {
		sendMu.Lock()
		defer sendMu.Unlock()
		return gstream.Send(&c2pb.CreatePortalResponse{
			Mote: mote,
		})
	}

	sshManager := ssh.NewPortalManager(portalID, srv.portalMux, sendToAgent)
	defer sshManager.Close()

	done := make(chan struct{}, 2)

	go func() {
		sendPortalInput(ctx, portalID, sendToAgent, sshManager, recv)
		done <- struct{}{}
	}()

	// Send portal output from gRPC stream to portal output topic
	go func() {
		sendPortalOutput(ctx, portalID, gstream, srv.portalMux, sshManager)
		done <- struct{}{}
	}()

	select {
	case <-ctx.Done():
	case <-done:
	}

	// Cleanup
	cancel()

	return nil
}

func sendPortalOutput(ctx context.Context, portalID int, gstream c2pb.C2_CreatePortalServer, mux *mux.Mux, sshManager *ssh.PortalManager) {
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

		// Route agent TCP/Close motes belonging to gRPC-side SSH sessions directly to the SSH manager
		if sshManager != nil && sshManager.RouteAgentMote(mote) {
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

func sendPortalInput(ctx context.Context, portalID int, sendToAgent ssh.AgentSenderFunc, sshManager *ssh.PortalManager, recv <-chan *portalpb.Mote) {
	ticker := time.NewTicker(keepAlivePingInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if err := sendToAgent(&portalpb.Mote{
				Payload: &portalpb.Mote_Bytes{
					Bytes: &portalpb.BytesPayload{
						Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_KEEPALIVE,
					},
				},
			}); err != nil {
				slog.ErrorContext(ctx, "portal failed to send gRPC keep alive ping", "error", err)
			}
		case mote := <-recv:
			if mote == nil {
				continue
			}

			// Intercept SSH portal motes and negotiate/manage SSH on this gRPC Tavern instance
			if mote.GetSsh() != nil && sshManager != nil {
				sshManager.HandleMote(ctx, mote)
				continue
			}

			// TRACE: Server Agent Sub
			if err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_AGENT_SUB); err != nil {
				slog.ErrorContext(ctx, "failed to add trace event (Server Agent Sub)", "error", err)
			}

			// TRACE: Server Agent Send
			if err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_AGENT_SEND); err != nil {
				slog.ErrorContext(ctx, "failed to add trace event (Server Agent Send)", "error", err)
			}

			if err := sendToAgent(mote); err != nil {
				slog.ErrorContext(ctx, "failed to send portal input message",
					"portal_id", portalID,
					"error", err,
				)
			}

			// Check for close message indicating portal termination
			if payload := mote.GetBytes(); payload != nil && payload.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
				if mote.GetStreamId() == "" {
					slog.InfoContext(ctx, "received portal close, disconnecting agent", "portal_id", portalID)
					return
				}
			}
		}
	}
}

func sendPortalClose(ctx context.Context, graph *ent.Client, mux *mux.Mux, portalID int) {
	// Log portal close with beacon and host info
	logAttrs := []any{"portal_id", portalID}
	if p, err := graph.Portal.Get(ctx, portalID); err == nil {
		if b, err := p.QueryBeacon().WithHost().Only(ctx); err == nil {
			logAttrs = append(logAttrs, "beacon_id", b.ID)
			if b.Edges.Host != nil {
				logAttrs = append(logAttrs, "host_id", b.Edges.Host.ID)
			}
		} else {
			slog.WarnContext(ctx, "failed to query beacon for portal close log", "portal_id", portalID, "error", err)
		}
	} else {
		slog.WarnContext(ctx, "failed to query portal for close log", "portal_id", portalID, "error", err)
	}
	slog.InfoContext(ctx, "portal closed", logAttrs...)

	// Update DB to Closed
	if err := graph.Portal.UpdateOneID(portalID).
		SetClosedAt(time.Now()).
		Exec(context.Background()); err != nil {
		slog.ErrorContext(ctx, "failed to update portal closed_at",
			"portal_id", portalID,
			"error", err,
		)
	}

	// Close all open ShellPivots associated with this portal
	now := time.Now()
	pivots, err := graph.ShellPivot.Query().
		Where(
			shellpivot.HasPortalWith(portal.ID(portalID)),
			shellpivot.ClosedAtIsNil(),
		).
		All(context.Background())
	if err != nil {
		slog.ErrorContext(ctx, "failed to query open shell pivots for portal",
			"portal_id", portalID,
			"error", err,
		)
	}

	portalOutTopic := mux.TopicOut(portalID)

	for _, p := range pivots {
		// Mark pivot as closed in DB
		if err := graph.ShellPivot.UpdateOneID(p.ID).
			SetClosedAt(now).
			Exec(context.Background()); err != nil {
			slog.ErrorContext(ctx, "failed to close shell pivot",
				"portal_id", portalID,
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
			slog.ErrorContext(ctx, "failed to send close mote to shell pivot",
				"portal_id", portalID,
				"pivot_id", p.ID,
				"stream_id", p.StreamID,
				"error", err,
			)
		}
	}

	// Send portal-level CLOSE mote (no stream_id) to disconnect the agent
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
