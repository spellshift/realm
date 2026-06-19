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
	"realm.pub/tavern/internal/ent/tag"
	"realm.pub/tavern/internal/ent/tome"
	tavernmcp "realm.pub/tavern/internal/mcp"
)

func TestHandleListHosts(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	ctxWithClient := tavernmcp.ContextWithClient(ctx, client)

	host := client.Host.Create().
		SetIdentifier("host-1").
		SetName("Host 1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SetPrimaryIP("192.168.1.1").
		SaveX(ctx)

	client.Beacon.Create().
		SetHost(host).
		SetName("beacon-1").
		SetPrincipal("root").
		SetTransport(c2pb.Transport_TRANSPORT_UNSPECIFIED).
		SaveX(ctx)

	client.Tag.Create().
		SetName("linux").
		SetKind(tag.KindGroup).
		AddHosts(host).
		SaveX(ctx)

	req := mcp.CallToolRequest{}
	res, err := tavernmcp.HandleListHosts(ctxWithClient, req)
	require.NoError(t, err)
	assert.False(t, res.IsError)
	assert.Len(t, res.Content, 1)

	textContent, ok := res.Content[0].(mcp.TextContent)
	require.True(t, ok)

	var hosts []map[string]any
	err = json.Unmarshal([]byte(textContent.Text), &hosts)
	require.NoError(t, err)

	assert.Len(t, hosts, 1)
	assert.Equal(t, "Host 1", hosts[0]["name"])
	assert.Equal(t, "192.168.1.1", hosts[0]["primaryIP"])

	tags := hosts[0]["tags"].([]any)
	assert.Len(t, tags, 1)
	assert.Equal(t, "linux", tags[0].(map[string]any)["name"])

	beacons := hosts[0]["beacons"].([]any)
	assert.Len(t, beacons, 1)
	assert.Equal(t, "beacon-1", beacons[0].(map[string]any)["name"])
}

func TestHandleListQuests(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	ctxWithClient := tavernmcp.ContextWithClient(ctx, client)

	testTome := client.Tome.Create().
		SetName("test-tome").
		SetDescription("A test tome").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('hello')").
		SetHash("abc123").
		SaveX(ctx)

	client.Quest.Create().
		SetName("quest-1").
		SetParameters(`{}`).
		SetTome(testTome).
		SetParamDefsAtCreation("[]").
		SetEldritchAtCreation("print('hello')").
		SaveX(ctx)

	req := mcp.CallToolRequest{}
	res, err := tavernmcp.HandleListQuests(ctxWithClient, req)
	require.NoError(t, err)
	assert.False(t, res.IsError)
	assert.Len(t, res.Content, 1)

	textContent, ok := res.Content[0].(mcp.TextContent)
	require.True(t, ok)

	var quests []map[string]any
	err = json.Unmarshal([]byte(textContent.Text), &quests)
	require.NoError(t, err)

	assert.Len(t, quests, 1)
	assert.Equal(t, "quest-1", quests[0]["name"])
	assert.Equal(t, `{}`, quests[0]["parameters"])
	assert.Equal(t, "test-tome", quests[0]["tomeName"])
}

func TestHandleListTomes(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	ctxWithClient := tavernmcp.ContextWithClient(ctx, client)

	client.Tome.Create().
		SetName("tome-1").
		SetDescription("Tome 1").
		SetAuthor("author-1").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('1')").
		SetHash("hash1").
		SetParamDefs(`[{"name":"p1"}]`).
		SaveX(ctx)

	req := mcp.CallToolRequest{}
	res, err := tavernmcp.HandleListTomes(ctxWithClient, req)
	require.NoError(t, err)
	assert.False(t, res.IsError)
	assert.Len(t, res.Content, 1)

	textContent, ok := res.Content[0].(mcp.TextContent)
	require.True(t, ok)

	var tomes []map[string]any
	err = json.Unmarshal([]byte(textContent.Text), &tomes)
	require.NoError(t, err)

	assert.Len(t, tomes, 1)
	assert.Equal(t, "tome-1", tomes[0]["name"])
	assert.Equal(t, "Tome 1", tomes[0]["description"])
	assert.Equal(t, `[{"name":"p1"}]`, tomes[0]["paramDefs"])
}

func TestHandleQuestOutput(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	ctxWithClient := tavernmcp.ContextWithClient(ctx, client)

	testTome := client.Tome.Create().
		SetName("test-tome").
		SetDescription("A test tome").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('hello')").
		SetHash("abc123").
		SaveX(ctx)

	host := client.Host.Create().
		SetIdentifier("host-1").
		SetName("Host 1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)

	beacon := client.Beacon.Create().
		SetHost(host).
		SetName("beacon-1").
		SetTransport(c2pb.Transport_TRANSPORT_UNSPECIFIED).
		SaveX(ctx)

	quest := client.Quest.Create().
		SetName("quest-output").
		SetParameters(`{}`).
		SetTome(testTome).
		SetParamDefsAtCreation("[]").
		SetEldritchAtCreation("print('hello')").
		SaveX(ctx)

	client.Task.Create().
		SetQuest(quest).
		SetBeacon(beacon).
		SetOutput("success output").
		SetExecFinishedAt(time.Now()).
		SaveX(ctx)

	req := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]any{
				"ids": []any{fmt.Sprintf("%d", quest.ID)},
			},
		},
	}
	res, err := tavernmcp.HandleQuestOutput(ctxWithClient, req)
	require.NoError(t, err)
	assert.False(t, res.IsError)
	assert.Len(t, res.Content, 1)

	textContent, ok := res.Content[0].(mcp.TextContent)
	require.True(t, ok)

	var quests []map[string]any
	err = json.Unmarshal([]byte(textContent.Text), &quests)
	require.NoError(t, err)

	assert.Len(t, quests, 1)
	assert.Equal(t, "quest-output", quests[0]["name"])

	tasks := quests[0]["tasks"].([]any)
	assert.Len(t, tasks, 1)
	assert.Equal(t, "success output", tasks[0].(map[string]any)["output"])

	bInfo := tasks[0].(map[string]any)["beacon"].(map[string]any)
	assert.Equal(t, "beacon-1", bInfo["name"])
	assert.Equal(t, "Host 1", bInfo["hostName"])
}

