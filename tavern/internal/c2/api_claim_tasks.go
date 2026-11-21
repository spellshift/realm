package c2

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"strconv"
	"strings"
	"time"

	"github.com/go-sql-driver/mysql"
	"github.com/mattn/go-sqlite3"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/robfig/cron/v3"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/epb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
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
	"realm.pub/tavern/internal/tomebundle"
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

	parser := cron.NewParser(cron.Minute | cron.Hour | cron.Dom | cron.Month | cron.Dow)
	cutoff := now.Add(interval)

	for _, st := range candidateTasks {
		if st.Edges.Tome == nil {
			continue
		}
		if (isNewBeacon && st.RunOnNewBeaconCallback) || (isNewHost && st.RunOnFirstHostCallback) {
			srv.createAutomatedTask(ctx, st.ID, beaconID, hostID, nil)
		}
		if st.RunOnSchedule == "" {
			continue
		}
		sched, err := parser.Parse(st.RunOnSchedule)
		if err != nil {
			continue
		}
		var occurrence time.Time
		if strings.Contains(st.RunOnSchedule, "-") {
			minute := now.Truncate(time.Minute)
			if sched.Next(minute.Add(-time.Second)).Equal(minute) {
				occurrence = minute
			}
		} else {
			occurrence = sched.Next(now.Add(-time.Second))
			if occurrence.After(cutoff) {
				occurrence = time.Time{}
			}
		}
		if !occurrence.IsZero() {
			srv.createAutomatedTask(ctx, st.ID, beaconID, hostID, &occurrence)
		}
	}
}

// createAutomatedTask retries concurrent unique-key conflicts from a fresh transaction.
func (srv *Server) createAutomatedTask(ctx context.Context, scheduledTaskID, beaconID, hostID int, occurrence *time.Time) {
	var lastErr error
	for attempt := 0; attempt < 8; attempt++ {
		if ctx.Err() != nil {
			return
		}
		err := srv.createAutomatedTaskOnce(ctx, scheduledTaskID, beaconID, hostID, occurrence)
		if err == nil {
			return
		}
		if isRetryableAutomatedTaskError(err) {
			lastErr = err
			time.Sleep(time.Duration(attempt+1) * 10 * time.Millisecond)
			continue
		}
		claimLog.ErrorContext(ctx, "failed to create automated task", "error", err, "scheduled_task_id", scheduledTaskID, "beacon_id", beaconID)
		metricTomeAutomationErrors.Inc()
		return
	}
	claimLog.ErrorContext(ctx, "failed to create automated task after retries", "error", lastErr, "scheduled_task_id", scheduledTaskID, "beacon_id", beaconID)
	metricTomeAutomationErrors.Inc()
}

func isRetryableAutomatedTaskError(err error) bool {
	var sqliteErr sqlite3.Error
	if errors.As(err, &sqliteErr) {
		if sqliteErr.Code == sqlite3.ErrBusy || sqliteErr.Code == sqlite3.ErrLocked {
			return true
		}
		return sqliteErr.ExtendedCode == sqlite3.ErrConstraintUnique && isScheduledKeyError(err.Error())
	}

	var mysqlErr *mysql.MySQLError
	if errors.As(err, &mysqlErr) {
		return mysqlErr.Number == 1062 && isScheduledKeyError(mysqlErr.Message)
	}
	return false
}

func isScheduledKeyError(message string) bool {
	return strings.Contains(message, quest.FieldScheduledOccurrenceKey) ||
		strings.Contains(message, task.FieldScheduledDedupKey)
}

