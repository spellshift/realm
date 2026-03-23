package graphql

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/tome"
	"realm.pub/tavern/internal/graphql/models"
)

func TestMetrics_QuestTimelineChart(t *testing.T) {
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer client.Close()

	ctx := context.Background()
	resolver := &metricsResolver{
		Resolver: &Resolver{client: client},
	}

	// Create some tomes with different tactics
	tome1, err := client.Tome.Create().
		SetName("tome1").
		SetDescription("test tome 1").
		SetAuthor("test_author").
		SetEldritch("test").
		SetTactic(tome.TacticRECON).
		Save(ctx)
	require.NoError(t, err)

	tome2, err := client.Tome.Create().
		SetName("tome2").
		SetDescription("test tome 2").
		SetAuthor("test_author").
		SetEldritch("test").
		SetTactic(tome.TacticEXECUTION).
		Save(ctx)
	require.NoError(t, err)

	now := time.Now()
	startTime := now.Add(-10 * time.Second)

	// Create quests at different times
	// Quest 1 at startTime + 1s (Bucket 1)
	_, err = client.Quest.Create().
		SetName("quest1").
		SetTome(tome1).
		SetCreatedAt(startTime.Add(1 * time.Second)).
		Save(ctx)
	require.NoError(t, err)

	// Quest 2 at startTime + 1s (Bucket 1)
	_, err = client.Quest.Create().
		SetName("quest2").
		SetTome(tome2).
		SetCreatedAt(startTime.Add(1 * time.Second)).
		Save(ctx)
	require.NoError(t, err)

	// Quest 3 at startTime + 3s (Bucket 3)
	_, err = client.Quest.Create().
		SetName("quest3").
		SetTome(tome1).
		SetCreatedAt(startTime.Add(3 * time.Second)).
		Save(ctx)
	require.NoError(t, err)

	// Empty bucket at startTime + 2s

	// Test QuestTimelineChart
	granularitySeconds := 1
	buckets, err := resolver.QuestTimelineChart(ctx, &models.Metrics{}, startTime, &now, granularitySeconds, nil)
	require.NoError(t, err)

	// Verify we have the correct number of buckets (10 seconds, granularity 1s = 11 buckets)
	expectedBuckets := 11
	require.Len(t, buckets, expectedBuckets)

	// Verify Bucket 1 (startTime + 1s)
	bucket1 := buckets[1]
	require.Equal(t, startTime.Add(1*time.Second).Truncate(time.Second), bucket1.StartTimestamp.Truncate(time.Second))
	require.Equal(t, 2, bucket1.Count)
	require.Len(t, bucket1.GroupByTactic, 2)

	tacticCounts := make(map[tome.Tactic]int)
	for _, tb := range bucket1.GroupByTactic {
		tacticCounts[tb.Tactic] = tb.Count
	}
	require.Equal(t, 1, tacticCounts[tome.TacticRECON])
	require.Equal(t, 1, tacticCounts[tome.TacticEXECUTION])

	// Verify Bucket 2 (startTime + 2s) - should be empty
	bucket2 := buckets[2]
	require.Equal(t, 0, bucket2.Count)
	require.Len(t, bucket2.GroupByTactic, 0)

	// Verify Bucket 3 (startTime + 3s)
	bucket3 := buckets[3]
	require.Equal(t, 1, bucket3.Count)
	require.Len(t, bucket3.GroupByTactic, 1)
	require.Equal(t, tome.TacticRECON, bucket3.GroupByTactic[0].Tactic)
	require.Equal(t, 1, bucket3.GroupByTactic[0].Count)
}
