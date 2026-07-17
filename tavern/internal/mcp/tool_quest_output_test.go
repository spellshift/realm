package mcp_test

import (
	"context"
	"encoding/json"
	"fmt"
	"testing"
	"time"

	"github.com/mark3labs/mcp-go/mcp"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/tome"
	tavernmcp "realm.pub/tavern/internal/mcp"
)

func TestHandleQuestOutput(t *testing.T) {
	client := setupTestDB(t)
	defer client.Close()

	ctx := context.WithValue(context.Background(), tavernmcp.ContextKey{}, client)

	host1 := client.Host.Create().
		SetIdentifier("host-1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SetName("web-server").
		SetPrimaryIP("192.168.1.10").
		SetExternalIP("203.0.113.10").
		SetLastSeenAt(time.Now()).
		SaveX(ctx)

	beacon1 := client.Beacon.Create().
		SetHost(host1).
		SetName("beacon-1").
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
		SetPrincipal("admin").
		SaveX(ctx)

	tome1 := client.Tome.Create().
		SetName("discovery-tome").
		SetTactic(tome.TacticDISCOVERY).
		SetDescription("Discover things").
		SetParamDefs(`[]`).
		SetAuthor("Alice").
		SetEldritch("console.log('hello')").
		SaveX(ctx)

	quest1 := client.Quest.Create().
		SetName("test-quest-1").
		SetParameters(`{}`).
		SetTome(tome1).
		SaveX(ctx)

	client.Task.Create().
		SetQuest(quest1).
		SetBeacon(beacon1).
		SetExecFinishedAt(time.Now()).
		SetOutput("Output from task 1").
		SaveX(ctx)

	client.Task.Create().
		SetQuest(quest1).
		SetBeacon(beacon1).
		SetExecFinishedAt(time.Now()).
		SetOutput("Output from task 2").
		SaveX(ctx)

	req := mcp.CallToolRequest{}
	req.Params.Arguments = map[string]interface{}{
		"ids": []interface{}{fmt.Sprintf("%d", quest1.ID)},
	}

	res, err := tavernmcp.HandleQuestOutput(ctx, req)
	require.NoError(t, err)
	require.NotNil(t, res)
	require.False(t, res.IsError)

	var results []map[string]interface{}
	textContent := res.Content[0].(mcp.TextContent).Text
	err = json.Unmarshal([]byte(textContent), &results)
	require.NoError(t, err)

	assert.Len(t, results, 1)

	q1Res := results[0]
	assert.Equal(t, float64(quest1.ID), q1Res["id"])
	assert.Equal(t, "test-quest-1", q1Res["name"])

	tasks := q1Res["tasks"].([]interface{})
	assert.Len(t, tasks, 2)

	task1 := tasks[0].(map[string]interface{})
	assert.Equal(t, "Output from task 1", task1["output"])
	beaconInfo := task1["beacon"].(map[string]interface{})
	assert.Equal(t, float64(beacon1.ID), beaconInfo["id"])
	assert.Equal(t, "beacon-1", beaconInfo["name"])
	assert.Equal(t, float64(host1.ID), beaconInfo["hostId"])
	assert.Equal(t, "web-server", beaconInfo["hostName"])

	task2 := tasks[1].(map[string]interface{})
	assert.Equal(t, "Output from task 2", task2["output"])

	// Test invalid ids format
	req.Params.Arguments = map[string]interface{}{
		"ids": "not-an-array",
	}
	res, err = tavernmcp.HandleQuestOutput(ctx, req)
	require.NoError(t, err)
	require.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "invalid ids")

	// test no client error
	res, err = tavernmcp.HandleQuestOutput(context.Background(), req)
	require.NoError(t, err)
	require.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "internal error: no database client")
}
