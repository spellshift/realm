package builder

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"strings"
	"time"
	"unicode/utf8"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	yaml "gopkg.in/yaml.v3"

	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
	entbuilder "realm.pub/tavern/internal/ent/builder"
	"realm.pub/tavern/internal/ent/buildtask"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/namegen"
)

var serverLog = logging.For("builder.server")

const maxBuilderHostnameLen = 255

func sanitizeBuilderHostname(raw string) string {
	hostname := strings.TrimSpace(raw)
	if len(hostname) <= maxBuilderHostnameLen {
		return hostname
	}
	// Truncate to at most maxBuilderHostnameLen bytes on a valid UTF-8 rune
	// boundary so MySQL varchar(255) never rejects the update and poisons last_seen_at.
	n := maxBuilderHostnameLen
	for n > 0 && !utf8.RuneStart(hostname[n]) {
		n--
	}
	return strings.TrimSpace(hostname[:n])
}

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

	// Update builder's last_seen_at timestamp and its reported hostname.
	update := s.graph.Builder.UpdateOne(b).SetLastSeenAt(now)
	if hostname := sanitizeBuilderHostname(req.GetHostname()); hostname != "" {
		if b.Hostname == nil || *b.Hostname != hostname {
			update.SetHostname(hostname)
		}
	}
	if _, err := update.Save(ctx); err != nil {
		serverLog.ErrorContext(ctx, "failed to update builder heartbeat",
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

	// Legacy tasks lack historical inputs and cannot safely be executed.
	eligible := tasks[:0]
	for _, task := range tasks {
		if task.ProfileAtCreation == nil {
			serverLog.WarnContext(ctx, "build task has no saved profile; recreate it to execute", "task_id", task.ID)
			continue
		}
		eligible = append(eligible, task)
	}
	tasks = eligible

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

		// Derive the IMIX config YAML from the build task's stored transports.
		profile := claimedTask.ProfileAtCreation
		transports := profile.Transports

		imixTransports := make([]ImixTransportConfig, len(transports))
		for i, t := range transports {
			imixTransports[i] = ImixTransportConfig{
				URI:      t.URI,
				Interval: t.Interval,
				Type:     TransportTypeToString(t.Type),
				Extra:    t.Extra,
				Jitter:   t.Jitter,
			}
		}
		imixCfg := ImixConfig{
			ServerPubkey: s.serverPubkey,
			Transports:   imixTransports,
		}
		cfgBytes, err := yaml.Marshal(imixCfg)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal IMIX config: %w", err)
		}

		// Build tome metadata (IDs, names, params) for the response.
		// Tome content is downloaded separately via DownloadTome RPC.
		var protoTomes []*builderpb.Tome
		for _, pt := range profile.Tomes {
			protoTomes = append(protoTomes, &builderpb.Tome{
				Id:     int64(pt.TomeID),
				Name:   pt.Name,
				Params: pt.Params,
			})
		}

		envVars := []string{
			fmt.Sprintf("IMIX_CONFIG=%s", string(cfgBytes)),
			fmt.Sprintf("ARTIFACT_PATH=%s", string(claimedTask.ArtifactPath)),
		}

		uniqueStr := claimedTask.Unique
		if uniqueStr == "" {
			uniqueStr = profile.Unique
		}
		if uniqueStr != "" {
			envVars = append(envVars, fmt.Sprintf("IMIX_UNIQUE=%s", uniqueStr))
		}

		resp.Tasks = append(resp.Tasks, &builderpb.BuildTaskSpec{
			Id:              int64(claimedTask.ID),
			TargetOs:        claimedTask.TargetOs.String(),
			BuildImage:      profile.BuildImage,
			BuildScript:     claimedTask.BuildScript,
			ArtifactPath:    claimedTask.ArtifactPath,
			PreBuildScript:  profile.Prebuildscript,
			PostBuildScript: profile.Postbuildscript,
			SetupScript:     claimedTask.Setupscript,
			Env:             envVars,
			Tomes:           protoTomes,
		})
	}

	if len(resp.Tasks) > 0 {
		serverLog.InfoContext(ctx, "builder claimed build tasks",
			"builder_id", b.ID,
			"task_count", len(resp.Tasks),
		)
	} else {
		serverLog.DebugContext(ctx, "builder polled with no pending build tasks",
			"builder_id", b.ID,
		)
	}

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
		taskID        int64
		finished      bool
		finalExitCode int64
		finalError    string
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
				serverLog.WarnContext(ctx, "duplicate stream for already-finished build task",
					"task_id", taskID, "builder_id", b.ID)
				return stream.SendAndClose(&builderpb.StreamBuildTaskOutputResponse{})
			}

			// Set started_at if not already set.
			if bt.StartedAt.IsZero() {
				if _, err := s.graph.BuildTask.UpdateOne(bt).
					SetStartedAt(time.Now()).
					Save(ctx); err != nil {
					serverLog.ErrorContext(ctx, "failed to set started_at",
						"task_id", taskID, "error", err)
				}
			}
		}

		if req.TaskId != 0 && req.TaskId != taskID {
			return status.Errorf(codes.InvalidArgument, "task_id changed mid-stream: got %d, expected %d", req.TaskId, taskID)
		}

		if req.Finished {
			finished = true
			finalExitCode = req.ExitCode
			finalError = req.Error
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

	if finished {
		if finalExitCode != 0 || finalError != "" {
			logging.AuditFailure(ctx, "builder", "build_task.failed", "build task failed",
				"build_task_id", int(taskID),
				"builder_id", b.ID,
				"exit_code", int(finalExitCode),
			)
		} else {
			logging.Audit(ctx, "builder", "build_task.completed", "build task completed",
				"build_task_id", int(taskID),
				"builder_id", b.ID,
				"exit_code", int(finalExitCode),
			)
		}
	} else {
		serverLog.InfoContext(ctx, "build task stream closed before completion",
			"task_id", taskID,
			"builder_id", b.ID,
		)
	}

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
		// bt is loaded once below, from the first stream message, to validate
		// task ownership. It's reused after the loop for target_os/target_format
		// and the saved profile instead of querying the build task a second time.
		bt *ent.BuildTask
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

			var err error
			bt, err = s.graph.BuildTask.Get(ctx, int(taskID))
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

	tx, err := s.graph.Tx(ctx)
	if err != nil {
		return status.Errorf(codes.Internal, "begin artifact transaction: %v", err)
	}
	defer tx.Rollback()
	bt, err = tx.Client().BuildTask.Get(ctx, bt.ID)
	if err != nil {
		return status.Errorf(codes.Internal, "load build task: %v", err)
	}
	artifact, err := bt.QueryArtifact().Only(ctx)
	if err == nil && artifact.Status == asset.StatusFINISHED {
		return sendFinishedArtifact(stream, artifact, buf.Bytes())
	}
	if err != nil && !ent.IsNotFound(err) {
		return status.Errorf(codes.Internal, "load build artifact: %v", err)
	}
	if bt.ProfileAtCreation == nil {
		return status.Error(codes.FailedPrecondition, "build task has no saved profile")
	}
	if !bt.FinishedAt.IsZero() && (bt.ExitCode == nil || *bt.ExitCode != 0) {
		return status.Error(codes.FailedPrecondition, "cannot upload artifact for failed build")
	}
	if err == nil {
		// Guard the transition in the database: another upload may have
		// finished this asset after it was read above.
		updated, updateErr := tx.Client().Asset.Update().
			Where(asset.ID(artifact.ID), asset.StatusNEQ(asset.StatusFINISHED)).
			SetContent(buf.Bytes()).SetStatus(asset.StatusFINISHED).Save(ctx)
		if updateErr != nil {
			return status.Errorf(codes.Internal, "save build artifact: %v", updateErr)
		}
		if updated == 0 {
			if err := tx.Rollback(); err != nil {
				return status.Errorf(codes.Internal, "rollback artifact transaction: %v", err)
			}
			return s.replyWithFinishedArtifact(ctx, stream, bt.ID, buf.Bytes())
		}
		artifact, err = tx.Client().Asset.Get(ctx, artifact.ID)
	} else if ent.IsNotFound(err) {
		profileName := strings.ReplaceAll(strings.ToLower(bt.ProfileAtCreation.Name), " ", "-")
		osName := strings.ToLower(strings.TrimPrefix(bt.TargetOs.String(), "PLATFORM_"))
		formatName := strings.ToLower(strings.TrimPrefix(bt.TargetFormat.String(), "TARGET_FORMAT_"))
		assetName := fmt.Sprintf("build/%s/%s/imix-%s-%d", osName, formatName, profileName, bt.ID)
		artifact, err = tx.Client().Asset.Create().SetName(assetName).SetContent(buf.Bytes()).Save(ctx)
		if err == nil {
			_, err = tx.Client().BuildTask.UpdateOneID(bt.ID).SetArtifact(artifact).Save(ctx)
		}
		if err != nil {
			if !ent.IsConstraintError(err) {
				return status.Errorf(codes.Internal, "save build artifact: %v", err)
			}
			// Legacy tasks have no placeholder. Concurrent uploads race to
			// create the same deterministic asset name; read the winner after
			// rolling back this transaction.
			if rollbackErr := tx.Rollback(); rollbackErr != nil {
				return status.Errorf(codes.Internal, "rollback artifact transaction: %v", rollbackErr)
			}
			return s.replyWithFinishedArtifact(ctx, stream, bt.ID, buf.Bytes())
		}
	}
	if err != nil {
		return status.Errorf(codes.Internal, "save build artifact: %v", err)
	}
	if err := tx.Commit(); err != nil {
		return status.Errorf(codes.Internal, "commit build artifact: %v", err)
	}

	auditAttrs := make([]any, 0, 12)
	auditAttrs = append(auditAttrs,
		"build_task_id", bt.ID,
		"builder_id", b.ID,
		"size", buf.Len(),
	)
	auditAttrs = append(auditAttrs, logging.AssetAttrs(artifact.ID, artifact.Name, artifact.Hash, artifact.Size)...)
	logging.Audit(ctx, "builder", "build_task.artifact_uploaded", "build artifact uploaded", auditAttrs...)

	return stream.SendAndClose(&builderpb.UploadBuildArtifactResponse{
		AssetId: int64(artifact.ID),
	})
}

