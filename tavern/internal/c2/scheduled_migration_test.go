package c2

import (
	"context"
	"testing"

	entsql "entgo.io/ent/dialect/sql"
	sqlschema "entgo.io/ent/dialect/sql/schema"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/migrate"
)

func TestScheduledOccurrenceMigrationPreservesExistingRows(t *testing.T) {
	ctx := context.Background()
	db := enttest.NewDB(t)
	client := ent.NewClient(ent.Driver(entsql.OpenDB("sqlite3", db)))
	defer client.Close()

	oldTables, err := sqlschema.CopyTables(migrate.Tables)
	require.NoError(t, err)
	for _, table := range oldTables {
		if table.Name != "quests" && table.Name != "tasks" {
			continue
		}
		columns := table.Columns[:0]
		for _, column := range table.Columns {
			if column.Name == "scheduled_occurrence_at" || column.Name == "scheduled_occurrence_key" || column.Name == "scheduled_dedup_key" {
				continue
			}
			columns = append(columns, column)
		}
		table.Columns = columns
	}
	require.NoError(t, migrate.Create(ctx, client.Schema, oldTables))

	host := client.Host.Create().SetIdentifier("legacy-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	beacon := client.Beacon.Create().SetIdentifier("legacy-beacon").SetHost(host).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)
	tome := client.Tome.Create().SetName("legacy-tome").SetDescription("Test").
		SetAuthor("Test").SetEldritch("print('legacy')").SaveX(ctx)
	st := client.ScheduledTask.Create().SetName("legacy-schedule").SetDescription("Test").
		SetTome(tome).SetRunOnSchedule("* * * * *").SaveX(ctx)
	quest := client.Quest.Create().SetName("legacy-quest").SetTome(tome).SetScheduledTask(st).SaveX(ctx)
	task := client.Task.Create().SetQuest(quest).SetBeacon(beacon).SaveX(ctx)

	require.NoError(t, client.Schema.Create(ctx))
	require.Equal(t, quest.ID, client.Quest.GetX(ctx, quest.ID).ID)
	require.Equal(t, task.ID, client.Task.GetX(ctx, task.ID).ID)
	require.Empty(t, client.Quest.GetX(ctx, quest.ID).ScheduledOccurrenceKey)
	require.Empty(t, client.Task.GetX(ctx, task.ID).ScheduledDedupKey)
}
