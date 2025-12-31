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
	"realm.pub/tavern/internal/c2/services/registration"
	"realm.pub/tavern/internal/c2/services/transaction"
	"realm.pub/tavern/internal/c2/validation"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/ent/task"
)

func (srv *Server) ClaimTasks(ctx context.Context, req *c2pb.ClaimTasksRequest) (*c2pb.ClaimTasksResponse, error) {
	now := time.Now()
	clientIP := GetClientIP(ctx)

	// Validate input
	if err := validation.ValidateBeaconRequest(req); err != nil {
		return nil, err
	}

	// Upsert Host and Beacon (Registration)
	regSvc := registration.New(srv.graph)
	beaconID, _, err := regSvc.UpsertBeacon(ctx, req, clientIP)
	if err != nil {
		// Note: The service might return a wrapped error, we might want to check for specific errors or return internal.
		return nil, status.Errorf(codes.Internal, "registration failed: %v", err)
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

	var taskIDs []int

	// Perform Task Claiming in Transaction
	if err := transaction.Run(ctx, srv.graph, func(tx *ent.Tx) error {
		client := tx.Client()
		taskIDs = make([]int, 0, len(tasks))
		for _, t := range tasks {
			_, err := client.Task.UpdateOne(t).
				SetClaimedAt(now).
				Save(ctx)
			if err != nil {
				return fmt.Errorf("failed to update task %d: %w", t.ID, err)
			}
			taskIDs = append(taskIDs, t.ID)
		}
		return nil
	}); err != nil {
		return nil, err
	}

	// Load the tasks with our non transactional client (cannot use transaction after commit)
	resp := c2pb.ClaimTasksResponse{}
	resp.Tasks = make([]*c2pb.Task, 0, len(taskIDs))
	for _, taskID := range taskIDs {
		claimedTask, err := srv.graph.Task.Get(ctx, taskID)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to load claimed task (but it was still claimed) %d: %v", taskID, err)
		}
		claimedQuest, err := claimedTask.QueryQuest().Only(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to load tome information for claimed task (id=%d): %v", taskID, err)
		}
		claimedTome, err := claimedQuest.QueryTome().Only(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to load tome information for claimed task (id=%d): %v", taskID, err)
		}
		var params map[string]string
		if claimedQuest.Parameters != "" {
			if err := json.Unmarshal([]byte(claimedQuest.Parameters), &params); err != nil {
				return nil, status.Errorf(codes.Internal, "failed to parse task parameters (id=%d,questID=%d): %v", taskID, claimedQuest.ID, err)
			}
		}
		claimedFiles, err := claimedTome.QueryFiles().All(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to load tome files (id=%d,tomeID=%d)", taskID, claimedTome.ID)
		}
		claimedFileNames := make([]string, 0, len(claimedFiles))
		for _, f := range claimedFiles {
			claimedFileNames = append(claimedFileNames, f.Name)
		}
		resp.Tasks = append(resp.Tasks, &c2pb.Task{
			Id:        int64(claimedTask.ID),
			QuestName: claimedQuest.Name,
			Tome: &epb.Tome{
				Eldritch:   claimedTome.Eldritch,
				Parameters: params,
				FileNames:  claimedFileNames,
			},
		})
	}

	// Return claimed tasks
	return &resp, nil
}
