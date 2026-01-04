package c2

import (
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"strings"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/epb"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/ent/host"
	"realm.pub/tavern/internal/ent/tag"
	"realm.pub/tavern/internal/ent/task"
	"realm.pub/tavern/internal/namegen"
)

var (
	metricHostCallbacksTotal = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "tavern_host_callbacks_total",
			Help: "The total number of ClaimTasks gRPC calls, provided with host labeling",
		},
		[]string{"host_identifier", "host_groups", "host_services"},
	)
)

func init() {
	prometheus.MustRegister(metricHostCallbacksTotal)
}

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

	// Check if host exists before upserting
	hostExists, err := srv.graph.Host.Query().
		Where(host.IdentifierEQ(req.Beacon.Host.Identifier)).
		Exist(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to query host entity: %v", err)
	}

	// Upsert the host
	hostID, err := srv.graph.Host.Create().
		SetIdentifier(req.Beacon.Host.Identifier).
		SetName(req.Beacon.Host.Name).
		SetPlatform(req.Beacon.Host.Platform).
		SetPrimaryIP(req.Beacon.Host.PrimaryIp).
		SetExternalIP(clientIP).
		SetLastSeenAt(now).
		SetNextSeenAt(now.Add(time.Duration(req.Beacon.ActiveTransport.Interval) * time.Second)).
		OnConflict().
		UpdateNewValues().
		ID(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to upsert host entity: %v", err)
	}

	isNewHost := !hostExists
	isNewBeacon := false

	// Metrics
	defer func() {
		var hostGroupTags []string
		var hostServiceTags []string
		var tagNames []struct {
			Name string
			Kind string
		}
		err := srv.graph.Host.Query().
			Where(host.ID(hostID)).
			QueryTags().
			Order(tag.ByKind()).
			Select(tag.FieldName, tag.FieldKind).
			Scan(ctx, &tagNames)
		if err != nil {
			slog.ErrorContext(ctx, "metrics failed to query host tags", "err", err, "host_id", hostID)
		}
		for _, t := range tagNames {
			if t.Kind == string(tag.KindGroup) {
				hostGroupTags = append(hostGroupTags, t.Name)
			}
			if t.Kind == string(tag.KindService) {
				hostServiceTags = append(hostServiceTags, t.Name)
			}
		}
		metricHostCallbacksTotal.
			WithLabelValues(
				req.Beacon.Host.Identifier,
				strings.Join(hostGroupTags, ","),
				strings.Join(hostServiceTags, ","),
			).
			Inc()
	}()

	// Generate name for new beacons
	beaconExists, err := srv.graph.Beacon.Query().
		Where(beacon.IdentifierEQ(req.Beacon.Identifier)).
		Exist(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to query beacon entity: %v", err)
	}
	isNewBeacon = !beaconExists
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
		SetNextSeenAt(now.Add(time.Duration(req.Beacon.ActiveTransport.Interval) * time.Second)).
		SetInterval(req.Beacon.ActiveTransport.Interval).
		SetTransport(req.Beacon.ActiveTransport.Type).
		OnConflict().
		UpdateNewValues().
		ID(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to upsert beacon entity: %v", err)
	}

	// Queue scheduled tasks for new hosts/beacons
	if isNewHost || isNewBeacon {
		// Query for tasks with schedules that match our condition
		scheduledTasks, err := srv.graph.Task.Query().
			Where(task.ScheduleNotNil()).
			WithQuest().
			All(ctx)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to query scheduled tasks: %v", err)
		}

		// Create task instances for matching scheduled tasks
		for _, scheduledTask := range scheduledTasks {
			if scheduledTask.Schedule != nil {
				shouldQueue := false
				if isNewHost && scheduledTask.Schedule.NewHost {
					shouldQueue = true
				}
				if isNewBeacon && scheduledTask.Schedule.NewBeacon {
					shouldQueue = true
				}

				if shouldQueue {
					// Create a new task instance based on the scheduled task
					_, err := srv.graph.Task.Create().
						SetQuestID(scheduledTask.Edges.Quest.ID).
						SetBeaconID(beaconID).
						Save(ctx)
					if err != nil {
						slog.ErrorContext(ctx, "failed to create scheduled task instance", "err", err, "scheduled_task_id", scheduledTask.ID, "beacon_id", beaconID)
						// Continue with other tasks even if one fails
					}
				}
			}
		}
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