func sendFinishedArtifact(stream builderpb.Builder_UploadBuildArtifactServer, artifact *ent.Asset, content []byte) error {
	if artifact.Status != asset.StatusFINISHED {
		return status.Error(codes.Aborted, "build artifact upload is still in progress; retry")
	}
	if !bytes.Equal(artifact.Content, content) {
		return status.Error(codes.FailedPrecondition, "build artifact already finished with different content")
	}
	return stream.SendAndClose(&builderpb.UploadBuildArtifactResponse{AssetId: int64(artifact.ID)})
}

func (s *Server) replyWithFinishedArtifact(ctx context.Context, stream builderpb.Builder_UploadBuildArtifactServer, taskID int, content []byte) error {
	bt, err := s.graph.BuildTask.Get(ctx, taskID)
	if err != nil {
		return status.Errorf(codes.Internal, "reload build task: %v", err)
	}
	artifact, err := bt.QueryArtifact().Only(ctx)
	if ent.IsNotFound(err) {
		return status.Error(codes.Aborted, "build artifact upload is still in progress; retry")
	}
	if err != nil {
		return status.Errorf(codes.Internal, "reload build artifact: %v", err)
	}
	return sendFinishedArtifact(stream, artifact, content)
}

// ArtifactAssetName generates the unique asset name for a build's artifact.
func ArtifactAssetName(targetOS c2pb.Host_Platform, targetFormat builderpb.TargetFormat, profileName string) string {
	profileName = strings.ReplaceAll(strings.ToLower(profileName), " ", "-")
	osName := strings.ToLower(strings.TrimPrefix(targetOS.String(), "PLATFORM_"))
	formatName := strings.ToLower(strings.TrimPrefix(targetFormat.String(), "TARGET_FORMAT_"))
	return fmt.Sprintf("build/%s/%s/imix-%s-%s", osName, formatName, profileName, namegen.NewSimple())
}

