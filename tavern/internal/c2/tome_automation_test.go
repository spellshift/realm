package c2

import (
	"context"
	"sync"
	"testing"
	"time"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/quest"
	"realm.pub/tavern/internal/ent/scheduledtask"
	"realm.pub/tavern/internal/ent/task"
)

func TestHandleTomeAutomation(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()

	srv := &Server{graph: client}
	now := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)

	// Create a dummy host and beacon for testing
	h := client.Host.Create().
		SetIdentifier("test-host").
		SetName("Test Host").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)
	b := client.Beacon.Create().
		SetIdentifier("test-beacon").
		SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
		SaveX(ctx)

	// 1. Setup Tomes
	// T1: New Beacon Only
	t1 := client.Tome.Create().
		SetName("Tome New Beacon").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('new beacon')").
		SaveX(ctx)
	client.ScheduledTask.Create().
		SetName("ST New Beacon").
		SetDescription("Test").
		SetTome(t1).
		SetRunOnNewBeaconCallback(true).
		SaveX(ctx)

	// T2: New Host Only
	t2 := client.Tome.Create().
		SetName("Tome New Host").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('new host')").
		SaveX(ctx)
	client.ScheduledTask.Create().
		SetName("ST New Host").
		SetDescription("Test").
		SetTome(t2).
		SetRunOnFirstHostCallback(true).
		SaveX(ctx)

	// T3: Schedule Matching (Every minute)
	t3 := client.Tome.Create().
		SetName("Tome Schedule Match").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('schedule')").
		SaveX(ctx)
	client.ScheduledTask.Create().
		SetName("ST Schedule Match").
		SetDescription("Test").
		SetTome(t3).
		SetRunOnSchedule("* * * * *").
		SaveX(ctx)

	// T4: Schedule Matching with Host Restriction (Allowed)
	t4 := client.Tome.Create().
		SetName("Tome Schedule Restricted Allowed").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('schedule restricted')").
		SaveX(ctx)
	client.ScheduledTask.Create().
		SetName("ST Schedule Restricted Allowed").
		SetDescription("Test").
		SetTome(t4).
		SetRunOnSchedule("* * * * *").
		AddScheduledHosts(h).
		SaveX(ctx)

	// T5: Schedule Matching with Host Restriction (Denied - different host)
	otherHost := client.Host.Create().
		SetIdentifier("other").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)

	t5 := client.Tome.Create().
		SetName("Tome Schedule Restricted Denied").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('schedule denied')").
		SaveX(ctx)
	client.ScheduledTask.Create().
		SetName("ST Schedule Restricted Denied").
		SetDescription("Test").
		SetTome(t5).
		SetRunOnSchedule("* * * * *").
		AddScheduledHosts(otherHost).
		SaveX(ctx)

	tests := []struct {
		name          string
		isNewBeacon   bool
		isNewHost     bool
		expectedTomes []string
	}{
		{
			name:        "New Beacon Only",
			isNewBeacon: true,
			isNewHost:   false,
			expectedTomes: []string{
				"Tome New Beacon",
				"Tome Schedule Match",
				"Tome Schedule Restricted Allowed",
			},
		},
		{
			name:        "New Host Only",
			isNewBeacon: false,
			isNewHost:   true,
			expectedTomes: []string{
				"Tome New Host",
				"Tome Schedule Match",
				"Tome Schedule Restricted Allowed",
			},
		},
		{
			name:        "Both New",
			isNewBeacon: true,
			isNewHost:   true,
			expectedTomes: []string{
				"Tome New Beacon",
				"Tome New Host",
				"Tome Schedule Match",
				"Tome Schedule Restricted Allowed",
			},
		},
		{
			name:        "Neither New",
			isNewBeacon: false,
			isNewHost:   false,
			expectedTomes: []string{
				"Tome Schedule Match",
				"Tome Schedule Restricted Allowed",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Clear existing quests/tasks to ensure clean slate
			client.Task.Delete().ExecX(ctx)
			client.Quest.Delete().ExecX(ctx)

			// Use 0 interval to match "current minute" logic (cutoff = now)
			// effectively checking sched.Next(now-1s) <= now
			srv.handleTomeAutomation(ctx, b.ID, h.ID, tt.isNewBeacon, tt.isNewHost, now, 0)

			// Verify Tasks
			tasks := client.Task.Query().WithQuest(func(q *ent.QuestQuery) {
				q.WithTome()
			}).AllX(ctx)

			var createdTomes []string
			for _, t := range tasks {
				createdTomes = append(createdTomes, t.Edges.Quest.Edges.Tome.Name)
			}

			assert.ElementsMatch(t, tt.expectedTomes, createdTomes)
		})
	}
}

