package c2

import (
	"context"
	"encoding/json"
	"fmt"
	"strconv"
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
	"realm.pub/tavern/internal/ent/quest"
	"realm.pub/tavern/internal/ent/scheduledtask"
	"realm.pub/tavern/internal/ent/shell"
	"realm.pub/tavern/internal/ent/shelltask"
	"realm.pub/tavern/internal/ent/tag"
	"realm.pub/tavern/internal/ent/task"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/namegen"
	"realm.pub/tavern/internal/redirectors"
)

var (
	claimLog = logging.For("c2.claim")

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

// selectedAutomation is a scheduled task selected to run for a beacon, along with
// the canonical cron fire-time (minute-aligned) when selected by schedule.
// A zero fireTime means the task is non-cron (new beacon / first host, one-shot).
type selectedAutomation struct {
	task     *ent.ScheduledTask
	fireTime time.Time
}

func (srv *Server) handleTomeAutomation(ctx context.Context, beaconID int, hostID int, isNewBeacon bool, isNewHost bool, now time.Time, interval time.Duration) {
	// Tome Automation Logic
	candidateTasks, err := srv.graph.ScheduledTask.Query().
		Where(
			scheduledtask.Disabled(false),
			scheduledtask.Or(
				scheduledtask.RunOnNewBeaconCallback(true),
				scheduledtask.RunOnFirstHostCallback(true),
				scheduledtask.RunOnScheduleNEQ(""),
			),
			scheduledtask.Or(
				scheduledtask.Not(scheduledtask.HasScheduledHosts()),
				scheduledtask.HasScheduledHostsWith(host.ID(hostID)),
			),
		).
		WithTome().
		All(ctx)

	if err != nil {
		claimLog.ErrorContext(ctx, "failed to query candidate scheduled tasks for automation", "error", err)
		metricTomeAutomationErrors.Inc()
		return
	}

	selectedTasks := make(map[int]selectedAutomation)
	parser := cron.NewParser(cron.Minute | cron.Hour | cron.Dom | cron.Month | cron.Dow)
	cutoff := now.Add(interval)

	for _, t := range candidateTasks {
		shouldRun := false
		var fireTime time.Time

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
						fireTime = currentMinute
					}
				} else {
					// Check if any point between now and the next expected check-in matches the run schedule
					// Next(now-1sec) <= now + interval?
					next := sched.Next(now.Add(-1 * time.Second))
					if !next.After(cutoff) {
						isMatch = true
						fireTime = next
					}
				}

				if isMatch {
					shouldRun = true
				}
			} else {
				// Don't log cron parse errors for now, as it might be spammy if stored in DB
				// metricTomeAutomationErrors.Inc()
			}
		}

		if shouldRun && t.Edges.Tome != nil {
			selectedTasks[t.ID] = selectedAutomation{task: t, fireTime: fireTime}
		}
	}

	// Create Quest and Task for each selected ScheduledTask
	for _, sel := range selectedTasks {
		st := sel.task
		fireTime := sel.fireTime
		if !fireTime.IsZero() {
			// Normalize to UTC so equality matching is stable across zones/drivers.
			fireTime = fireTime.UTC()
		}
		tome := st.Edges.Tome

		var q *ent.Quest
		created := false
		if !fireTime.IsZero() {
			// Cron occurrence: atomically ensure the shared quest for this
			// (scheduled task, fire time) exists; no-op on conflict.
			key := fmt.Sprintf("%d-%d", st.ID, fireTime.Unix())
			err = srv.graph.Quest.Create().
				SetName(fmt.Sprintf("Automated: %s", st.Name)).
				SetTome(tome).
				SetScheduledTaskID(st.ID).
				SetParameters(st.Parameters).
				SetParamDefsAtCreation(tome.ParamDefs).
				SetEldritchAtCreation(tome.Eldritch).
				SetScheduledAt(fireTime).
				SetOccurrenceKey(key).
				OnConflictColumns(quest.FieldOccurrenceKey).
				Ignore().
				Exec(ctx)
			if err != nil {
				claimLog.ErrorContext(ctx, "failed to upsert automated quest", "error", err, "scheduled_task_id", st.ID, "occurrence_key", key)
				metricTomeAutomationErrors.Inc()
				continue
			}

			// Load the one canonical row by its deterministic key.
			q, err = srv.graph.Quest.Query().Where(quest.OccurrenceKeyEQ(key)).Only(ctx)
			if err != nil {
				claimLog.ErrorContext(ctx, "failed to load automated quest for occurrence", "error", err, "occurrence_key", key)
				metricTomeAutomationErrors.Inc()
				continue
			}
			// TODO(future): move quest.created audit to a Quest-create ent hook for exactly-once semantics.
			created = q.CreatedAt.After(now)
		} else {
			questCreate := srv.graph.Quest.Create().
				SetName(fmt.Sprintf("Automated: %s", st.Name)).
				SetTome(tome).
				SetScheduledTaskID(st.ID).
				SetParameters(st.Parameters).
				SetParamDefsAtCreation(tome.ParamDefs).
				SetEldritchAtCreation(tome.Eldritch)
			q, err = questCreate.Save(ctx)
			if err != nil {
				claimLog.ErrorContext(ctx, "failed to create automated quest", "error", err, "scheduled_task_id", st.ID, "tome_id", tome.ID)
				metricTomeAutomationErrors.Inc()
				continue
			}
			created = true
		}

		if !fireTime.IsZero() {
			// A beacon runs a given cron occurrence at most once, regardless of
			// how many times it checks in within the look-ahead/range window.
			exists, err := q.QueryTasks().
				Where(task.HasBeaconWith(beacon.ID(beaconID))).
				Exist(ctx)
			if err != nil {
				claimLog.ErrorContext(ctx, "failed to check for existing automated task", "error", err, "quest_id", q.ID, "beacon_id", beaconID)
				metricTomeAutomationErrors.Inc()
				continue
			}
			if exists {
				continue
			}
		}

		_, err = srv.graph.Task.Create().
			SetQuest(q).
			SetBeaconID(beaconID).
			Save(ctx)
		if err != nil {
			claimLog.ErrorContext(ctx, "failed to create automated task", "error", err, "quest_id", q.ID)
			metricTomeAutomationErrors.Inc()
			continue
		}

		if !created {
			continue
		}

		schedCtx := logging.WithActor(ctx, logging.Actor{
			Kind:       logging.ActorScheduler,
			ID:         strconv.Itoa(st.ID),
			Name:       st.Name,
			AuthMethod: "scheduled_task",
		})
		questAuditAttrs := make([]any, 0, 12)
		questAuditAttrs = append(questAuditAttrs, "scheduled_task_id", st.ID)
		questAuditAttrs = append(questAuditAttrs, logging.QuestAttrs(q.ID, q.Name, st.Parameters)...)
		questAuditAttrs = append(questAuditAttrs, logging.TomeAttrs(tome.ID, tome.Name, string(tome.Tactic))...)
		questAuditAttrs = append(questAuditAttrs, "beacon_id", beaconID, "host_id", hostID)
		logging.Audit(schedCtx, "quest", "quest.created", "scheduled task created quest", questAuditAttrs...)
	}
}

