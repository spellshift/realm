package c2

import (
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"strings"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/robfig/cron/v3"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/epb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/ent/host"
	"realm.pub/tavern/internal/ent/tag"
	"realm.pub/tavern/internal/ent/task"
	"realm.pub/tavern/internal/ent/tome"
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
	metricTomeAutomationErrors = prometheus.NewCounter(
		prometheus.CounterOpts{
			Name: "tavern_tome_automation_errors_total",
			Help: "The total number of errors encountered during tome automation",
		},
	)
)

func init() {
	prometheus.MustRegister(metricHostCallbacksTotal)
	prometheus.MustRegister(metricTomeAutomationErrors)
}

func (srv *Server) handleTomeAutomation(ctx context.Context, beaconID int, hostID int, isNewBeacon bool, isNewHost bool, now time.Time, interval time.Duration) {
	// Tome Automation Logic
	candidateTomes, err := srv.graph.Tome.Query().
		Where(tome.Or(
			tome.RunOnNewBeaconCallback(true),
			tome.RunOnFirstHostCallback(true),
			tome.RunOnScheduleNEQ(""),
		)).
		All(ctx)

	if err != nil {
		slog.ErrorContext(ctx, "failed to query candidate tomes for automation", "err", err)
		metricTomeAutomationErrors.Inc()
		return
	}

	selectedTomes := make(map[int]*ent.Tome)
	parser := cron.NewParser(cron.Minute | cron.Hour | cron.Dom | cron.Month | cron.Dow)
	cutoff := now.Add(interval)

	for _, t := range candidateTomes {
		shouldRun := false

		// Check RunOnNewBeaconCallback
		if isNewBeacon && t.RunOnNewBeaconCallback {
			shouldRun = true
		}

		// Check RunOnFirstHostCallback
		if !shouldRun && isNewHost && t.RunOnFirstHostCallback {
			shouldRun = true
		}

		// Check RunOnSchedule
		if !shouldRun && t.RunOnSchedule != "" {
			sched, err := parser.Parse(t.RunOnSchedule)
			if err == nil {
				isMatch := false
				// If schedule contains a range (hyphen), checking for strict current match
				// without factoring in callback interval.
				if strings.Contains(t.RunOnSchedule, "-") {
					currentMinute := now.Truncate(time.Minute)
					// Verify if 'currentMinute' is a valid trigger time.
					// We check if the next trigger after (currentMinute - 1s) is exactly currentMinute.
					next := sched.Next(currentMinute.Add(-1 * time.Second))
					if next.Equal(currentMinute) {
						isMatch = true
					}
				} else {
					// Check if any point between now and the next expected check-in matches the run schedule
					// Next(now-1sec) <= now + interval?
					next := sched.Next(now.Add(-1 * time.Second))
					if !next.After(cutoff) {
						isMatch = true
					}
				}

				if isMatch {
					// Check scheduled_hosts constraint
					hostCount, err := t.QueryScheduledHosts().Count(ctx)
					if err != nil {
						slog.ErrorContext(ctx, "failed to count scheduled hosts for automation", "err", err, "tome_id", t.ID)
						metricTomeAutomationErrors.Inc()
						continue
					}
					if hostCount == 0 {
						shouldRun = true
					} else {
						hostExists, err := t.QueryScheduledHosts().
							Where(host.ID(hostID)).
							Exist(ctx)
						if err != nil {
							slog.ErrorContext(ctx, "failed to check host existence for automation", "err", err, "tome_id", t.ID)
							metricTomeAutomationErrors.Inc()
							continue
						}
						if hostExists {
							shouldRun = true
						}
					}
				}
			} else {
				// Don't log cron parse errors for now, as it might be spammy if stored in DB
				// metricTomeAutomationErrors.Inc()
			}
		}

		if shouldRun {
			selectedTomes[t.ID] = t
		}
	}

	// Create Quest and Task for each selected Tome
	for _, t := range selectedTomes {
		q, err := srv.graph.Quest.Create().
			SetName(fmt.Sprintf("Automated: %s", t.Name)).
			SetTome(t).
			SetParamDefsAtCreation(t.ParamDefs).
			SetEldritchAtCreation(t.Eldritch).
			Save(ctx)
		if err != nil {
			slog.ErrorContext(ctx, "failed to create automated quest", "err", err, "tome_id", t.ID)
			metricTomeAutomationErrors.Inc()
			continue
		}

		_, err = srv.graph.Task.Create().
			SetQuest(q).
			SetBeaconID(beaconID).
			Save(ctx)
		if err != nil {
			slog.ErrorContext(ctx, "failed to create automated task", "err", err, "quest_id", q.ID)
			metricTomeAutomationErrors.Inc()
		}
	}
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

	// Check if host is new (before upsert)
	hostExists, err := srv.graph.Host.Query().
		Where(host.IdentifierEQ(req.Beacon.Host.Identifier)).
		Exist(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to query host existence: %v", err)
	}
	isNewHost := !hostExists

	// Upsert the host
	hostID, err := srv.graph.Host.Create().
		SetIdentifier(req.Beacon.Host.Identifier).
		SetName(req.Beacon.Host.Name).
		SetPlatform(req.Beacon.Host.Platform).
		SetPrimaryIP(req.Beacon.Host.PrimaryIp).
		SetExternalIP(clientIP).
		SetLastSeenAt(now).
		SetNextSeenAt(now.Add(time.Duration(activeTransport.Interval) * time.Second)).
		OnConflict().
		UpdateNewValues().
		ID(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to upsert host entity: %v", err)
	}

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
	isNewBeacon := !beaconExists

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
		SetNextSeenAt(now.Add(time.Duration(activeTransport.Interval) * time.Second)).
		SetInterval(activeTransport.Interval).
		SetTransport(activeTransport.Type).
		OnConflict().
		UpdateNewValues().
		ID(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to upsert beacon entity: %v", err)
	}

	// Run Tome Automation (non-blocking, best effort)
	srv.handleTomeAutomation(ctx, beaconID, hostID, isNewBeacon, isNewHost, now, time.Duration(req.GetBeacon().GetActiveTransport().GetInterval())*time.Second)

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
