package c2

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/ent/task"
	"realm.pub/tavern/internal/namegen"
)

func (srv *Server) ClaimTasks(ctx context.Context, req *c2pb.ClaimTasksRequest) (*c2pb.ClaimTasksResponse, error) {
	now := time.Now()

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
	hostPlaform := convertHostPlatform(req.Beacon.Host.Platform)

	// Upsert the host
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
		return nil, status.Errorf(codes.Internal, "failed to upsert host entity: %v", err)
	}
	// Generate name for new beacons
	beaconExists, err := srv.graph.Beacon.Query().
		Where(beacon.IdentifierEQ(req.Beacon.Identifier)).
		Exist(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to query beacon entity: %v", err)
	}
	var beaconNameAddr *string = nil
	if !beaconExists {
		candidateNames := []string{
			namegen.NewSimple(),
			namegen.New(),
			namegen.NewComplex(),
		}

		collisions, err := srv.graph.Beacon.Query().
			Where(beacon.NameIn(candidateNames...)).
			All(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to query beacon entity: %v", err)
		}
		if len(collisions) == 3 {
			candidateNames := []string{
				namegen.NewSimple(),
				namegen.New(),
				namegen.NewComplex(),
			}

			collisions, err = srv.graph.Beacon.Query().
				Where(beacon.NameIn(candidateNames...)).
				All(ctx)
			if err != nil {
				return nil, status.Errorf(codes.Internal, "failed to query beacon entity: %v", err)
			}
		}
		for _, canidate := range candidateNames {
			if !namegen.IsCollision(collisions, canidate) {
				beaconNameAddr = &canidate
				break
			}
		}
	}

	// Upsert the beacon
	beaconID, err := srv.graph.Beacon.Create().
		SetPrincipal(req.Beacon.Principal).
		SetIdentifier(req.Beacon.Identifier).
		SetAgentIdentifier(req.Beacon.Agent.Identifier).
		SetNillableName(beaconNameAddr).
		SetHostID(hostID).
		SetLastSeenAt(now).
		SetInterval(req.Beacon.Interval).
		OnConflict().
		UpdateNewValues().
		ID(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to upsert beacon entity: %v", err)
	}

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
		claimedFiles, err := claimedTome.QueryFiles().All(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to load tome files (id=%d,tomeID=%d)", taskID, claimedTome.ID))
		}
		claimedFileNames := make([]string, 0, len(claimedFiles))
		for _, f := range claimedFiles {
			claimedFileNames = append(claimedFileNames, f.Name)
		}
		resp.Tasks = append(resp.Tasks, &c2pb.Task{
			Id:         int64(claimedTask.ID),
			Eldritch:   claimedTome.Eldritch,
			Parameters: params,
			FileNames:  claimedFileNames,
		})
	}

	// Return claimed tasks
	return &resp, nil
}
