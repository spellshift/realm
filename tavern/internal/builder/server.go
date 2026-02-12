package builder

import (
	"bytes"
	"context"
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"io"
	"log/slog"
	"strings"
	"time"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/ent"
	entbuilder "realm.pub/tavern/internal/ent/builder"
	"realm.pub/tavern/internal/ent/buildtask"
)

// Server implements the Builder gRPC service.
type Server struct {
	graph        *ent.Client
	serverPubkey string
	builderpb.UnimplementedBuilderServer
}

// New creates a new Builder gRPC server.
// serverPubkey is the base64-encoded X25519 server public key embedded into agent builds.
func New(graph *ent.Client, serverPubkey string) *Server {
	return &Server{
		graph:        graph,
		serverPubkey: serverPubkey,
	}
}

// ClaimBuildTasks returns unclaimed build tasks assigned to the authenticated builder and marks them as claimed.
func (s *Server) ClaimBuildTasks(ctx context.Context, req *builderpb.ClaimBuildTasksRequest) (*builderpb.ClaimBuildTasksResponse, error) {
	now := time.Now()

	// Extract authenticated builder from context
	b, ok := BuilderFromContext(ctx)
	if !ok {
		return nil, status.Error(codes.Unauthenticated, "builder not authenticated")
	}

	// Update builder's last_seen_at timestamp
	if _, err := s.graph.Builder.UpdateOne(b).
		SetLastSeenAt(now).
		Save(ctx); err != nil {
		slog.ErrorContext(ctx, "failed to update builder last_seen_at",
			"builder_id", b.ID, "error", err)
	}

	// Load unclaimed build tasks assigned to this builder
	tasks, err := s.graph.BuildTask.Query().
		Where(
			buildtask.HasBuilderWith(entbuilder.ID(b.ID)),
			buildtask.ClaimedAtIsNil(),
		).
		All(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to query build tasks: %v", err)
	}

	// Prepare transaction for claiming tasks
	tx, err := s.graph.Tx(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to initialize transaction: %v", err)
	}
	client := tx.Client()

	// Rollback transaction if we panic
	defer func() {
		if v := recover(); v != nil {
			tx.Rollback()
			panic(v)
		}
	}()

	// Update all ClaimedAt timestamps to claim tasks
	taskIDs := make([]int, 0, len(tasks))
	for _, t := range tasks {
		_, err := client.BuildTask.UpdateOne(t).
			SetClaimedAt(now).
			Save(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to update build task %d: %w", t.ID, err))
		}
		taskIDs = append(taskIDs, t.ID)
	}

	// Commit the transaction
	if err := tx.Commit(); err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to commit transaction: %w", err))
	}

	// Load the tasks with our non-transactional client (cannot use transaction after commit)
	resp := &builderpb.ClaimBuildTasksResponse{}
	resp.Tasks = make([]*builderpb.BuildTaskSpec, 0, len(taskIDs))
	for _, taskID := range taskIDs {
		claimedTask, err := s.graph.BuildTask.Get(ctx, taskID)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to load claimed build task (but it was still claimed) %d: %v", taskID, err)
		}

		// Derive the IMIX config YAML from the build task's stored fields.
		imixCfg := ImixConfig{
			ServerPubkey: s.serverPubkey,
			Transports: []ImixTransportConfig{
				{
					URI: claimedTask.CallbackURI,
					Interval:    claimedTask.Interval,
					Type:        TransportTypeToString(claimedTask.TransportType),
					Extra:       claimedTask.Extra,
				},
			},
		}
		imixCfgYAML, err := MarshalImixConfig(imixCfg)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to marshal IMIX config for task %d: %v", taskID, err)
		}

		resp.Tasks = append(resp.Tasks, &builderpb.BuildTaskSpec{
			Id:           int64(claimedTask.ID),
			TargetOs:     claimedTask.TargetOs.String(),
			BuildImage:   claimedTask.BuildImage,
			BuildScript:  claimedTask.BuildScript,
			ArtifactPath: claimedTask.ArtifactPath,
			Env:          []string{fmt.Sprintf("IMIX_CONFIG=%s", imixCfgYAML)},
		})
	}

	slog.InfoContext(ctx, "builder claimed build tasks",
		"builder_id", b.ID,
		"task_count", len(resp.Tasks),
	)

	return resp, nil
}

