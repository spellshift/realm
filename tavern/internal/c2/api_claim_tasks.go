package c2

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/epb"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/ent/task"
)

func (srv *Server) ClaimTasks(ctx context.Context, req *c2pb.ClaimTasksRequest) (*c2pb.ClaimTasksResponse, error) {
	now := time.Now()
	clientIP := GetClientIP(ctx)

	// Validate input
	if req.Beacon == nil {
		return nil, status.Errorf(codes.InvalidArgument, "must provide beacon info")
	}
	if req.Beacon.Principal == "" {
		return nil, status.Errorf(codes.InvalidArgument, "must provide beacon principal")
	}
	if req.Beacon.Host == nil {
		return nil, status.Errorf(codes.InvalidArgument, "must provide beacon host info")
	}
	if req.Beacon.Host.Identifier == "" {
		return nil, status.Errorf(codes.InvalidArgument, "must provide host identifier")
	}
	if req.Beacon.Host.Name == "" {
		return nil, status.Errorf(codes.InvalidArgument, "must provide host name")
	}
	if req.Beacon.Agent == nil {
		return nil, status.Errorf(codes.InvalidArgument, "must provide beacon agent info")
	}
	if req.Beacon.Agent.Identifier == "" {
		return nil, status.Errorf(codes.InvalidArgument, "must provide agent identifier")
	}
	if req.Beacon.AvailableTransports == nil {
		return nil, status.Errorf(codes.InvalidArgument, "must provide available transports")
	}
	if len(req.Beacon.AvailableTransports.Transports) == 0 {
		return nil, status.Errorf(codes.InvalidArgument, "must provide at least one transport")
	}
	if req.Beacon.AvailableTransports.ActiveIndex >= uint32(len(req.Beacon.AvailableTransports.Transports)) {
		return nil, status.Errorf(codes.InvalidArgument, "active_index out of bounds")
	}

	// Get the active transport
	activeTransport := req.Beacon.AvailableTransports.Transports[req.Beacon.AvailableTransports.ActiveIndex]
	interval := time.Duration(activeTransport.Interval) * time.Second

	// Register Host
	hostID, isNewHost, err := srv.registration.RegisterHost(ctx, req, clientIP, now, interval)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "%v", err)
	}

	// Register Beacon
	beaconID, isNewBeacon, err := srv.registration.RegisterBeacon(ctx, req, hostID, now, interval, int32(activeTransport.Type))
	if err != nil {
		return nil, status.Errorf(codes.Internal, "%v", err)
	}

	// Run Tome Automation (non-blocking, best effort)
	idx := req.GetBeacon().GetAvailableTransports().GetActiveIndex()
	srv.automation.RunTomeAutomation(ctx, beaconID, hostID, isNewBeacon, isNewHost, now, time.Duration(req.GetBeacon().GetAvailableTransports().GetTransports()[idx].GetInterval())*time.Second)

	// Load Tasks
	tasks, err := srv.graph.Task.Query().
		Where(task.And(
			task.HasBeaconWith(beacon.ID(beaconID)),
			task.ClaimedAtIsNil(),
		)).
		All(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to query tasks: %v", err)
	}

	// Prepare Transaction for Claiming Tasks
	tx, err := srv.graph.Tx(ctx)
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

	// Commit the transaction
	if err := tx.Commit(); err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to commit transaction: %w", err))
	}

	// Load the tasks with our non transactional client (cannot use transaction after commit)
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
		claimedAssets, err := claimedTome.QueryAssets().All(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to load tome assets (id=%d,tomeID=%d)", taskID, claimedTome.ID))
		}
		claimedAssetNames := make([]string, 0, len(claimedAssets))
		for _, a := range claimedAssets {
			claimedAssetNames = append(claimedAssetNames, a.Name)
		}
		resp.Tasks = append(resp.Tasks, &c2pb.Task{
			Id:        int64(claimedTask.ID),
			QuestName: claimedQuest.Name,
			Tome: &epb.Tome{
				Eldritch:   claimedTome.Eldritch,
				Parameters: params,
				FileNames:  claimedAssetNames,
			},
		})
	}

	// Return claimed tasks
	return &resp, nil
}
