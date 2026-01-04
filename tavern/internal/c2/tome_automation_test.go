package c2

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	_ "github.com/mattn/go-sqlite3"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestHandleTomeAutomation(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
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
		SetTransport(c2pb.ActiveTransport_TRANSPORT_HTTP1).
		SaveX(ctx)

	// 1. Setup Tomes
	// T1: New Beacon Only
	client.Tome.Create().
		SetName("Tome New Beacon").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('new beacon')").
		SetRunOnNewBeaconCallback(true).
		SaveX(ctx)

	// T2: New Host Only
	client.Tome.Create().
		SetName("Tome New Host").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('new host')").
		SetRunOnFirstHostCallback(true).
		SaveX(ctx)

	// T3: Schedule Matching (Every minute)
	client.Tome.Create().
		SetName("Tome Schedule Match").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('schedule')").
		SetRunOnSchedule("* * * * *").
		SaveX(ctx)

	// T4: Schedule Matching with Host Restriction (Allowed)
	client.Tome.Create().
		SetName("Tome Schedule Restricted Allowed").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('schedule restricted')").
		SetRunOnSchedule("* * * * *").
		AddScheduledHosts(h).
		SaveX(ctx)

	// T5: Schedule Matching with Host Restriction (Denied - different host)
	otherHost := client.Host.Create().
		SetIdentifier("other").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)

	client.Tome.Create().
		SetName("Tome Schedule Restricted Denied").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('schedule denied')").
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
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
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
		SetTransport(c2pb.ActiveTransport_TRANSPORT_HTTP1).
		SaveX(ctx)

	// Tome with ALL triggers enabled
	client.Tome.Create().
		SetName("Super Tome").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('super')").
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
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
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
		SetTransport(c2pb.ActiveTransport_TRANSPORT_HTTP1).
		SaveX(ctx)

	// Schedule: 3:01:00 PM -> "1 15 * * *"
	client.Tome.Create().
		SetName("Scheduled Tome").
		SetDescription("Test").
		SetAuthor("Test Author").
		SetEldritch("print('schedule')").
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
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer client.Close()

	srv := &Server{graph: client}
	// 06:05:00
	now := time.Date(2023, 10, 27, 6, 5, 0, 0, time.UTC)

	h := client.Host.Create().SetIdentifier("h").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	b := client.Beacon.Create().SetIdentifier("b").SetHost(h).SetTransport(c2pb.ActiveTransport_TRANSPORT_HTTP1).SaveX(ctx)

	// Range Schedule: "* 6-12 * * *" (Every minute of hours 6-12)
	client.Tome.Create().
		SetName("Range Tome").
		SetDescription("Test").
		SetAuthor("Test").
		SetEldritch("print('range')").
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