// StreamBuildTaskOutput receives a stream of build output messages from the builder client.
// Each message is flushed to the database immediately. On the final message (finished=true)
// or stream close, the task is finalized.
func (s *Server) StreamBuildTaskOutput(stream builderpb.Builder_StreamBuildTaskOutputServer) error {
	ctx := stream.Context()

	b, ok := BuilderFromContext(ctx)
	if !ok {
		return status.Error(codes.Unauthenticated, "builder not authenticated")
	}

	var (
		taskID   int64
		finished bool
	)

	for {
		req, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to receive stream message: %v", err)
		}

		// First message: validate task ownership and set started_at.
		if taskID == 0 {
			taskID = req.TaskId
			if taskID == 0 {
				return status.Error(codes.InvalidArgument, "first message must include task_id")
			}

			bt, err := s.graph.BuildTask.Get(ctx, int(taskID))
			if err != nil {
				return status.Errorf(codes.NotFound, "build task %d not found: %v", taskID, err)
			}
			btBuilder, err := bt.QueryBuilder().Only(ctx)
			if err != nil {
				return status.Errorf(codes.Internal, "failed to query builder for task %d: %v", taskID, err)
			}
			if btBuilder.ID != b.ID {
				return status.Errorf(codes.PermissionDenied, "build task %d is not assigned to this builder", taskID)
			}

			// Idempotency: if already finished, return success.
			if !bt.FinishedAt.IsZero() {
				slog.WarnContext(ctx, "duplicate stream for already-finished build task",
					"task_id", taskID, "builder_id", b.ID)
				return stream.SendAndClose(&builderpb.StreamBuildTaskOutputResponse{})
			}

			// Set started_at if not already set.
			if bt.StartedAt.IsZero() {
				if _, err := s.graph.BuildTask.UpdateOne(bt).
					SetStartedAt(time.Now()).
					Save(ctx); err != nil {
					slog.ErrorContext(ctx, "failed to set started_at",
						"task_id", taskID, "error", err)
				}
			}
		}

		if req.TaskId != 0 && req.TaskId != taskID {
			return status.Errorf(codes.InvalidArgument, "task_id changed mid-stream: got %d, expected %d", req.TaskId, taskID)
		}

		if req.Finished {
			finished = true
		}

		// Flush every message to the database immediately.
		if req.Output != "" || req.Error != "" || finished {
			if err := s.flushStreamOutput(ctx, int(taskID), req.Output, req.Error, finished, req.ExitCode); err != nil {
				return status.Errorf(codes.Internal, "failed to flush build output for task %d: %v", taskID, err)
			}
		}

		if finished {
			break
		}
	}

	if taskID == 0 {
		return status.Error(codes.InvalidArgument, "no messages received")
	}

	slog.InfoContext(ctx, "build task stream completed",
		"task_id", taskID,
		"builder_id", b.ID,
		"finished", finished,
	)

	return stream.SendAndClose(&builderpb.StreamBuildTaskOutputResponse{})
}

