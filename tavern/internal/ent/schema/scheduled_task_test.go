package schema_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestHostCanBelongToMultipleScheduledTasks(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	ctx := context.Background()

	host := graph.Host.Create().
		SetIdentifier("host-1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)
	tome := graph.Tome.Create().
		SetName("test-tome").
		SetDescription("test description").
		SetAuthor("test-author").
		SetEldritch("print('hello')").
		SaveX(ctx)

	firstTask := graph.ScheduledTask.Create().
		SetName("scheduled-task-1").
		SetDescription("first scheduled task").
		SetTome(tome).
		AddScheduledHosts(host).
		SaveX(ctx)
	secondTask := graph.ScheduledTask.Create().
		SetName("scheduled-task-2").
		SetDescription("second scheduled task").
		SetTome(tome).
		AddScheduledHosts(host).
		SaveX(ctx)

	firstTaskHosts, err := firstTask.QueryScheduledHosts().All(ctx)
	require.NoError(t, err)
	require.Len(t, firstTaskHosts, 1)
	assert.Equal(t, host.ID, firstTaskHosts[0].ID)

	secondTaskHosts, err := secondTask.QueryScheduledHosts().All(ctx)
	require.NoError(t, err)
	require.Len(t, secondTaskHosts, 1)
	assert.Equal(t, host.ID, secondTaskHosts[0].ID)

	hostScheduledTasks, err := host.QueryScheduledTasks().All(ctx)
	require.NoError(t, err)
	assert.ElementsMatch(t, []int{firstTask.ID, secondTask.ID}, []int{hostScheduledTasks[0].ID, hostScheduledTasks[1].ID})
}
