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

func TestHandleListTomes(t *testing.T) {
	client := setupTestDB(t)
	defer client.Close()

	ctx := context.WithValue(context.Background(), tavernmcp.ContextKey{}, client)

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
	_ = tome2

	req := mcp.CallToolRequest{}
	res, err := tavernmcp.HandleListTomes(ctx, req)
	require.NoError(t, err)
	require.NotNil(t, res)
	require.False(t, res.IsError)

	var results []map[string]interface{}
	textContent := res.Content[0].(mcp.TextContent).Text
	err = json.Unmarshal([]byte(textContent), &results)
	require.NoError(t, err)

	assert.Len(t, results, 2)

	var t1Res map[string]interface{}
	var t2Res map[string]interface{}
	if results[0]["id"].(float64) == float64(tome1.ID) {
		t1Res = results[0]
		t2Res = results[1]
	} else {
		t1Res = results[1]
		t2Res = results[0]
	}

	assert.Equal(t, "discovery-tome", t1Res["name"])
	assert.Equal(t, "DISCOVERY", t1Res["tactic"])
	assert.Equal(t, "Discover things", t1Res["description"])
	assert.Equal(t, `[{"name": "p1", "type":"string"}]`, t1Res["paramDefs"])

	assert.Equal(t, "exec-tome", t2Res["name"])
	assert.Equal(t, "EXECUTION", t2Res["tactic"])
	assert.Equal(t, "Execute things", t2Res["description"])
	assert.Equal(t, `[]`, t2Res["paramDefs"])

	// test no client error
	res, err = tavernmcp.HandleListTomes(context.Background(), req)
	require.NoError(t, err)
	require.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "internal error: no database client")
}
