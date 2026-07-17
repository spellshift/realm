package mcp_test

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/mark3labs/mcp-go/mcp"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent/tome"
	tavernmcp "realm.pub/tavern/internal/mcp"
)

func TestHandleListQuests(t *testing.T) {
	client := setupTestDB(t)
	defer client.Close()

	ctx := context.WithValue(context.Background(), tavernmcp.ContextKey{}, client)

	// Create user
	user1 := client.User.Create().
		SetOauthID("oauth-1").
		SetName("Alice").
		SetPhotoURL("http://example.com/photo").
		SaveX(ctx)

	// Create mock tomes
	tome1 := client.Tome.Create().
		SetName("discovery-tome").
		SetTactic(tome.TacticDISCOVERY).
		SetDescription("Discover things").
		SetParamDefs(`[{"name": "p1", "type":"string"}]`).
		SetAuthor("Alice").
		SetEldritch("console.log('hello')").
		SaveX(ctx)

	tome2 := client.Tome.Create().
		SetName("exec-tome").
		SetTactic(tome.TacticEXECUTION).
		SetDescription("Execute things").
		SetParamDefs(`[]`).
		SetAuthor("Bob").
		SetEldritch("console.log('world')").
		SaveX(ctx)

	quest1 := client.Quest.Create().
		SetName("test-quest-1").
		SetParameters(`{"p1": "v1"}`).
		SetTome(tome1).
		SetCreator(user1).
		SaveX(ctx)

	quest2 := client.Quest.Create().
		SetName("test-quest-2").
		SetParameters(`{}`).
		SetTome(tome2).
		SaveX(ctx)
	_ = quest2

	req := mcp.CallToolRequest{}
	res, err := tavernmcp.HandleListQuests(ctx, req)
	require.NoError(t, err)
	require.NotNil(t, res)
	require.False(t, res.IsError)

	var results []map[string]interface{}
	textContent := res.Content[0].(mcp.TextContent).Text
	err = json.Unmarshal([]byte(textContent), &results)
	require.NoError(t, err)

	assert.Len(t, results, 2)

	var q1Res map[string]interface{}
	var q2Res map[string]interface{}
	if results[0]["id"].(float64) == float64(quest1.ID) {
		q1Res = results[0]
		q2Res = results[1]
	} else {
		q1Res = results[1]
		q2Res = results[0]
	}

	assert.Equal(t, "test-quest-1", q1Res["name"])
	assert.Equal(t, "Alice", q1Res["creator"])
	assert.Equal(t, `{"p1": "v1"}`, q1Res["parameters"])
	assert.Equal(t, "discovery-tome", q1Res["tomeName"])
	assert.Equal(t, "DISCOVERY", q1Res["tomeTactic"])
	assert.Equal(t, "Discover things", q1Res["tomeDescription"])
	assert.NotEmpty(t, q1Res["createdAt"])

	assert.Equal(t, "test-quest-2", q2Res["name"])
	assert.NotContains(t, q2Res, "creator")
	assert.Equal(t, `{}`, q2Res["parameters"])
	assert.Equal(t, "exec-tome", q2Res["tomeName"])
	assert.Equal(t, "EXECUTION", q2Res["tomeTactic"])
	assert.Equal(t, "Execute things", q2Res["tomeDescription"])
	assert.NotEmpty(t, q2Res["createdAt"])

	// test no client error
	res, err = tavernmcp.HandleListQuests(context.Background(), req)
	require.NoError(t, err)
	require.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "internal error: no database client")
}
