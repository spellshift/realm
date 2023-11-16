package c2

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/kcarretto/realm/tavern/internal/c2/c2pb"
	"github.com/kcarretto/realm/tavern/internal/ent"
	"github.com/kcarretto/realm/tavern/internal/ent/beacon"
	"github.com/kcarretto/realm/tavern/internal/ent/host"
	"github.com/kcarretto/realm/tavern/internal/ent/task"
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
	if req.Beacon.Host.Platform == "" {
		return nil, fmt.Errorf("must provide host platform")
	}
	if req.Beacon.Agent == nil {
		return nil, fmt.Errorf("must provide beacon agent info")
	}
	if req.Beacon.Agent.Identifier == "" {
		return nil, fmt.Errorf("must provide agent identifier")
	}

	// 1. Upsert the host
	hostID, err := srv.graph.Host.Create().
		SetIdentifier(req.Beacon.Host.Identifier).
		SetName(req.Beacon.Host.Name).
		SetPlatform(host.Platform(req.Beacon.Host.Platform)).
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
			Id:         int32(claimedTask.ID),
			Eldritch:   claimedTome.Eldritch,
			Parameters: params,
		})
	}

	// 9. Return claimed tasks
	return &resp, nil
}

func (srv *Server) ReportTaskOutputs(ctx context.Context, req *c2pb.ReportTaskOutputsRequest) (*c2pb.ReportTaskOutputsResponse, error) {
	// 1. Prepare Transaction
	tx, err := srv.graph.Tx(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to initialize transaction: %w", err)
	}
	client := tx.Client()

	// 2. Rollback transaction if we panic
	defer func() {
		if v := recover(); v != nil {
			tx.Rollback()
			panic(v)
		}
	}()

	for _, output := range req.Outputs {
		var (
			execStartedAt  *time.Time
			execFinishedAt *time.Time
			taskErr        *string
		)
		if output.ExecStartedAt != nil {
			timestamp := output.ExecStartedAt.AsTime()
			execStartedAt = &timestamp
		}
		if output.ExecFinishedAt != nil {
			timestamp := output.ExecFinishedAt.AsTime()
			execFinishedAt = &timestamp
		}
		if output.Error != nil {
			taskErr = &output.Error.Msg
		}

		t, err := client.Task.Get(ctx, int(output.Id))
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to load task (id=%d): %w", output.Id, err))
		}
		if _, err := t.Update().
			SetOutput(fmt.Sprintf("%s%s", t.Output, output.Output)).
			SetExecStartedAt(*execStartedAt).
			SetNillableExecFinishedAt(execFinishedAt).
			SetNillableError(taskErr).
			Save(ctx); err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to update task (id=%d): %w", output.Id, err))
		}
	}

	// 3. Commit the transaction
	if err := tx.Commit(); err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to commit transaction: %w", err))
	}

	return &c2pb.ReportTaskOutputsResponse{}, nil
}
