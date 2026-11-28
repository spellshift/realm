package builder

import (
	"context"
	"errors"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/mattn/go-sqlite3"

	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/buildprofile"
	"realm.pub/tavern/internal/ent/buildtask"
	"realm.pub/tavern/internal/logging"
)

func autoAgentBuildsDisabled() bool {
	switch strings.ToLower(strings.TrimSpace(os.Getenv("DISABLE_AUTO_AGENT_BUILDS"))) {
	case "1", "true", "yes", "on":
		return true
	default:
		return false
	}
}

// queueMissingDefaultBuilds queues each supported Default build once globally.
// The unique auto_build_key protects the check and creation across Tavern
// processes and concurrent builder polls. Existing manual tasks also count.
func (s *Server) queueMissingDefaultBuilds(ctx context.Context, pollingBuilder *ent.Builder) {
	ctx = logging.WithActor(ctx, logging.Actor{
		Kind: logging.ActorBuilder, ID: strconv.Itoa(pollingBuilder.ID), Name: pollingBuilder.Identifier,
	})
	profile, err := s.graph.BuildProfile.Query().Where(buildprofile.Name(DefaultBuildProfileName)).Order(ent.Asc(buildprofile.FieldID)).First(ctx)
	if ent.IsNotFound(err) {
		return
	}
	if err != nil {
		serverLog.ErrorContext(ctx, "failed to load default build profile for automatic builds", "error", err)
		return
	}

	for _, targetOS := range pollingBuilder.SupportedTargets {
		for _, targetFormat := range SupportedFormats[targetOS] {
			s.queueMissingDefaultBuild(ctx, pollingBuilder.ID, profile.ID, targetOS, targetFormat)
		}
	}
}

func (s *Server) queueMissingDefaultBuild(ctx context.Context, builderID, profileID int, targetOS c2pb.Host_Platform, targetFormat builderpb.TargetFormat) {
	err := retryAutoBuildOnLock(ctx, func() error {
		return s.tryQueueMissingDefaultBuild(ctx, builderID, profileID, targetOS, targetFormat)
	})
	if err != nil {
		serverLog.ErrorContext(ctx, "failed to queue automatic default build", "builder_id", builderID, "target_os", targetOS, "target_format", targetFormat, "error", err)
	}
}

func (s *Server) tryQueueMissingDefaultBuild(ctx context.Context, builderID, profileID int, targetOS c2pb.Host_Platform, targetFormat builderpb.TargetFormat) error {
	exists, err := s.graph.BuildTask.Query().Where(
		buildtask.HasProfileWith(buildprofile.Name(DefaultBuildProfileName)),
		buildtask.TargetOsEQ(targetOS),
		buildtask.TargetFormatEQ(targetFormat),
	).Exist(ctx)
	if err != nil {
		return fmt.Errorf("check prior default build tasks: %w", err)
	}
	if exists {
		return nil
	}

	key := fmt.Sprintf("default:%d:%d", targetOS, targetFormat)
	_, err = CreateBuildTask(ctx, s.graph, CreateBuildTaskInput{
		ProfileID: profileID, TargetOS: targetOS, TargetFormat: &targetFormat, AutoBuildKey: &key,
	}, &builderID)
	if ent.IsConstraintError(err) {
		duplicate, checkErr := s.graph.BuildTask.Query().Where(buildtask.AutoBuildKeyEQ(key)).Exist(ctx)
		if checkErr != nil {
			return fmt.Errorf("verify automatic build key after constraint error: %w", checkErr)
		}
		if duplicate {
			return nil // Another poll committed the same pair first.
		}
	}
	return err
}

// Concurrent SQLite reads and writes can return SQLITE_BUSY or SQLITE_LOCKED.
// Retry the full existence check and creation attempt on either error.
func retryAutoBuildOnLock(ctx context.Context, create func() error) error {
	backoff := 10 * time.Millisecond
	var lastErr error
	for attempt := 0; attempt < 8; attempt++ {
		err := create()
		if err == nil {
			return nil
		}
		var sqliteErr sqlite3.Error
		if !errors.As(err, &sqliteErr) || (sqliteErr.Code != sqlite3.ErrBusy && sqliteErr.Code != sqlite3.ErrLocked) {
			return err
		}
		lastErr = err
		if attempt == 7 {
			return err
		}
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-time.After(backoff):
			backoff *= 2
			if backoff > 160*time.Millisecond {
				backoff = 160 * time.Millisecond
			}
		}
	}
	return lastErr
}
