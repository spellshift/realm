package c2

import (
	"context"
	"testing"
	"time"

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
			if column.Name == "scheduled_occurrence_at" || column.Name == "scheduled_occurrence_key" || column.Name == "scheduled_dedup_key" || column.Name == "has_scheduled_snapshot" {
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
	const questID = 5001
	now := time.Now()
	_, err = db.ExecContext(ctx, "INSERT INTO quests (id, created_at, last_modified_at, name, quest_tome, scheduled_task_quests) VALUES (?, ?, ?, ?, ?, ?)", questID, now, now, "legacy-quest", tome.ID, st.ID)
	require.NoError(t, err)
	task := client.Task.Create().SetQuestID(questID).SetBeacon(beacon).SaveX(ctx)

	require.NoError(t, client.Schema.Create(ctx))
	require.Equal(t, questID, client.Quest.GetX(ctx, questID).ID)
	require.Equal(t, task.ID, client.Task.GetX(ctx, task.ID).ID)
	require.Empty(t, client.Quest.GetX(ctx, questID).ScheduledOccurrenceKey)
	require.False(t, client.Quest.GetX(ctx, questID).HasScheduledSnapshot)
	require.Empty(t, client.Task.GetX(ctx, task.ID).ScheduledDedupKey)
}