func (srv *Server) createAutomatedTaskOnce(ctx context.Context, scheduledTaskID, beaconID, hostID int, occurrence *time.Time) error {
	tx, err := srv.graph.Tx(ctx)
	if err != nil {
		return err
	}
	defer tx.Rollback()
	client := tx.Client()
	st, err := client.ScheduledTask.Query().Where(
		scheduledtask.ID(scheduledTaskID),
		scheduledtask.Disabled(false),
		scheduledtask.Or(
			scheduledtask.Not(scheduledtask.HasScheduledHosts()),
			scheduledtask.HasScheduledHostsWith(host.ID(hostID)),
		),
	).WithTome().Only(ctx)
	if ent.IsNotFound(err) {
		return nil
	}
	if err != nil {
		return err
	}
	if st.Edges.Tome == nil {
		return nil
	}

	var q *ent.Quest
	createdQuest := false
	var key string
	if occurrence != nil {
		key = fmt.Sprintf("%d:%d", st.ID, occurrence.UTC().Unix())
		q, err = client.Quest.Query().Where(quest.ScheduledOccurrenceKey(key)).Only(ctx)
		if err != nil && !ent.IsNotFound(err) {
			return err
		}
	}
	if q == nil {
		tome := st.Edges.Tome
		assets, err := tome.QueryAssets().Order(ent.Asc(asset.FieldID)).All(ctx)
		if err != nil {
			return err
		}
		create := client.Quest.Create().SetName(fmt.Sprintf("Automated: %s", st.Name)).
			SetTome(tome).SetScheduledTask(st).SetParameters(st.Parameters).
			SetParamDefsAtCreation(tome.ParamDefs).SetEldritchAtCreation(tome.Eldritch).
			SetHasScheduledSnapshot(true)
		bundle, err := tomebundle.Create(ctx, client, assets)
		if err != nil {
			return err
		}
		create.SetBundle(bundle)
		if occurrence != nil {
			create.SetScheduledOccurrenceAt(*occurrence).SetScheduledOccurrenceKey(key)
		}
		q, err = create.Save(ctx)
		if err != nil {
			return err
		}
		createdQuest = true
	}

	createTask := client.Task.Create().SetQuest(q).SetBeaconID(beaconID)
	if occurrence != nil {
		taskKey := fmt.Sprintf("%d:%d", q.ID, beaconID)
		exists, err := client.Task.Query().Where(task.ScheduledDedupKey(taskKey)).Exist(ctx)
		if err != nil {
			return err
		}
		if exists {
			return nil
		}
		createTask.SetScheduledDedupKey(taskKey)
	}
	t, err := createTask.Save(ctx)
	if err != nil {
		return err
	}
	if err := tx.Commit(); err != nil {
		return err
	}

	schedCtx := logging.WithActor(ctx, logging.Actor{
		Kind: logging.ActorScheduler, ID: strconv.Itoa(st.ID), Name: st.Name, AuthMethod: "scheduled_task",
	})
	if createdQuest {
		attrs := []any{"scheduled_task_id", st.ID}
		attrs = append(attrs, logging.QuestAttrs(q.ID, q.Name, q.Parameters)...)
		attrs = append(attrs, logging.TomeAttrs(st.Edges.Tome.ID, st.Edges.Tome.Name, string(st.Edges.Tome.Tactic))...)
		attrs = append(attrs, "beacon_id", beaconID, "host_id", hostID)
		logging.Audit(schedCtx, "quest", "quest.created", "scheduled task created quest", attrs...)
	}
	logging.Audit(schedCtx, "task", "task.created", "scheduled task created task",
		"task_id", t.ID, "quest_id", q.ID, "beacon_id", beaconID, "host_id", hostID, "scheduled_task_id", st.ID)
	return nil
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
		updated, err := client.Task.Update().
			Where(task.ID(t.ID), task.ClaimedAtIsNil()).
			SetClaimedAt(now).
			Save(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to update task %d: %w", t.ID, err))
		}
		if updated == 1 {
			taskIDs = append(taskIDs, t.ID)
		}
	}

	// Update all ClaimedAt timestamps to claim shell tasks
	shellTaskIDs := make([]int, 0, len(shellTasks))
	for _, t := range shellTasks {
		updated, err := client.ShellTask.Update().
			Where(shelltask.ID(t.ID), shelltask.ClaimedAtIsNil()).
			SetClaimedAt(now).
			Save(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to update shell task %d: %w", t.ID, err))
		}
		if updated == 1 {
			shellTaskIDs = append(shellTaskIDs, t.ID)
		}
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