func TestHandleTomeAutomation_Deduplication(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()

	srv := &Server{graph: client}
	now := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)

	h := client.Host.Create().
		SetIdentifier("test").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)
	b := client.Beacon.Create().
		SetIdentifier("test").
		SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
		SaveX(ctx)

	// Tome with ALL triggers enabled
	t1 := client.Tome.Create().
		SetName("Super Tome").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('super')").
		SaveX(ctx)
	client.ScheduledTask.Create().
		SetName("Super ST").
		SetDescription("Test").
		SetTome(t1).
		SetRunOnNewBeaconCallback(true).
		SetRunOnFirstHostCallback(true).
		SetRunOnSchedule("* * * * *").
		SaveX(ctx)

	// Trigger all conditions
	srv.handleTomeAutomation(ctx, b.ID, h.ID, true, true, now, 0)

	// Both event triggers share one quest, while the cron trigger has its own.
	count := client.Task.Query().CountX(ctx)
	assert.Equal(t, 2, count)
	assert.Equal(t, 2, client.Quest.Query().CountX(ctx))
	assert.False(t, client.Quest.Query().Where(quest.ScheduledOccurrenceAtIsNil()).OnlyX(ctx).QueryBundle().ExistX(ctx))
}

func TestHandleTomeAutomation_IntervalWindow(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()

	srv := &Server{graph: client}
	// Date: 3:00:56 PM
	now := time.Date(2023, 10, 27, 15, 0, 56, 0, time.UTC)

	h := client.Host.Create().
		SetIdentifier("test-host").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)
	b := client.Beacon.Create().
		SetIdentifier("test-beacon").
		SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
		SaveX(ctx)

	// Schedule: 3:01:00 PM -> "1 15 * * *"
	t1 := client.Tome.Create().
		SetName("Scheduled Tome").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('schedule')").
		SaveX(ctx)
	client.ScheduledTask.Create().
		SetName("Scheduled ST").
		SetDescription("Test").
		SetTome(t1).
		SetRunOnSchedule("1 15 * * *").
		SaveX(ctx)

	// 1. First Check-in at 3:00:56 PM. Interval 120s.
	// Window: [3:00:56, 3:02:56].
	// Schedule 3:01:00 is in window.
	srv.handleTomeAutomation(ctx, b.ID, h.ID, false, false, now, 120*time.Second)

	count := client.Task.Query().CountX(ctx)
	assert.Equal(t, 1, count, "Tome should be queued at 3:00:56 for 3:01:00 schedule")

	// Clear tasks
	client.Task.Delete().ExecX(ctx)
	client.Quest.Delete().ExecX(ctx)

	// 2. Second Check-in at 3:02:56 PM. Interval 120s.
	// Next checkin: 3:04:56.
	// Schedule 3:01:00 (tomorrow) is NOT in window.
	nextCheckin := now.Add(2 * time.Minute) // 3:02:56
	srv.handleTomeAutomation(ctx, b.ID, h.ID, false, false, nextCheckin, 120*time.Second)

	count = client.Task.Query().CountX(ctx)
	assert.Equal(t, 0, count, "Tome should NOT be queued at 3:02:56")
}

