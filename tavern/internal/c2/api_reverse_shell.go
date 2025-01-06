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
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
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

	// Get initial message for registration
	registerMsg, err := gstream.Recv()
	if err != nil {
		return status.Errorf(codes.Internal, "failed to receive registration message: %v", err)
	}

	// Load Relevant Ents
	task, err := srv.graph.Task.Get(ctx, int(registerMsg.TaskId))
	if err != nil {
		if ent.IsNotFound(err) {
			slog.ErrorContext(ctx, "reverse shell failed: associated task does not exist", "task_id", registerMsg.TaskId, "error", err)
			return status.Errorf(codes.NotFound, "task does not exist (task_id=%d)", registerMsg.TaskId)
		}
		slog.ErrorContext(ctx, "reverse shell failed: could not load associated task", "task_id", registerMsg.TaskId, "error", err)
		return status.Errorf(codes.Internal, "failed to load task ent (task_id=%d): %v", registerMsg.TaskId, err)
	}
	beacon, err := task.Beacon(ctx)
	if err != nil {
		slog.ErrorContext(ctx, "reverse shell failed: could not load associated beacon", "task_id", registerMsg.TaskId, "error", err)
		return status.Errorf(codes.Internal, "failed to load beacon ent (task_id=%d): %v", registerMsg.TaskId, err)
	}
	quest, err := task.Quest(ctx)
	if err != nil {
		slog.ErrorContext(ctx, "reverse shell failed: could not load associated quest", "task_id", registerMsg.TaskId, "error", err)
		return status.Errorf(codes.Internal, "failed to load quest ent (task_id=%d): %v", registerMsg.TaskId, err)
	}
	creator, err := quest.Creator(ctx)
	if err != nil {
		slog.ErrorContext(ctx, "reverse shell failed: could not load associated quest creator", "task_id", registerMsg.TaskId, "error", err)
		return status.Errorf(codes.Internal, "failed to load quest creator (task_id=%d): %v", registerMsg.TaskId, err)
	}

	// Create the Shell Entity
	shell, err := srv.graph.Shell.Create().
		SetOwner(creator).
		SetBeacon(beacon).
		SetTask(task).
		SetData([]byte{}).
		Save(ctx)
	if err != nil {
		slog.ErrorContext(ctx, "reverse shell failed: could not create shell entity", "task_id", registerMsg.TaskId, "error", err)
		return status.Errorf(codes.Internal, "failed to create shell: %v", err)
	}
	shellID := shell.ID

	// Log Shell Session
	slog.InfoContext(ctx, "started gRPC reverse shell",
		"shell_id", shellID,
		"task_id", registerMsg.TaskId,
		"creator_id", creator.ID,
	)
	defer func(start time.Time) {
		slog.InfoContext(ctx, "closed gRPC reverse shell",
			"started_at", start.String(),
			"ended_at", time.Now().String(),
			"duration", time.Since(start).String(),
			"shell_id", shellID,
			"task_id", registerMsg.TaskId,
			"creator_id", creator.ID,
		)
	}(time.Now())

	// Create new Stream
	pubsubStream := stream.New(fmt.Sprintf("%d", shellID))

	// Cleanup
	defer func() {
		closedAt := time.Now()

		// Prepare New Context
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()

		// Notify Subscribers that the stream is closed
		slog.DebugContext(ctx, "reverse shell closed, sending stream close message", "shell_id", shell.ID)
		if err := pubsubStream.SendMessage(ctx, &pubsub.Message{
			Metadata: map[string]string{
				stream.MetadataStreamClose: fmt.Sprintf("%d", shellID),
			},
		}, srv.mux); err != nil {
			slog.ErrorContext(ctx, "reverse shell closed and failed to notify subscribers",
				"shell_id", shell.ID,
				"error", err,
			)
		}

		// Update Ent
		shell, err := srv.graph.Shell.Get(ctx, shellID)
		if err != nil {
			slog.ErrorContext(ctx, "reverse shell closed and failed to load ent for updates",
				"error", err,
				"shell_id", shell.ID,
			)
			return
		}
		if _, err := shell.Update().
			SetClosedAt(closedAt).
			Save(ctx); err != nil {
			slog.ErrorContext(ctx, "reverse shell closed and failed to update ent",
				"error", err,
				"shell_id", shell.ID,
			)
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
		sendShellInput(ctx, shellID, gstream, pubsubStream)
	}()

	// Send Output (to pubsub)
	err = sendShellOutput(ctx, shellID, gstream, pubsubStream, srv.mux)

	wg.Wait()

	return err
}

func sendShellInput(ctx context.Context, shellID int, gstream c2pb.C2_ReverseShellServer, pubsubStream *stream.Stream) {
	for {
		select {
		case <-ctx.Done():
			return
		case msg := <-pubsubStream.Messages():
			msgLen := len(msg.Body)
			if err := gstream.Send(&c2pb.ReverseShellResponse{
				Kind: c2pb.ReverseShellMessageKind_REVERSE_SHELL_MESSAGE_KIND_DATA,
				Data: msg.Body,
			}); err != nil {
				slog.ErrorContext(ctx, "failed to send shell input to reverse shell",
					"shell_id", shellID,
					"msg_len", msgLen,
					"error", err,
				)
				return
			}
			slog.DebugContext(ctx, "reverse shell sent input to agent via gRPC",
				"shell_id", shellID,
				"msg_len", msgLen,
			)
		}
	}
}

func sendShellOutput(ctx context.Context, shellID int, gstream c2pb.C2_ReverseShellServer, pubsubStream *stream.Stream, mux *stream.Mux) error {
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
		msgLen := len(req.Data)
		if err := pubsubStream.SendMessage(ctx, &pubsub.Message{
			Body: req.Data,
		}, mux); err != nil {
			slog.ErrorContext(ctx, "reverse shell failed to publish shell output",
				"shell_id", shellID,
				"msg_len", msgLen,
				"error", err,
			)
			return status.Errorf(codes.Internal, "failed to publish message: %v", err)
		}
		slog.DebugContext(ctx, "reverse shell published shell output",
			"shell_id", shellID,
			"msg_len", msgLen,
		)
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
				slog.ErrorContext(ctx, "reverse shell failed to send gRPC keep alive ping", "error", err)
			}
		}
	}
}