func (srv *Server) ClaimTasks(ctx context.Context, req *c2pb.ClaimTasksRequest) (*c2pb.ClaimTasksResponse, error) {
	now := time.Now()
	ctx, clientIP := enrichC2Context(ctx)

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

	hostCreate := srv.graph.Host.Create().
		SetIdentifier(req.Beacon.Host.Identifier).
		SetName(req.Beacon.Host.Name).
		SetPlatform(req.Beacon.Host.Platform).
		SetLastSeenAt(now).
		SetNextSeenAt(now.Add(time.Duration(activeTransport.Interval) * time.Second))

	// Only update primary IP if it's not empty
	if req.Beacon.Host.PrimaryIp != "" {
		hostCreate.SetPrimaryIP(req.Beacon.Host.PrimaryIp)
	}
	if req.Beacon.Host.Subnet != "" {
		hostCreate.SetSubnet(req.Beacon.Host.Subnet)
	}

	// Only update external IP if it's not NOOP
	if clientIP != redirectors.ExternalIPNoop {
		hostCreate.SetExternalIP(clientIP)
	}

	// Upsert the host
	hostID, err := hostCreate.
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
			claimLog.ErrorContext(ctx, "metrics failed to query host tags", "error", err, "host_id", hostID)
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
	existingBeacon, err := srv.graph.Beacon.Query().
		Where(beacon.IdentifierEQ(req.Beacon.Identifier)).
		Only(ctx)
	if err != nil && !ent.IsNotFound(err) {
		return nil, status.Errorf(codes.Internal, "failed to query beacon entity: %v", err)
	}
	isNewBeacon := existingBeacon == nil

	var beaconNameAddr *string = nil
	if isNewBeacon {
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
		for _, candidate := range candidateNames {
			if !namegen.IsCollision(collisions, candidate) {
				beaconNameAddr = &candidate
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

	var beaconName string
	if beaconNameAddr != nil {
		beaconName = *beaconNameAddr
	} else if existingBeacon != nil {
		beaconName = existingBeacon.Name
	}

	ctx = logging.WithActor(ctx, logging.Actor{
		Kind:       logging.ActorBeacon,
		ID:         strconv.Itoa(beaconID),
		Name:       beaconName,
		AuthMethod: "c2",
	})

	if isNewHost {
		externalIP := ""
		if clientIP != redirectors.ExternalIPNoop && clientIP != "unknown" {
			externalIP = clientIP
		}
		hostAuditAttrs := make([]any, 0, 10)
		hostAuditAttrs = append(hostAuditAttrs, logging.HostAttrs(hostID, req.Beacon.Host.Name, req.Beacon.Host.Identifier)...)
		hostAuditAttrs = append(hostAuditAttrs,
			"platform", req.Beacon.Host.Platform.String(),
			"primary_ip", req.Beacon.Host.PrimaryIp,
			"external_ip", externalIP,
		)
		logging.Audit(ctx, "c2", "c2.host_registered", "new target host registered", hostAuditAttrs...)
	}

	if isNewBeacon {
		beaconAuditAttrs := make([]any, 0, 12)
		beaconAuditAttrs = append(beaconAuditAttrs, logging.BeaconAttrs(beaconID, beaconName, req.Beacon.Identifier)...)
		beaconAuditAttrs = append(beaconAuditAttrs, logging.HostAttrs(hostID, req.Beacon.Host.Name, req.Beacon.Host.Identifier)...)
		beaconAuditAttrs = append(beaconAuditAttrs,
			"principal", req.Beacon.Principal,
			"transport", activeTransport.Type.String(),
		)
		logging.Audit(ctx, "c2", "c2.beacon_registered", "new beacon registered", beaconAuditAttrs...)
	}

	// Create BeaconHistory record
	var latency int64
	if !isNewBeacon && !existingBeacon.NextSeenAt.IsZero() {
		latency = now.Sub(existingBeacon.NextSeenAt).Milliseconds()
	}

	_, err = srv.graph.BeaconHistory.Create().
		SetBeaconID(beaconID).
		SetLatency(latency).
		Save(ctx)
	if err != nil {
		claimLog.ErrorContext(ctx, "failed to create beacon history record", "error", err, "beacon_id", beaconID)
	}

	// Run Tome Automation (non-blocking, best effort)
	idx := req.GetBeacon().GetAvailableTransports().GetActiveIndex()
	srv.handleTomeAutomation(ctx, beaconID, hostID, isNewBeacon, isNewHost, now, time.Duration(req.GetBeacon().GetAvailableTransports().GetTransports()[idx].GetInterval())*time.Second)

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

	// Load Shell Tasks
	shellTasks, err := srv.graph.ShellTask.Query().
		Where(shelltask.And(
			shelltask.HasShellWith(shell.HasBeaconWith(beacon.ID(beaconID))),
			shelltask.ClaimedAtIsNil(),
		)).
		All(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to query shell tasks: %v", err)
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

	// Update all ClaimedAt timestamps to claim shell tasks
	shellTaskIDs := make([]int, 0, len(shellTasks))
	for _, t := range shellTasks {
		_, err := client.ShellTask.UpdateOne(t).
			SetClaimedAt(now).
			Save(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to update shell task %d: %w", t.ID, err))
		}
		shellTaskIDs = append(shellTaskIDs, t.ID)
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

		// Generate JWT with beacon ID
		jwtToken, err := srv.generateTaskJWT(beaconID)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to generate JWT for task (id=%d): %w", taskID, err))
		}

		resp.Tasks = append(resp.Tasks, &c2pb.Task{
			Id:        int64(claimedTask.ID),
			QuestName: claimedQuest.Name,
			Tome: &epb.Tome{
				Eldritch:   claimedTome.Eldritch,
				Parameters: params,
				FileNames:  claimedAssetNames,
			},
			Jwt: jwtToken,
		})
	}

	resp.ShellTasks = make([]*c2pb.ShellTask, 0, len(shellTaskIDs))
	for _, shellTaskID := range shellTaskIDs {
		claimedShellTask, err := srv.graph.ShellTask.Get(ctx, shellTaskID)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to load claimed shell task (but it was still claimed) %d: %w", shellTaskID, err))
		}
		shellID, err := claimedShellTask.QueryShell().OnlyID(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to load shell for claimed shell task (id=%d): %w", shellTaskID, err))
		}

		// Generate JWT for ShellTask
		shellJwtToken, err := srv.generateTaskJWT(beaconID)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to generate JWT for shell task (id=%d): %w", shellTaskID, err))
		}

		resp.ShellTasks = append(resp.ShellTasks, &c2pb.ShellTask{
			Id:         int64(claimedShellTask.ID),
			Input:      claimedShellTask.Input,
			ShellId:    int64(shellID),
			SequenceId: claimedShellTask.SequenceID,
			StreamId:   claimedShellTask.StreamID,
			Jwt:        shellJwtToken,
		})
	}

	if len(taskIDs) > 0 || len(shellTaskIDs) > 0 {
		logging.Audit(ctx, "c2", "c2.tasks_claimed", "beacon claimed tasks",
			"beacon_id", beaconID,
			"host_id", hostID,
			"task_ids", taskIDs,
			"shell_task_ids", shellTaskIDs,
		)
	} else {
		claimLog.DebugContext(ctx, "beacon polled with no pending tasks",
			"beacon_id", beaconID,
			"host_id", hostID,
		)
	}

	// Return claimed tasks
	return &resp, nil
}
