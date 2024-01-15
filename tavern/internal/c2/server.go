package c2

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/ent/host"
	"realm.pub/tavern/internal/ent/task"
)

type Server struct {
	graph *ent.Client
	c2pb.UnimplementedC2Server
}

func New(graph *ent.Client) *Server {
	return &Server{
		graph: graph,
	}
}

func (srv *Server) ClaimTasks(ctx context.Context, req *c2pb.ClaimTasksRequest) (*c2pb.ClaimTasksResponse, error) {
	now := time.Now()

	// Validate input
	if req.Beacon == nil {
		return nil, fmt.Errorf("must provide beacon info")
	}
	if req.Beacon.Principal == "" {
		return nil, fmt.Errorf("must provide beacon principal")
	}
	if req.Beacon.Host == nil {
		return nil, fmt.Errorf("must provide beacon host info")
	}
	if req.Beacon.Host.Identifier == "" {
		return nil, fmt.Errorf("must provide host identifier")
	}
	if req.Beacon.Host.Name == "" {
		return nil, fmt.Errorf("must provide host name")
	}
	if req.Beacon.Agent == nil {
		return nil, fmt.Errorf("must provide beacon agent info")
	}
	if req.Beacon.Agent.Identifier == "" {
		return nil, fmt.Errorf("must provide agent identifier")
	}
	hostPlaform := convertHostPlatform(req.Beacon.Host.Platform)

	// 1. Upsert the host
	hostID, err := srv.graph.Host.Create().
		SetIdentifier(req.Beacon.Host.Identifier).
		SetName(req.Beacon.Host.Name).
		SetPlatform(hostPlaform).
		SetPrimaryIP(req.Beacon.Host.PrimaryIp).
		SetLastSeenAt(now).
		OnConflict().
		UpdateNewValues().
		ID(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to upsert host entity: %w", err)
	}

	// 2. Upsert the beacon
	beaconID, err := srv.graph.Beacon.Create().
		SetPrincipal(req.Beacon.Principal).
		SetIdentifier(req.Beacon.Identifier).
		SetAgentIdentifier(req.Beacon.Agent.Identifier).
		SetHostID(hostID).
		SetLastSeenAt(now).
		SetInterval(req.Beacon.Interval).
		OnConflict().
		UpdateNewValues().
		ID(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to upsert beacon entity: %w", err)
	}

	// 3. Load Tasks
	tasks, err := srv.graph.Task.Query().
		Where(task.And(
			task.HasBeaconWith(beacon.ID(beaconID)),
			task.ClaimedAtIsNil(),
		)).
		All(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to query tasks: %w", err)
	}

	// 4. Prepare Transaction for Claiming Tasks
	tx, err := srv.graph.Tx(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to initialize transaction: %w", err)
	}
	client := tx.Client()

	// 5. Rollback transaction if we panic
	defer func() {
		if v := recover(); v != nil {
			tx.Rollback()
			panic(v)
		}
	}()

	// 6. Update all ClaimedAt timestamps to claim tasks
	// ** Note: If one fails to update, we roll back the transaction and return the error
	taskIDs := make([]int, 0, len(tasks))
	for _, t := range tasks {
		_, err := client.Task.UpdateOne(t).
			SetClaimedAt(now).
			Save(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to update task %d: %w", t.ID, err))
		}
		taskIDs = append(taskIDs, t.ID)
	}

	// 7. Commit the transaction
	if err := tx.Commit(); err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to commit transaction: %w", err))
	}

	// 8. Load the tasks with our non transactional client (cannot use transaction after commit)
	resp := c2pb.ClaimTasksResponse{}
	resp.Tasks = make([]*c2pb.Task, 0, len(taskIDs))
	for _, taskID := range taskIDs {
		claimedTask, err := srv.graph.Task.Get(ctx, taskID)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to load claimed task (but it was still claimed) %d: %w", taskID, err))
		}
		claimedQuest, err := claimedTask.QueryQuest().Only(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to load tome information for claimed task (id=%d): %w", taskID, err))
		}
		claimedTome, err := claimedQuest.QueryTome().Only(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to load tome information for claimed task (id=%d): %w", taskID, err))
		}
		var params map[string]string
		if claimedQuest.Parameters != "" {
			if err := json.Unmarshal([]byte(claimedQuest.Parameters), &params); err != nil {
				return nil, rollback(tx, fmt.Errorf("failed to parse task parameters (id=%d,questID=%d): %w", taskID, claimedQuest.ID, err))
			}
		}
		resp.Tasks = append(resp.Tasks, &c2pb.Task{
			Id:         int64(claimedTask.ID),
			Eldritch:   claimedTome.Eldritch,
			Parameters: params,
		})
	}

	// 9. Return claimed tasks
	return &resp, nil
}

func (srv *Server) ReportTaskOutput(ctx context.Context, req *c2pb.ReportTaskOutputRequest) (*c2pb.ReportTaskOutputResponse, error) {
	// 1. Parse Input
	var (
		execStartedAt  *time.Time
		execFinishedAt *time.Time
		taskErr        *string
	)
	if req.Output.ExecStartedAt != nil {
		timestamp := req.Output.ExecStartedAt.AsTime()
		execStartedAt = &timestamp
	}
	if req.Output.ExecFinishedAt != nil {
		timestamp := req.Output.ExecFinishedAt.AsTime()
		execFinishedAt = &timestamp
	}
	if req.Output.Error != nil {
		taskErr = &req.Output.Error.Msg
	}

	// 2. Load the task
	t, err := srv.graph.Task.Get(ctx, int(req.Output.Id))
	if err != nil {
		return nil, fmt.Errorf("failed to submit task result (id=%d): %w", req.Output.Id, err)
	}

	// 3. Update task info
	_, err = t.Update().
		SetNillableExecStartedAt(execStartedAt).
		SetOutput(fmt.Sprintf("%s%s", t.Output, req.Output.Output)).
		SetNillableExecFinishedAt(execFinishedAt).
		SetNillableError(taskErr).
		Save(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to save submitted task result (id=%d): %w", t.ID, err)
	}

	return &c2pb.ReportTaskOutputResponse{}, nil
}

func convertHostPlatform(platform c2pb.Host_Platform) host.Platform {
	switch platform {
	case c2pb.Host_PLATFORM_WINDOWS:
		return host.PlatformWindows
	case c2pb.Host_PLATFORM_LINUX:
		return host.PlatformLinux
	case c2pb.Host_PLATFORM_MACOS:
		return host.PlatformMacOS
	case c2pb.Host_PLATFORM_BSD:
		return host.PlatformBSD
	}

	return host.PlatformUnknown
}
