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
	"google.golang.org/grpc"
)

type Server struct {
	graph *ent.Client
	c2pb.UnimplementedC2Server
}

func (srv *Server) ClaimTasks(ctx context.Context, input *c2pb.ClaimTasksRequest, opts ...grpc.CallOption) (*c2pb.ClaimTasksResponse, error) {
	now := time.Now()

	if input.Beacon == nil {
		return nil, fmt.Errorf("must provide beacon info")
	}
	if input.Beacon.Host == nil {
		return nil, fmt.Errorf("must provide beacon host info")
	}
	if input.Beacon.Agent == nil {
		return nil, fmt.Errorf("must provide beacon agent info")
	}

	// 1. Upsert the host
	hostID, err := srv.graph.Host.Create().
		SetIdentifier(input.Beacon.Host.Identifier).
		SetName(input.Beacon.Host.Name).
		SetPlatform(host.Platform(input.Beacon.Host.Platform)).
		SetPrimaryIP(input.Beacon.Host.PrimaryIp).
		SetLastSeenAt(now).
		OnConflict().
		UpdateNewValues().
		ID(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to upsert host entity: %w", err)
	}

	// 2. Upsert the beacon
	beaconID, err := srv.graph.Beacon.Create().
		SetPrincipal(input.Beacon.Principal).
		SetIdentifier(input.Beacon.Identifier).
		SetAgentIdentifier(input.Beacon.Agent.Identifier).
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
			return nil, fmt.Errorf("failed to load claimed task (but it was still claimed) %d: %w", taskID, err)
		}
		claimedQuest, err := claimedTask.QueryQuest().Only(ctx)
		if err != nil {
			return nil, fmt.Errorf("failed to load tome information for claimed task (id=%d): %w", taskID, err)
		}
		claimedTome, err := claimedQuest.QueryTome().Only(ctx)
		if err != nil {
			return nil, fmt.Errorf("failed to load tome information for claimed task (id=%d): %w", taskID, err)
		}
		var params map[string]string
		if claimedQuest.Parameters != "" {
			if err := json.Unmarshal([]byte(claimedQuest.Parameters), &params); err != nil {
				return nil, fmt.Errorf("failed to parse task parameters (id=%d,questID=%d): %w", taskID, claimedQuest.ID, err)
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
func (srv *Server) ReportTaskOutputs(ctx context.Context, input *c2pb.ReportTaskOutputsRequest, opts ...grpc.CallOption) (*c2pb.ReportTaskOutputsResponse, error) {
	tx, err := srv.graph.Tx(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to initialize transaction: %w", err)
	}
	client := tx.Client()

	for _, output := range input.Outputs {
		t, err := srv.graph.Task.Get(ctx, int(output.Id))
		if err != nil {
			return nil, fmt.Errorf("failed to submit task result (id=%d): %w", output.Id, err)
		}
		srv.graph.Task.UpdateOneID(int(output.Id)).
			SetExecStartedAt(output.ExecStartedAt.AsTime())
			SetOutput(fmt.Sprintf("%s%s", t.Output, input.Output))
	}


	// 1. Load the task
	t, err := srv.graph.Task.Get(ctx, input.)
	if err != nil {
		return nil, fmt.Errorf("failed to submit task result: %w", err)
	}

	t, err = t.Update().
		SetExecStartedAt(input.ExecStartedAt).
		SetOutput(fmt.Sprintf("%s%s", t.Output, input.Output)).
		SetNillableExecFinishedAt(input.ExecFinishedAt).
		SetNillableError(input.Error).
		Save(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to save submitted task result: %w", err)
	}

	return t, nil
}
