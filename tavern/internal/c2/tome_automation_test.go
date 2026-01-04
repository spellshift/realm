package c2

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
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

			srv.handleTomeAutomation(ctx, b.ID, h.ID, tt.isNewBeacon, tt.isNewHost, now)

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
	srv.handleTomeAutomation(ctx, b.ID, h.ID, true, true, now)

	// Should only have 1 task
	count := client.Task.Query().CountX(ctx)
	assert.Equal(t, 1, count, "Should only create one task despite multiple triggers matching")
}