func TestHandleTomeAutomation_CronRange(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()

	srv := &Server{graph: client}
	// 06:05:00
	now := time.Date(2023, 10, 27, 6, 5, 0, 0, time.UTC)

	h := client.Host.Create().SetIdentifier("h").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	b := client.Beacon.Create().SetIdentifier("b").SetHost(h).SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)

	// Range Schedule: "* 6-12 * * *" (Every minute of hours 6-12)
	t1 := client.Tome.Create().
		SetName("Range Tome").
		SetDescription("Test").
		SetAuthor("Test").
		SetEldritch("print('range')").
		SaveX(ctx)
	client.ScheduledTask.Create().
		SetName("Range ST").
		SetDescription("Test").
		SetTome(t1).
		SetRunOnSchedule("* 6-12 * * *").
		SaveX(ctx)

	// 1. Current time 06:05:00. Matches range. Should run.
	// Interval irrelevant (using 1h to prove lookahead is ignored if range logic applies)
	srv.handleTomeAutomation(ctx, b.ID, h.ID, false, false, now, 1*time.Hour)
	count := client.Task.Query().CountX(ctx)
	assert.Equal(t, 1, count, "Range tome should run at 06:05:00")

	client.Task.Delete().ExecX(ctx)
	client.Quest.Delete().ExecX(ctx)

	// 2. Current time 05:55:00. Does NOT match range.
	// Lookahead (1h) would see 06:00:00.
	// But Range logic should IGNORE lookahead.
	early := now.Add(-10 * time.Minute) // 05:55:00
	srv.handleTomeAutomation(ctx, b.ID, h.ID, false, false, early, 1*time.Hour)
	count = client.Task.Query().CountX(ctx)
	assert.Equal(t, 0, count, "Range tome should NOT run at 05:55:00 even with lookahead")
}

func TestHandleTomeAutomation_SharedOccurrence(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()
	srv := &Server{graph: client}
	h := client.Host.Create().SetIdentifier("shared-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	beacons := make([]*ent.Beacon, 2)
	for i, id := range []string{"shared-a", "shared-b"} {
		beacons[i] = client.Beacon.Create().SetIdentifier(id).SetHost(h).
			SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)
	}
	tome := client.Tome.Create().SetName("shared-tome").SetDescription("Test").
		SetAuthor("Test").SetEldritch("print('first')").
		SetParamDefs(`[{"name":"version","label":"Version","type":"string"}]`).SaveX(ctx)
	st := client.ScheduledTask.Create().SetName("shared-schedule").SetDescription("Test").
		SetTome(tome).SetParameters(`{"version":"first"}`).SetRunOnSchedule("* * * * *").SaveX(ctx)
	now := time.Date(2023, 10, 27, 10, 0, 56, 0, time.UTC)
	srv.handleTomeAutomation(ctx, beacons[0].ID, h.ID, false, false, now, 10*time.Second)
	srv.handleTomeAutomation(ctx, beacons[0].ID, h.ID, false, false, now, 10*time.Second)
	require.Equal(t, 1, client.Task.Query().CountX(ctx))
	require.Equal(t, 1, client.Quest.Query().CountX(ctx))

	client.ScheduledTask.UpdateOne(st).SetParameters(`{"version":"second"}`).ExecX(ctx)
	client.Tome.UpdateOne(tome).SetEldritch("print('second')").ExecX(ctx)
	srv.handleTomeAutomation(ctx, beacons[1].ID, h.ID, false, false, now.Add(time.Second), 10*time.Second)
	require.Equal(t, 2, client.Task.Query().CountX(ctx))
	q := client.Quest.Query().OnlyX(ctx)
	assert.Equal(t, now.Add(4*time.Second), q.ScheduledOccurrenceAt)
	assert.Equal(t, `{"version":"first"}`, q.Parameters)
	assert.Equal(t, "print('first')", q.EldritchAtCreation)
	assert.Equal(t, tome.ParamDefs, q.ParamDefsAtCreation)
	assert.False(t, q.QueryBundle().ExistX(ctx))

	srv.handleTomeAutomation(ctx, beacons[0].ID, h.ID, false, false, now.Add(60*time.Second), 10*time.Second)
	require.Equal(t, 2, client.Quest.Query().CountX(ctx))
	client.ScheduledTask.UpdateOne(st).SetDisabled(true).ExecX(ctx)
	srv.handleTomeAutomation(ctx, beacons[1].ID, h.ID, false, false, now.Add(61*time.Second), 10*time.Second)
	assert.Equal(t, 3, client.Task.Query().CountX(ctx))
}