// UploadBuildArtifact receives a chunked binary artifact stream from the builder client
// and creates an Asset entity.
func (s *Server) UploadBuildArtifact(stream builderpb.Builder_UploadBuildArtifactServer) error {
	ctx := stream.Context()

	b, ok := BuilderFromContext(ctx)
	if !ok {
		return status.Error(codes.Unauthenticated, "builder not authenticated")
	}

	var (
		taskID       int64
		artifactName string
		buf          bytes.Buffer
	)

	for {
		req, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to receive artifact chunk: %v", err)
		}

		// First message: validate task ownership.
		if taskID == 0 {
			taskID = req.TaskId
			if taskID == 0 {
				return status.Error(codes.InvalidArgument, "first message must include task_id")
			}
			artifactName = req.ArtifactName
			if artifactName == "" {
				artifactName = fmt.Sprintf("artifact-%d", taskID)
			}

			bt, err := s.graph.BuildTask.Get(ctx, int(taskID))
			if err != nil {
				return status.Errorf(codes.NotFound, "build task %d not found: %v", taskID, err)
			}
			btBuilder, err := bt.QueryBuilder().Only(ctx)
			if err != nil {
				return status.Errorf(codes.Internal, "failed to query builder for task %d: %v", taskID, err)
			}
			if btBuilder.ID != b.ID {
				return status.Errorf(codes.PermissionDenied, "build task %d is not assigned to this builder", taskID)
			}
		}

		buf.Write(req.Chunk)
	}

	if taskID == 0 {
		return status.Error(codes.InvalidArgument, "no messages received")
	}

	if buf.Len() == 0 {
		return status.Error(codes.InvalidArgument, "empty artifact")
	}

	// Load build task to get target_os and target_format for the asset name.
	bt, err := s.graph.BuildTask.Get(ctx, int(taskID))
	if err != nil {
		return status.Errorf(codes.Internal, "failed to load build task for asset naming: %v", err)
	}

	// Generate 6 random bytes for a unique suffix.
	randomBytes := make([]byte, 3)
	if _, err := rand.Read(randomBytes); err != nil {
		return status.Errorf(codes.Internal, "failed to generate random suffix: %v", err)
	}
	randomSuffix := hex.EncodeToString(randomBytes)

	osName := strings.ToLower(strings.TrimPrefix(bt.TargetOs.String(), "PLATFORM_"))
	formatName := strings.ToLower(strings.TrimPrefix(bt.TargetFormat.String(), "TARGET_FORMAT_"))
	assetName := fmt.Sprintf("build/%s/%s/imix-%s", osName, formatName, randomSuffix)
	asset, err := s.graph.Asset.Create().
		SetName(assetName).
		SetContent(buf.Bytes()).
		Save(ctx)
	if err != nil {
		return status.Errorf(codes.Internal, "failed to create asset: %v", err)
	}

	slog.InfoContext(ctx, "build artifact uploaded",
		"task_id", taskID,
		"builder_id", b.ID,
		"asset_id", asset.ID,
		"size", buf.Len(),
	)

	return stream.SendAndClose(&builderpb.UploadBuildArtifactResponse{
		AssetId: int64(asset.ID),
	})
}

// flushStreamOutput appends output and error to the build task in the database.
// If finished is true, it also sets finished_at and exit_code to mark the task as complete.
func (s *Server) flushStreamOutput(ctx context.Context, taskID int, output string, errMsg string, finished bool, exitCode int64) error {
	bt, err := s.graph.BuildTask.Get(ctx, taskID)
	if err != nil {
		return fmt.Errorf("failed to load build task %d: %w", taskID, err)
	}

	// Append new content to any existing output already stored from previous flushes.
	newOutput := output
	if bt.Output != "" && newOutput != "" {
		newOutput = bt.Output + "\n" + newOutput
	} else if bt.Output != "" {
		newOutput = bt.Output
	}

	newError := errMsg
	if bt.Error != "" && newError != "" {
		newError = bt.Error + "\n" + newError
	} else if bt.Error != "" {
		newError = bt.Error
	}

	update := s.graph.BuildTask.UpdateOne(bt).
		SetOutput(newOutput)
	if newError != "" {
		update = update.SetError(newError)
	}
	if finished {
		update = update.SetFinishedAt(time.Now()).
			SetExitCode(int(exitCode))
	}

	if _, err := update.Save(ctx); err != nil {
		return fmt.Errorf("failed to update build task %d: %w", taskID, err)
	}

	return nil
}
