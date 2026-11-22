package c2

import (
	"context"
	"testing"
	"time"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
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
	now := time.Now()

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

	// Should only have 1 task
	count := client.Task.Query().CountX(ctx)
	assert.Equal(t, 1, count, "Should only create one task despite multiple triggers matching")
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

func setupOccurrenceTest(t *testing.T) (context.Context, *ent.Client, *Server, [2]*ent.Beacon, [2]*ent.Host) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)

	var hosts [2]*ent.Host
	var beacons [2]*ent.Beacon
	for i, id := range []string{"occ-1", "occ-2"} {
		hosts[i] = client.Host.Create().
			SetIdentifier(id).
			SetPlatform(c2pb.Host_PLATFORM_LINUX).
			SaveX(ctx)
		beacons[i] = client.Beacon.Create().
			SetIdentifier(id).
			SetHost(hosts[i]).
			SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
			SaveX(ctx)
	}

	tome := client.Tome.Create().
		SetName("Occurrence Tome").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('occ')").
		SaveX(ctx)
	client.ScheduledTask.Create().
		SetName("Occurrence ST").
		SetDescription("Test").
		SetTome(tome).
		SetRunOnSchedule("* * * * *").
		SaveX(ctx)

	return ctx, client, &Server{graph: client}, beacons, hosts
}

func TestHandleTomeAutomation_OccurrenceSharedQuest(t *testing.T) {
	ctx, client, srv, beacons, hosts := setupOccurrenceTest(t)
	defer client.Close()

	now := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)
	for i := range beacons {
		srv.handleTomeAutomation(ctx, beacons[i].ID, hosts[i].ID, false, false, now, 0)
	}

	quests := client.Quest.Query().AllX(ctx)
	assert.Len(t, quests, 1, "beacons firing the same occurrence should share one quest")
	if assert.NotNil(t, quests[0].ScheduledAt) {
		assert.True(t, quests[0].ScheduledAt.Equal(now))
	}
	assert.Equal(t, 2, quests[0].QueryTasks().CountX(ctx), "each beacon should get a task on the shared quest")
}

func TestHandleTomeAutomation_OccurrenceDifferentMinutes(t *testing.T) {
	ctx, client, srv, beacons, hosts := setupOccurrenceTest(t)
	defer client.Close()

	now := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)
	srv.handleTomeAutomation(ctx, beacons[0].ID, hosts[0].ID, false, false, now, 0)
	srv.handleTomeAutomation(ctx, beacons[1].ID, hosts[1].ID, false, false, now.Add(5*time.Minute), 0)

	assert.Equal(t, 2, client.Quest.Query().CountX(ctx), "different occurrences should create separate quests")
	assert.Equal(t, 2, client.Task.Query().CountX(ctx))
}

func TestHandleTomeAutomation_OccurrenceLookAheadShared(t *testing.T) {
	ctx, client, srv, beacons, hosts := setupOccurrenceTest(t)
	defer client.Close()

	base := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)
	srv.handleTomeAutomation(ctx, beacons[0].ID, hosts[0].ID, false, false, base.Add(30*time.Second), 60*time.Second)
	srv.handleTomeAutomation(ctx, beacons[1].ID, hosts[1].ID, false, false, base.Add(50*time.Second), 60*time.Second)

	quests := client.Quest.Query().AllX(ctx)
	assert.Len(t, quests, 1)
	if assert.NotNil(t, quests[0].ScheduledAt) {
		assert.True(t, quests[0].ScheduledAt.Equal(base.Add(time.Minute)))
	}
	assert.Equal(t, 2, client.Task.Query().CountX(ctx))
}

func TestHandleTomeAutomation_OccurrenceRangeShared(t *testing.T) {
	ctx, client, srv, beacons, hosts := setupOccurrenceTest(t)
	defer client.Close()

	client.ScheduledTask.Update().SetRunOnSchedule("0-5 10 * * *").ExecX(ctx)

	base := time.Date(2023, 10, 27, 10, 2, 0, 0, time.UTC)
	srv.handleTomeAutomation(ctx, beacons[0].ID, hosts[0].ID, false, false, base.Add(5*time.Second), 0)
	srv.handleTomeAutomation(ctx, beacons[1].ID, hosts[1].ID, false, false, base.Add(40*time.Second), 0)

	quests := client.Quest.Query().AllX(ctx)
	assert.Len(t, quests, 1)
	if assert.NotNil(t, quests[0].ScheduledAt) {
		assert.True(t, quests[0].ScheduledAt.Equal(base))
	}
	assert.Equal(t, 2, client.Task.Query().CountX(ctx))
}

func TestHandleTomeAutomation_OneShotUngrouped(t *testing.T) {
	ctx, client, srv, beacons, hosts := setupOccurrenceTest(t)
	defer client.Close()

	client.ScheduledTask.Update().SetRunOnSchedule("").SetRunOnNewBeaconCallback(true).ExecX(ctx)

	now := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)
	for i := range beacons {
		srv.handleTomeAutomation(ctx, beacons[i].ID, hosts[i].ID, true, false, now, 0)
	}

	quests := client.Quest.Query().AllX(ctx)
	assert.Len(t, quests, 2)
	for _, q := range quests {
		assert.Nil(t, q.ScheduledAt)
	}
}