func TestCreateAutomatedTaskOnceCronDeduplicatesByQuestAndBeacon(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()
	srv := &Server{graph: client}
	h := client.Host.Create().SetIdentifier("cron-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	first := client.Beacon.Create().SetIdentifier("cron-first").SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)
	second := client.Beacon.Create().SetIdentifier("cron-second").SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)
	tome := client.Tome.Create().SetName("cron-tome").SetDescription("Test").
		SetAuthor("Test").SetEldritch("print('cron')").SaveX(ctx)
	st := client.ScheduledTask.Create().SetName("cron-schedule").SetDescription("Test").
		SetTome(tome).SaveX(ctx)
	occurrence := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)

	require.NoError(t, srv.createAutomatedTaskOnce(ctx, st.ID, first.ID, h.ID, &occurrence))
	require.NoError(t, srv.createAutomatedTaskOnce(ctx, st.ID, first.ID, h.ID, &occurrence))
	require.Equal(t, 1, client.Quest.Query().CountX(ctx))
	require.Equal(t, 1, client.Task.Query().CountX(ctx))

	require.NoError(t, srv.createAutomatedTaskOnce(ctx, st.ID, second.ID, h.ID, &occurrence))
	require.Equal(t, 1, client.Quest.Query().CountX(ctx))
	q := client.Quest.Query().OnlyX(ctx)
	tasks := q.QueryTasks().AllX(ctx)
	require.Len(t, tasks, 2)
	beaconIDs := make(map[int]struct{}, len(tasks))
	for _, task := range tasks {
		beaconIDs[task.QueryBeacon().OnlyIDX(ctx)] = struct{}{}
	}
	require.Equal(t, map[int]struct{}{first.ID: {}, second.ID: {}}, beaconIDs)
}

func TestCreateAutomatedTaskOnceCronOccurrenceScope(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()
	srv := &Server{graph: client}
	h := client.Host.Create().SetIdentifier("scope-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	first := client.Beacon.Create().SetIdentifier("scope-first").SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)
	second := client.Beacon.Create().SetIdentifier("scope-second").SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)
	tome := client.Tome.Create().SetName("scope-tome").SetDescription("Test").
		SetAuthor("Test").SetEldritch("print('scope')").SaveX(ctx)
	st1 := client.ScheduledTask.Create().SetName("scope-one").SetDescription("Test").SetTome(tome).SaveX(ctx)
	st2 := client.ScheduledTask.Create().SetName("scope-two").SetDescription("Test").SetTome(tome).SaveX(ctx)
	at := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)
	otherZone := time.FixedZone("other", 2*60*60)
	sameInstant := at.In(otherZone)
	next := at.Add(time.Minute)

	require.NoError(t, srv.createAutomatedTaskOnce(ctx, st1.ID, first.ID, h.ID, &at))
	q1 := st1.QueryQuests().OnlyX(ctx)
	require.NoError(t, srv.createAutomatedTaskOnce(ctx, st1.ID, second.ID, h.ID, &sameInstant))
	require.Equal(t, q1.ID, client.Task.Query().Where(task.HasBeaconWith(beacon.ID(second.ID))).OnlyX(ctx).QueryQuest().OnlyIDX(ctx))
	require.Equal(t, 1, st1.QueryQuests().CountX(ctx))
	require.True(t, at.Equal(st1.QueryQuests().OnlyX(ctx).ScheduledOccurrenceAt))

	require.NoError(t, srv.createAutomatedTaskOnce(ctx, st1.ID, first.ID, h.ID, &next))
	require.Equal(t, 2, st1.QueryQuests().CountX(ctx))
	require.NoError(t, srv.createAutomatedTaskOnce(ctx, st2.ID, first.ID, h.ID, &at))
	require.Equal(t, 1, st2.QueryQuests().CountX(ctx))
	require.NotEqual(t, q1.ID, st2.QueryQuests().OnlyIDX(ctx))
}

