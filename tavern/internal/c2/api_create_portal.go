package c2

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
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/portal/portalpb"
)

func (srv *Server) CreatePortal(gstream c2pb.C2_CreatePortalServer) error {
	// Setup Context
	ctx := gstream.Context()

	// Get initial message for registration
	registerMsg, err := gstream.Recv()
	if err != nil {
		return status.Errorf(codes.Internal, "failed to receive registration message: %v", err)
	}

	// Load Relevant Ents
	task, err := srv.graph.Task.Get(ctx, int(registerMsg.TaskId))
	if err != nil {
		if ent.IsNotFound(err) {
			slog.ErrorContext(ctx, "Create portal failed: associated task does not exist", "task_id", registerMsg.TaskId, "error", err)
			return status.Errorf(codes.NotFound, "task does not exist (task_id=%d)", registerMsg.TaskId)
		}
		slog.ErrorContext(ctx, "Create portal failed: could not load associated task", "task_id", registerMsg.TaskId, "error", err)
		return status.Errorf(codes.Internal, "failed to load task ent (task_id=%d): %v", registerMsg.TaskId, err)
	}
	beacon, err := task.Beacon(ctx)
	if err != nil {
		slog.ErrorContext(ctx, "Create portal failed: could not load associated beacon", "task_id", registerMsg.TaskId, "error", err)
		return status.Errorf(codes.Internal, "failed to load beacon ent (task_id=%d): %v", registerMsg.TaskId, err)
	}
	quest, err := task.Quest(ctx)
	if err != nil {
		slog.ErrorContext(ctx, "Create portal failed: could not load associated quest", "task_id", registerMsg.TaskId, "error", err)
		return status.Errorf(codes.Internal, "failed to load quest ent (task_id=%d): %v", registerMsg.TaskId, err)
	}
	creator, err := quest.Creator(ctx)
	if err != nil {
		slog.ErrorContext(ctx, "Create portal failed: could not load associated quest creator", "task_id", registerMsg.TaskId, "error", err)
		return status.Errorf(codes.Internal, "failed to load quest creator (task_id=%d): %v", registerMsg.TaskId, err)
	}

	// Create the Portal Entity
	portal, err := srv.graph.Portal.Create().
		SetOwner(creator).
		SetBeacon(beacon).
		SetTask(task).
		Save(ctx)
	if err != nil {
		slog.ErrorContext(ctx, "Create portal failed: could not Create entity", "task_id", registerMsg.TaskId, "error", err)
		return status.Errorf(codes.Internal, "failed to Create portal: %v", err)
	}
	portalID := portal.ID

	// Log Portal Session
	slog.InfoContext(ctx, "started gRPC portal ✨",
		"portal_id", portalID,
		"task_id", registerMsg.TaskId,
		"creator_id", creator.ID,
	)
	defer func(start time.Time) {
		slog.InfoContext(ctx, "closed gRPC portal",
			"started_at", start.String(),
			"ended_at", time.Now().String(),
			"duration", time.Since(start).String(),
			"portal_id", portalID,
			"task_id", registerMsg.TaskId,
			"creator_id", creator.ID,
		)
	}(time.Now())

	// Create new Stream
	pubsubStream := stream.New(fmt.Sprintf("%d", portalID))

	// Cleanup
	defer func() {
		closedAt := time.Now()

		// Prepare New Context
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()

		// Notify Subscribers that the stream is closed
		slog.DebugContext(ctx, "portal closed, sending stream close message", "portal_id", portal.ID)
		if err := pubsubStream.SendMessage(ctx, &pubsub.Message{
			Metadata: map[string]string{
				stream.MetadataStreamClose: fmt.Sprintf("%d", portalID),
			},
		}, srv.portalRelayMux); err != nil {
			slog.ErrorContext(ctx, "portal closed and failed to notify subscribers",
				"portal_id", portal.ID,
				"error", err,
			)
		}

		// Update Ent
		portal, err := srv.graph.Portal.Get(ctx, portalID)
		if err != nil {
			slog.ErrorContext(ctx, "portal closed and failed to load ent for updates",
				"error", err,
				"portal_id", portalID,
			)
			return
		}
		if _, err := portal.Update().
			SetClosedAt(closedAt).
			Save(ctx); err != nil {
			slog.ErrorContext(ctx, "portal closed and failed to update ent",
				"error", err,
				"portal_id", portal.ID,
			)
		}
	}()

	// Register stream with Mux
	srv.portalRelayMux.Register(pubsubStream)
	defer srv.portalRelayMux.Unregister(pubsubStream)

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
	err = sendPortalOutput(ctx, portalID, gstream, pubsubStream, srv.portalRelayMux)

	wg.Wait()

	return err
}

func sendPortalInput(ctx context.Context, portalID int, gstream c2pb.C2_CreatePortalServer, pubsubStream *stream.Stream) {
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
			if err := gstream.Send(&c2pb.CreatePortalResponse{
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

func sendPortalOutput(ctx context.Context, portalID int, gstream c2pb.C2_CreatePortalServer, pubsubStream *stream.Stream, mux *stream.Mux) error {
	for {
		req, err := gstream.Recv()
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to receive portal request: %v", err)
		}

		data, err := proto.Marshal(req.Payload)
		if err != nil {
			return status.Errorf(codes.Internal, "failed to marshal portal request: %v", err)
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

func sendPortalKeepAlives(ctx context.Context, gstream c2pb.C2_CreatePortalServer) {
	ticker := time.NewTicker(keepAlivePingInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if err := gstream.Send(&c2pb.CreatePortalResponse{
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