func TestHandleWaitForQuest(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	ctxWithClient := tavernmcp.ContextWithClient(ctx, client)

	testTome := client.Tome.Create().
		SetName("test-tome").
		SetDescription("A test tome").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('hello')").
		SetHash("abc123").
		SaveX(ctx)

	host := client.Host.Create().
		SetIdentifier("host-1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)

	beacon := client.Beacon.Create().
		SetHost(host).
		SetName("beacon-1").
		SetTransport(c2pb.Transport_TRANSPORT_UNSPECIFIED).
		SaveX(ctx)

	quest := client.Quest.Create().
		SetName("wait-quest").
		SetParameters(`{}`).
		SetTome(testTome).
		SetParamDefsAtCreation("[]").
		SetEldritchAtCreation("print('hello')").
		SaveX(ctx)

	client.Task.Create().
		SetQuest(quest).
		SetBeacon(beacon).
		SetOutput("done").
		SetExecFinishedAt(time.Now()).
		SaveX(ctx)

	req := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]any{
				"quest_id": fmt.Sprintf("%d", quest.ID),
			},
		},
	}
	res, err := tavernmcp.HandleWaitForQuest(ctxWithClient, req)
	require.NoError(t, err)
	assert.False(t, res.IsError)
	assert.Len(t, res.Content, 1)

	textContent, ok := res.Content[0].(mcp.TextContent)
	require.True(t, ok)

	var result map[string]any
	err = json.Unmarshal([]byte(textContent.Text), &result)
	require.NoError(t, err)

	assert.Contains(t, result["message"], "have finished")

	tasks := result["tasks"].([]any)
	assert.Len(t, tasks, 1)
	assert.Equal(t, "beacon-1", tasks[0].(map[string]any)["beacon_name"])
}