// DownloadTome streams a tome's packaged content (tar.gz) to the builder client in chunks.
// The builder must own the task referenced in the request, and the tome must be part of
// the task's build profile.
func (s *Server) DownloadTome(req *builderpb.DownloadTomeRequest, stream builderpb.Builder_DownloadTomeServer) error {
	ctx := stream.Context()

	b, ok := BuilderFromContext(ctx)
	if !ok {
		return status.Error(codes.Unauthenticated, "builder not authenticated")
	}

	if req.TaskId == 0 {
		return status.Error(codes.InvalidArgument, "task_id is required")
	}
	if req.TomeId == 0 {
		return status.Error(codes.InvalidArgument, "tome_id is required")
	}

	// Validate task ownership.
	bt, err := s.graph.BuildTask.Get(ctx, int(req.TaskId))
	if err != nil {
		return status.Errorf(codes.NotFound, "build task %d not found: %v", req.TaskId, err)
	}
	btBuilder, err := bt.QueryBuilder().Only(ctx)
	if err != nil {
		return status.Errorf(codes.Internal, "failed to query builder for task %d: %v", req.TaskId, err)
	}
	if btBuilder.ID != b.ID {
		return status.Errorf(codes.PermissionDenied, "build task %d is not assigned to this builder", req.TaskId)
	}

	// Membership, names, and content all come from creation-time inputs.
	profile := bt.ProfileAtCreation
	if profile == nil {
		return status.Error(codes.FailedPrecondition, "build task has no saved profile")
	}
	var selected *builderpb.BuildTomeSnapshot
	for i := range profile.Tomes {
		if int64(profile.Tomes[i].TomeID) == req.TomeId {
			selected = &profile.Tomes[i]
			break
		}
	}
	if selected == nil {
		return status.Errorf(codes.InvalidArgument, "tome %d is not part of the saved build profile for task %d", req.TomeId, req.TaskId)
	}
	bundle, err := bt.QueryBundle().Only(ctx)
	if err != nil {
		return status.Errorf(codes.FailedPrecondition, "load frozen build inputs: %v", err)
	}
	data, err := FrozenTome(bundle.Content, selected.TomeID)
	if err != nil {
		return status.Errorf(codes.Internal, "read frozen tome: %v", err)
	}

	// Stream the data in 1MB chunks.
	const chunkSize = 1 * 1024 * 1024
	for i := 0; i < len(data); i += chunkSize {
		end := i + chunkSize
		if end > len(data) {
			end = len(data)
		}

		msg := &builderpb.DownloadTomeResponse{
			Chunk: data[i:end],
		}
		// Include the name on the first message only.
		if i == 0 {
			msg.Name = selected.Name
		}

		if err := stream.Send(msg); err != nil {
			return status.Errorf(codes.Internal, "failed to send tome chunk: %v", err)
		}
	}

	serverLog.DebugContext(ctx, "tome downloaded",
		"task_id", req.TaskId,
		"tome_id", req.TomeId,
		"builder_id", b.ID,
		"size", len(data),
	)

	return nil
}