func TestHandleTomeAutomation_OccurrenceDifferentScheduledTasks(t *testing.T) {
	ctx, client, srv, beacons, hosts := setupOccurrenceTest(t)
	defer client.Close()

	tome2 := client.Tome.Create().
		SetName("Occurrence Tome 2").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('occ2')").
		SaveX(ctx)
	client.ScheduledTask.Create().
		SetName("Occurrence ST 2").
		SetDescription("Test").
		SetTome(tome2).
		SetRunOnSchedule("* * * * *").
		SaveX(ctx)

	now := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)
	for i := range beacons {
		srv.handleTomeAutomation(ctx, beacons[i].ID, hosts[i].ID, false, false, now, 0)
	}

	assert.Equal(t, 2, client.Quest.Query().CountX(ctx))
	assert.Equal(t, 4, client.Task.Query().CountX(ctx))
}

func TestHandleTomeAutomation_DedupLookAheadAcrossCheckins(t *testing.T) {
	ctx, client, srv, beacons, hosts := setupOccurrenceTest(t)
	defer client.Close()

	base := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)
	srv.handleTomeAutomation(ctx, beacons[0].ID, hosts[0].ID, false, false, base.Add(30*time.Second), 60*time.Second)
	srv.handleTomeAutomation(ctx, beacons[0].ID, hosts[0].ID, false, false, base.Add(45*time.Second), 60*time.Second)

	assert.Equal(t, 1, client.Quest.Query().CountX(ctx))
	assert.Equal(t, 1, client.Task.Query().CountX(ctx))
}

func TestHandleTomeAutomation_DedupRangeWithinMinute(t *testing.T) {
	ctx, client, srv, beacons, hosts := setupOccurrenceTest(t)
	defer client.Close()

	client.ScheduledTask.Update().SetRunOnSchedule("0-5 10 * * *").ExecX(ctx)

	base := time.Date(2023, 10, 27, 10, 2, 0, 0, time.UTC)
	for _, sec := range []int{5, 20, 50} {
		srv.handleTomeAutomation(ctx, beacons[0].ID, hosts[0].ID, false, false, base.Add(time.Duration(sec)*time.Second), 0)
	}

	assert.Equal(t, 1, client.Quest.Query().CountX(ctx))
	assert.Equal(t, 1, client.Task.Query().CountX(ctx))
}

func TestHandleTomeAutomation_DedupIsPerBeacon(t *testing.T) {
	ctx, client, srv, beacons, hosts := setupOccurrenceTest(t)
	defer client.Close()

	now := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)
	for i := range beacons {
		srv.handleTomeAutomation(ctx, beacons[i].ID, hosts[i].ID, false, false, now, 0)
	}

	quests := client.Quest.Query().AllX(ctx)
	assert.Len(t, quests, 1)
	assert.Equal(t, 2, quests[0].QueryTasks().CountX(ctx))
}

func TestHandleTomeAutomation_DedupNextOccurrenceRuns(t *testing.T) {
	ctx, client, srv, beacons, hosts := setupOccurrenceTest(t)
	defer client.Close()

	now := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)
	srv.handleTomeAutomation(ctx, beacons[0].ID, hosts[0].ID, false, false, now, 0)
	assert.Equal(t, 1, client.Task.Query().CountX(ctx))

	srv.handleTomeAutomation(ctx, beacons[0].ID, hosts[0].ID, false, false, now.Add(time.Minute), 0)
	assert.Equal(t, 2, client.Quest.Query().CountX(ctx))
	assert.Equal(t, 2, client.Task.Query().CountX(ctx))
}

func TestHandleTomeAutomation_DedupCountsClaimedTasks(t *testing.T) {
	ctx, client, srv, beacons, hosts := setupOccurrenceTest(t)
	defer client.Close()

	now := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)
	srv.handleTomeAutomation(ctx, beacons[0].ID, hosts[0].ID, false, false, now, 0)
	assert.Equal(t, 1, client.Task.Query().CountX(ctx))

	client.Task.Update().SetClaimedAt(now).ExecX(ctx)

	srv.handleTomeAutomation(ctx, beacons[0].ID, hosts[0].ID, false, false, now.Add(10*time.Second), 0)
	assert.Equal(t, 1, client.Quest.Query().CountX(ctx))
	assert.Equal(t, 1, client.Task.Query().CountX(ctx), "claimed task must still dedup the occurrence")
}

func TestHandleTomeAutomation_OneShotNotDeduped(t *testing.T) {
	ctx, client, srv, beacons, hosts := setupOccurrenceTest(t)
	defer client.Close()

	client.ScheduledTask.Update().SetRunOnSchedule("").SetRunOnNewBeaconCallback(true).ExecX(ctx)

	now := time.Date(2023, 10, 27, 10, 0, 0, 0, time.UTC)
	srv.handleTomeAutomation(ctx, beacons[0].ID, hosts[0].ID, true, false, now, 0)

	quests := client.Quest.Query().AllX(ctx)
	if assert.Len(t, quests, 1) {
		assert.Nil(t, quests[0].ScheduledAt)
		assert.Equal(t, 1, quests[0].QueryTasks().CountX(ctx))
	}
	assert.Equal(t, 1, client.Task.Query().CountX(ctx))
}