func TestCreateAutomatedTaskOnceCronDuplicateQuestsSelectLowestID(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()
	srv := &Server{graph: client}
	h := client.Host.Create().SetIdentifier("duplicates-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	b := client.Beacon.Create().SetIdentifier("duplicates-beacon").SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)
	tome := client.Tome.Create().SetName("duplicates-tome").SetDescription("Test").
		SetAuthor("Test").SetEldritch("print('duplicates')").SaveX(ctx)
	st := client.ScheduledTask.Create().SetName("duplicates-schedule").SetDescription("Test").SetTome(tome).SaveX(ctx)
	at := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)
	first := client.Quest.Create().SetName("first").SetTome(tome).SetScheduledTask(st).
		SetScheduledOccurrenceAt(at).SaveX(ctx)
	client.Quest.Create().SetName("second").SetTome(tome).SetScheduledTask(st).
		SetScheduledOccurrenceAt(at).SaveX(ctx)

	require.NoError(t, srv.createAutomatedTaskOnce(ctx, st.ID, b.ID, h.ID, &at))
	require.Equal(t, 2, st.QueryQuests().CountX(ctx))
	require.Equal(t, first.ID, client.Task.Query().OnlyX(ctx).QueryQuest().OnlyIDX(ctx))
}

func TestCreateAutomatedTaskOnceEventCallbacksCreateSeparateQuests(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()
	srv := &Server{graph: client}
	h := client.Host.Create().SetIdentifier("event-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	b := client.Beacon.Create().SetIdentifier("event-beacon").SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)
	tome := client.Tome.Create().SetName("event-tome").SetDescription("Test").
		SetAuthor("Test").SetEldritch("print('event')").SaveX(ctx)
	st := client.ScheduledTask.Create().SetName("event-schedule").SetDescription("Test").
		SetTome(tome).SaveX(ctx)

	require.NoError(t, srv.createAutomatedTaskOnce(ctx, st.ID, b.ID, h.ID, nil))
	require.NoError(t, srv.createAutomatedTaskOnce(ctx, st.ID, b.ID, h.ID, nil))
	quests := client.Quest.Query().AllX(ctx)
	require.Len(t, quests, 2)
	require.Equal(t, 2, client.Task.Query().CountX(ctx))
	for _, q := range quests {
		require.Equal(t, b.ID, q.QueryTasks().OnlyX(ctx).QueryBeacon().OnlyIDX(ctx))
	}
}

func TestCreateAutomatedTaskOnce_ConcurrentCallbacks(t *testing.T) {
	ctx := context.Background()
	client, db := enttest.OpenTempDBWithDB(t)
	defer client.Close()
	// Serialize the goroutines' DB access; assert one Task per beacon and no errors.
	db.SetMaxOpenConns(1)
	srv := &Server{graph: client}
	h := client.Host.Create().SetIdentifier("parallel-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	tome := client.Tome.Create().SetName("parallel-tome").SetDescription("Test").
		SetAuthor("Test").SetEldritch("print('parallel')").SaveX(ctx)
	st := client.ScheduledTask.Create().SetName("parallel-schedule").SetDescription("Test").
		SetTome(tome).SetRunOnSchedule("* * * * *").SaveX(ctx)
	beacons := make([]*ent.Beacon, 4)
	for i, id := range []string{"parallel-a", "parallel-b", "parallel-c", "parallel-d"} {
		beacons[i] = client.Beacon.Create().SetIdentifier(id).SetHost(h).
			SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)
	}
	now := time.Date(2023, 10, 27, 10, 0, 56, 0, time.UTC)
	occurrence := now.Add(4 * time.Second)
	var wg sync.WaitGroup
	errs := make([]error, len(beacons))
	for i, b := range beacons {
		wg.Add(1)
		go func(i, id int) {
			defer wg.Done()
			errs[i] = srv.createAutomatedTaskOnce(ctx, st.ID, id, h.ID, &occurrence)
		}(i, b.ID)
	}
	wg.Wait()
	for _, err := range errs {
		require.NoError(t, err)
	}
	quests := st.QueryQuests().AllX(ctx)
	require.NotEmpty(t, quests)
	for _, q := range quests {
		assert.True(t, occurrence.Equal(q.ScheduledOccurrenceAt))
	}
	for _, b := range beacons {
		assert.Equal(t, 1, client.Task.Query().Where(task.HasBeaconWith(beacon.ID(b.ID)),
			task.HasQuestWith(quest.HasScheduledTaskWith(scheduledtask.ID(st.ID)), quest.ScheduledOccurrenceAt(occurrence))).CountX(ctx))
	}
}

func TestScheduledCreationRechecksDisabledAndHostRestriction(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()
	srv := &Server{graph: client}
	allowedHost := client.Host.Create().SetIdentifier("allowed-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	otherHost := client.Host.Create().SetIdentifier("other-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	b := client.Beacon.Create().SetIdentifier("restricted-beacon").SetHost(otherHost).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)
	tome := client.Tome.Create().SetName("restricted-tome").SetDescription("test").
		SetAuthor("test").SetEldritch("print('restricted')").SaveX(ctx)
	st := client.ScheduledTask.Create().SetName("restricted-schedule").SetDescription("test").
		SetTome(tome).AddScheduledHosts(allowedHost).SetRunOnSchedule("* * * * *").SaveX(ctx)
	occurrence := time.Date(2023, 10, 27, 10, 1, 0, 0, time.UTC)
	require.NoError(t, srv.createAutomatedTaskOnce(ctx, st.ID, b.ID, otherHost.ID, &occurrence))
	require.Zero(t, client.Quest.Query().CountX(ctx))
	client.ScheduledTask.UpdateOne(st).ClearScheduledHosts().SetDisabled(true).ExecX(ctx)
	require.NoError(t, srv.createAutomatedTaskOnce(ctx, st.ID, b.ID, otherHost.ID, &occurrence))
	require.Zero(t, client.Quest.Query().CountX(ctx))
}

func TestHandleTomeAutomationPreservesCronLookback(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()
	srv := &Server{graph: client}
	h := client.Host.Create().SetIdentifier("lookback-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	b := client.Beacon.Create().SetIdentifier("lookback-beacon").SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)
	tome := client.Tome.Create().SetName("lookback-tome").SetDescription("test").
		SetAuthor("test").SetEldritch("print('lookback')").SaveX(ctx)
	client.ScheduledTask.Create().SetName("lookback-schedule").SetDescription("test").
		SetTome(tome).SetRunOnSchedule("* * * * *").SaveX(ctx)
	now := time.Date(2023, 10, 27, 10, 0, 0, 500_000_000, time.UTC)
	srv.handleTomeAutomation(ctx, b.ID, h.ID, false, false, now, 0)
	require.Equal(t, 1, client.Task.Query().CountX(ctx))
	require.Equal(t, now.Truncate(time.Minute), client.Quest.Query().OnlyX(ctx).ScheduledOccurrenceAt)
	srv.handleTomeAutomation(ctx, b.ID, h.ID, false, false, now.Add(2*time.Second), 0)
	require.Equal(t, 1, client.Task.Query().CountX(ctx))
}