// flushStreamOutput appends output and error to the build task in the database.
// If finished is true, it also sets finished_at and exit_code to mark the task as complete.
func (s *Server) flushStreamOutput(ctx context.Context, taskID int, output string, errMsg string, finished bool, exitCode int64) error {
	tx, err := s.graph.Tx(ctx)
	if err != nil {
		return err
	}
	defer tx.Rollback()
	bt, err := tx.Client().BuildTask.Get(ctx, taskID)
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

	update := tx.Client().BuildTask.UpdateOne(bt).
		SetOutput(newOutput)
	if newError != "" {
		update = update.SetError(newError)
	}
	if finished {
		if errMsg != "" && exitCode == 0 {
			exitCode = -1
		}
		update = update.SetFinishedAt(time.Now()).
			SetExitCode(int(exitCode))
	}

	if _, err := update.Save(ctx); err != nil {
		return fmt.Errorf("failed to update build task %d: %w", taskID, err)
	}
	if finished && exitCode != 0 {
		artifact, err := bt.QueryArtifact().Only(ctx)
		if err == nil {
			if err := tx.Client().Asset.UpdateOne(artifact).SetStatus(asset.StatusERROR).Exec(ctx); err != nil {
				return fmt.Errorf("mark build artifact as failed: %w", err)
			}
		} else if !ent.IsNotFound(err) {
			return fmt.Errorf("load build artifact: %w", err)
		}
	}
	return tx.Commit()
}
