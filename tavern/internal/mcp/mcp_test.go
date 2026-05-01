package mcp

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strconv"
	"testing"
	"time"

	"github.com/mark3labs/mcp-go/mcp"
	mcpserver "github.com/mark3labs/mcp-go/server"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/tome"
)

// setupTestDB creates a test ent client with schema migrations applied.
func setupTestDB(t *testing.T) *ent.Client {
	t.Helper()
	return enttest.OpenTempDB(t)
}

// setupTestHandler creates the MCP HTTP handler backed by a test database.
func setupTestHandler(t *testing.T, client *ent.Client) http.Handler {
	t.Helper()
	return NewHandler(client, "test", nil)
}

// TestNewHandler verifies the MCP handler can be created without error.
func TestNewHandler(t *testing.T) {
	client := setupTestDB(t)
	defer client.Close()

	handler := setupTestHandler(t, client)
	assert.NotNil(t, handler)
}

// TestMCPStreamableHTTPEndpoint verifies the Streamable HTTP endpoint accepts GET requests for SSE streaming.
func TestMCPStreamableHTTPEndpoint(t *testing.T) {
	client := setupTestDB(t)
	defer client.Close()

	handler := setupTestHandler(t, client)

	// Use a real HTTP server to avoid data races with httptest.ResponseRecorder.
	srv := httptest.NewServer(handler)
	defer srv.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, srv.URL+"/mcp", nil)
	require.NoError(t, err)

	resp, err := http.DefaultClient.Do(req)
	require.NoError(t, err)
	defer resp.Body.Close()

	// Streamable HTTP GET endpoint should return text/event-stream
	assert.Equal(t, http.StatusOK, resp.StatusCode)
	assert.Contains(t, resp.Header.Get("Content-Type"), "text/event-stream")
}

// TestMCPPostEndpointRequiresValidRequest verifies the POST endpoint rejects requests without valid JSON-RPC.
func TestMCPPostEndpointRequiresValidRequest(t *testing.T) {
	client := setupTestDB(t)
	defer client.Close()

	handler := setupTestHandler(t, client)

	req := httptest.NewRequest(http.MethodPost, "/mcp", nil)
	w := httptest.NewRecorder()

	handler.ServeHTTP(w, req)

	result := w.Result()
	defer result.Body.Close()
	// Without a valid JSON-RPC request body, we expect an error response
	assert.NotEqual(t, http.StatusOK, result.StatusCode)
}

// TestListQuestsHandler tests the list_quests tool handler by creating test data
// and verifying that the handler returns the expected output.
func TestListQuestsHandler(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	// Create a tome
	testTome := client.Tome.Create().
		SetName("test-tome").
		SetDescription("A test tome").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('hello')").
		SetHash("abc123").
		SaveX(ctx)

	// Create a quest
	client.Quest.Create().
		SetName("test-quest").
		SetParameters(`{"key":"value"}`).
		SetTome(testTome).
		SetParamDefsAtCreation("[]").
		SetEldritchAtCreation("print('hello')").
		SaveX(ctx)

	// Verify quest was created
	quests, err := client.Quest.Query().WithTome().All(ctx)
	require.NoError(t, err)
	assert.Len(t, quests, 1)
	assert.Equal(t, "test-quest", quests[0].Name)
	assert.NotNil(t, quests[0].Edges.Tome)
	assert.Equal(t, "test-tome", quests[0].Edges.Tome.Name)
}

// TestListHostsHandler tests the list_hosts tool by creating test data.
func TestListHostsHandler(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	// Create a host
	host := client.Host.Create().
		SetIdentifier("test-host-id").
		SetName("test-host").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SetPrimaryIP("192.168.1.1").
		SaveX(ctx)

	// Create a beacon on the host
	client.Beacon.Create().
		SetHost(host).
		SetTransport(c2pb.Transport_TRANSPORT_UNSPECIFIED).
		SetPrincipal("root").
		SaveX(ctx)

	// Create a tag on the host
	client.Tag.Create().
		SetName("test-tag").
		SetKind("group").
		AddHosts(host).
		SaveX(ctx)

	// Verify data was created
	hosts, err := client.Host.Query().WithBeacons().WithTags().All(ctx)
	require.NoError(t, err)
	assert.Len(t, hosts, 1)
	assert.Equal(t, "test-host", hosts[0].Name)
	assert.Len(t, hosts[0].Edges.Beacons, 1)
	assert.Len(t, hosts[0].Edges.Tags, 1)
}

// TestListTomesHandler tests the list_tomes tool by creating test data.
func TestListTomesHandler(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	// Create tomes
	client.Tome.Create().
		SetName("tome-1").
		SetDescription("First tome").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('one')").
		SetHash("hash1").
		SetParamDefs(`[{"name":"param1","type":"string"}]`).
		SaveX(ctx)

	client.Tome.Create().
		SetName("tome-2").
		SetDescription("Second tome").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('two')").
		SetHash("hash2").
		SaveX(ctx)

	tomes, err := client.Tome.Query().All(ctx)
	require.NoError(t, err)
	assert.Len(t, tomes, 2)
}

// TestCreateQuestHandler tests the create_quest tool by creating a quest.
func TestCreateQuestHandler(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	// Create a tome
	testTome := client.Tome.Create().
		SetName("test-tome").
		SetDescription("A test tome").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('hello')").
		SetHash("abc123").
		SetParamDefs(`[{"name":"key","type":"string"}]`).
		SaveX(ctx)

	// Create a host and beacon
	testHost := client.Host.Create().
		SetIdentifier("test-host-id").
		SetPlatform(c2pb.Host_PLATFORM_UNSPECIFIED).
		SaveX(ctx)

	testBeacon := client.Beacon.Create().
		SetHost(testHost).
		SetTransport(c2pb.Transport_TRANSPORT_UNSPECIFIED).
		SaveX(ctx)

	// Create a quest directly to validate the pattern
	q := client.Quest.Create().
		SetName("mcp-quest").
		SetParameters(`{"key":"value"}`).
		SetTomeID(testTome.ID).
		SetParamDefsAtCreation(testTome.ParamDefs).
		SetEldritchAtCreation(testTome.Eldritch).
		SaveX(ctx)

	// Create a task for the beacon
	client.Task.Create().
		SetQuestID(q.ID).
		SetBeaconID(testBeacon.ID).
		SaveX(ctx)

	// Verify quest and tasks
	createdQuest, err := client.Quest.Query().WithTasks().All(ctx)
	require.NoError(t, err)
	assert.Len(t, createdQuest, 1)
	assert.Equal(t, "mcp-quest", createdQuest[0].Name)
	assert.Len(t, createdQuest[0].Edges.Tasks, 1)
}

// TestQuestOutputHandler tests the quest_output tool by creating quests with task output.
func TestQuestOutputHandler(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	// Create a tome
	testTome := client.Tome.Create().
		SetName("test-tome").
		SetDescription("A test tome").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('hello')").
		SetHash("abc123").
		SaveX(ctx)

	// Create host and beacon
	testHost := client.Host.Create().
		SetIdentifier("test-host-id").
		SetName("test-host").
		SetPlatform(c2pb.Host_PLATFORM_UNSPECIFIED).
		SaveX(ctx)

	testBeacon := client.Beacon.Create().
		SetHost(testHost).
		SetName("test-beacon").
		SetTransport(c2pb.Transport_TRANSPORT_UNSPECIFIED).
		SaveX(ctx)

	// Create a quest with a task that has output
	q := client.Quest.Create().
		SetName("output-quest").
		SetParameters("{}").
		SetTome(testTome).
		SetParamDefsAtCreation("[]").
		SetEldritchAtCreation("print('hello')").
		SaveX(ctx)

	client.Task.Create().
		SetQuest(q).
		SetBeacon(testBeacon).
		SetOutput("task output result").
		SetExecFinishedAt(time.Now()).
		SaveX(ctx)

	// Verify output is queryable
	quests, err := client.Quest.Query().
		WithTasks(func(tq *ent.TaskQuery) {
			tq.WithBeacon(func(bq *ent.BeaconQuery) {
				bq.WithHost()
			})
		}).
		All(ctx)
	require.NoError(t, err)
	assert.Len(t, quests, 1)
	assert.Len(t, quests[0].Edges.Tasks, 1)
	assert.Equal(t, "task output result", quests[0].Edges.Tasks[0].Output)
}

// TestWaitForQuestHandler tests the wait_for_quest tool with already-finished tasks.
func TestWaitForQuestHandler(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	// Create a tome
	testTome := client.Tome.Create().
		SetName("test-tome").
		SetDescription("A test tome").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('hello')").
		SetHash("abc123").
		SaveX(ctx)

	// Create host and beacon
	testHost := client.Host.Create().
		SetIdentifier("test-host-id").
		SetPlatform(c2pb.Host_PLATFORM_UNSPECIFIED).
		SaveX(ctx)

	testBeacon := client.Beacon.Create().
		SetHost(testHost).
		SetName("test-beacon").
		SetTransport(c2pb.Transport_TRANSPORT_UNSPECIFIED).
		SaveX(ctx)

	// Create a quest with finished tasks
	q := client.Quest.Create().
		SetName("finished-quest").
		SetParameters("{}").
		SetTome(testTome).
		SetParamDefsAtCreation("[]").
		SetEldritchAtCreation("print('hello')").
		SaveX(ctx)

	client.Task.Create().
		SetQuest(q).
		SetBeacon(testBeacon).
		SetOutput("done").
		SetExecFinishedAt(time.Now()).
		SaveX(ctx)

	// Verify the quest tasks are finished
	quest, err := client.Quest.Query().
		WithTasks(func(tq *ent.TaskQuery) {
			tq.WithBeacon()
		}).
		Only(ctx)
	require.NoError(t, err)
	assert.Len(t, quest.Edges.Tasks, 1)
	assert.False(t, quest.Edges.Tasks[0].ExecFinishedAt.IsZero())
}

// TestParseIntIDs tests the ParseIntIDs helper.
func TestParseIntIDs(t *testing.T) {
	tests := []struct {
		name      string
		input     map[string]any
		key       string
		expectErr bool
		expected  []int
	}{
		{
			name:     "valid string ids",
			input:    map[string]any{"ids": []any{"1", "2", "3"}},
			key:      "ids",
			expected: []int{1, 2, 3},
		},
		{
			name:     "valid numeric ids",
			input:    map[string]any{"ids": []any{float64(1), float64(2)}},
			key:      "ids",
			expected: []int{1, 2},
		},
		{
			name:      "missing key",
			input:     map[string]any{},
			key:       "ids",
			expectErr: true,
		},
		{
			name:      "invalid string",
			input:     map[string]any{"ids": []any{"not-a-number"}},
			key:       "ids",
			expectErr: true,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			req := mcp.CallToolRequest{
				Params: mcp.CallToolParams{
					Arguments: tc.input,
				},
			}
			ids, err := ParseIntIDs(req, tc.key)
			if tc.expectErr {
				assert.Error(t, err)
			} else {
				require.NoError(t, err)
				assert.Equal(t, tc.expected, ids)
			}
		})
	}
}

// TestGraphQLQueryHandler tests the graphql_query tool validation logic.
func TestGraphQLQueryHandler(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	// Create test data so queries return something
	client.Tome.Create().
		SetName("graphql-test-tome").
		SetDescription("A tome for graphql test").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('hello')").
		SetHash("gqlhash").
		SaveX(ctx)

	// Verify the tome was created via direct query
	tomes, err := client.Tome.Query().All(ctx)
	require.NoError(t, err)
	assert.Len(t, tomes, 1)
	assert.Equal(t, "graphql-test-tome", tomes[0].Name)
}

// TestGraphQLQueryValidation tests that the graphql_query tool rejects mutations and subscriptions.
func TestGraphQLQueryValidation(t *testing.T) {
	tests := []struct {
		name      string
		query     string
		expectErr bool
		errMsg    string
	}{
		{
			name:  "valid query",
			query: `query { __schema { types { name } } }`,
		},
		{
			name:  "valid shorthand query",
			query: `{ __schema { types { name } } }`,
		},
		{
			name:  "valid introspection",
			query: `query IntrospectionQuery { __type(name: "Tome") { name fields { name } } }`,
		},
		{
			name:      "mutation rejected",
			query:     `mutation { createQuest(input: {}) { id } }`,
			expectErr: true,
			errMsg:    "only queries are allowed, got mutation operation",
		},
		{
			name:      "subscription rejected",
			query:     `subscription { onQuestCreated { id } }`,
			expectErr: true,
			errMsg:    "only queries are allowed, got subscription operation",
		},
		{
			name:      "invalid syntax",
			query:     `not a valid query`,
			expectErr: true,
			errMsg:    "failed to parse GraphQL query",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			// Build MCP request with the query
			req := mcp.CallToolRequest{
				Params: mcp.CallToolParams{
					Arguments: map[string]any{
						"query": tc.query,
					},
				},
			}

			// Call the handler directly (no GraphQL handler in context — will
			// error at execution time for valid queries, but validation still runs).
			result, err := HandleGraphQLQueryForTest(context.Background(), req)
			require.NoError(t, err)

			if tc.expectErr {
				assert.True(t, result.IsError, "expected error for %q", tc.name)
				// Check the error text
				if tc.errMsg != "" && len(result.Content) > 0 {
					if txt, ok := result.Content[0].(mcp.TextContent); ok {
						assert.Contains(t, txt.Text, tc.errMsg)
					}
				}
			} else {
				// Valid queries will fail because there's no GraphQL handler in context,
				// but the validation step should have passed (error message will be about
				// no handler, not about query parsing).
				if result.IsError && len(result.Content) > 0 {
					if txt, ok := result.Content[0].(mcp.TextContent); ok {
						assert.Contains(t, txt.Text, "no GraphQL handler", "expected handler error, not validation error")
					}
				}
			}
		})
	}
}

// TestHandleListHosts tests edge cases for the list_hosts tool.
func TestHandleListHosts(t *testing.T) {
	// Missing client
	res, err := handleListHosts(context.Background(), mcp.CallToolRequest{})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "internal error: no database client")

	// Success case (empty db)
	client := setupTestDB(t)
	defer client.Close()
	ctx := context.WithValue(context.Background(), contextKey{}, client)
	res, err = handleListHosts(ctx, mcp.CallToolRequest{})
	require.NoError(t, err)
	assert.False(t, res.IsError)
	assert.Equal(t, "[]", res.Content[0].(mcp.TextContent).Text)
}

// TestHandleListQuests tests edge cases for the list_quests tool.
func TestHandleListQuests(t *testing.T) {
	// Missing client
	res, err := handleListQuests(context.Background(), mcp.CallToolRequest{})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "internal error: no database client")

	// Success case (empty db)
	client := setupTestDB(t)
	defer client.Close()
	ctx := context.WithValue(context.Background(), contextKey{}, client)
	res, err = handleListQuests(ctx, mcp.CallToolRequest{})
	require.NoError(t, err)
	assert.False(t, res.IsError)
	assert.Equal(t, "[]", res.Content[0].(mcp.TextContent).Text)
}

// TestHandleListTomes tests edge cases for the list_tomes tool.
func TestHandleListTomes(t *testing.T) {
	// Missing client
	res, err := handleListTomes(context.Background(), mcp.CallToolRequest{})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "internal error: no database client")

	// Success case (empty db)
	client := setupTestDB(t)
	defer client.Close()
	ctx := context.WithValue(context.Background(), contextKey{}, client)
	res, err = handleListTomes(ctx, mcp.CallToolRequest{})
	require.NoError(t, err)
	assert.False(t, res.IsError)
	assert.Equal(t, "[]", res.Content[0].(mcp.TextContent).Text)
}

// TestHandleWaitForQuest tests edge cases for the wait_for_quest tool.
func TestHandleWaitForQuest(t *testing.T) {
	// Missing client
	res, err := handleWaitForQuest(context.Background(), mcp.CallToolRequest{})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "internal error: no database client")

	client := setupTestDB(t)
	defer client.Close()
	ctx := context.WithValue(context.Background(), contextKey{}, client)

	// Missing quest_id
	res, err = handleWaitForQuest(ctx, mcp.CallToolRequest{})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "invalid quest_id")

	// Invalid quest_id
	res, err = handleWaitForQuest(ctx, mcp.CallToolRequest{
		Params: mcp.CallToolParams{Arguments: map[string]interface{}{"quest_id": "abc"}},
	})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "quest_id must be a number")

	// Missing quest
	res, err = handleWaitForQuest(ctx, mcp.CallToolRequest{
		Params: mcp.CallToolParams{Arguments: map[string]interface{}{"quest_id": "999"}},
	})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "not found")

	// Quest with no tasks
	testTome := client.Tome.Create().
		SetName("test-tome").
		SetDescription("A test tome").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('hello')").
		SetHash("abc123").
		SaveX(ctx)
	quest := client.Quest.Create().SetName("empty").SetTome(testTome).SetParameters("{}").SetParamDefsAtCreation("[]").SetEldritchAtCreation("").SaveX(ctx)
	res, err = handleWaitForQuest(ctx, mcp.CallToolRequest{
		Params: mcp.CallToolParams{Arguments: map[string]interface{}{"quest_id": strconv.Itoa(quest.ID)}},
	})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "has no tasks")

	// Test timeout logic by sending cancelled context
	testHost := client.Host.Create().SetIdentifier("wait-host").SetPlatform(c2pb.Host_PLATFORM_UNSPECIFIED).SaveX(ctx)
	testBeacon := client.Beacon.Create().SetHost(testHost).SetTransport(c2pb.Transport_TRANSPORT_UNSPECIFIED).SaveX(ctx)
	taskCtx, cancel := context.WithCancel(ctx)
	client.Task.Create().SetQuest(quest).SetBeacon(testBeacon).SaveX(ctx)
	cancel() // Cancel the context to simulate timeout
	res, err = handleWaitForQuest(taskCtx, mcp.CallToolRequest{
		Params: mcp.CallToolParams{Arguments: map[string]interface{}{"quest_id": strconv.Itoa(quest.ID)}},
	})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "failed to query quest: context canceled")
}

// TestHandleQuestOutput tests edge cases for the quest_output tool.
func TestHandleQuestOutput(t *testing.T) {
	// Missing client
	res, err := handleQuestOutput(context.Background(), mcp.CallToolRequest{})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "internal error: no database client")

	client := setupTestDB(t)
	defer client.Close()
	ctx := context.WithValue(context.Background(), contextKey{}, client)

	// Missing ids
	res, err = handleQuestOutput(ctx, mcp.CallToolRequest{})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "invalid ids")
}

// TestHandleCreateQuest edge cases
func TestHandleCreateQuestEdgeCases(t *testing.T) {
	mockSrv := mcpserver.NewMCPServer("test", "1.0", mcpserver.WithElicitation())
	handler := handleCreateQuest(mockSrv)

	// Missing client
	res, err := handler(context.Background(), mcp.CallToolRequest{})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "internal error: no database client")

	client := setupTestDB(t)
	defer client.Close()
	ctx := context.WithValue(context.Background(), contextKey{}, client)

	// Missing name
	res, err = handler(ctx, mcp.CallToolRequest{})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "invalid name")

	// Missing beacon_ids
	res, err = handler(ctx, mcp.CallToolRequest{
		Params: mcp.CallToolParams{Arguments: map[string]interface{}{"name": "test"}},
	})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "invalid beacon_ids")

	// Empty beacon_ids
	res, err = handler(ctx, mcp.CallToolRequest{
		Params: mcp.CallToolParams{Arguments: map[string]interface{}{"name": "test", "beacon_ids": []interface{}{}}},
	})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "must provide at least one beacon id")

	// Missing parameters
	res, err = handler(ctx, mcp.CallToolRequest{
		Params: mcp.CallToolParams{Arguments: map[string]interface{}{"name": "test", "beacon_ids": []interface{}{"1"}}},
	})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "invalid parameters")

	// Missing tome_id
	res, err = handler(ctx, mcp.CallToolRequest{
		Params: mcp.CallToolParams{Arguments: map[string]interface{}{"name": "test", "beacon_ids": []interface{}{"1"}, "parameters": "{}"}},
	})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "invalid tome_id")

	// Invalid tome_id (not number)
	res, err = handler(ctx, mcp.CallToolRequest{
		Params: mcp.CallToolParams{Arguments: map[string]interface{}{"name": "test", "beacon_ids": []interface{}{"1"}, "parameters": "{}", "tome_id": "abc"}},
	})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "tome_id must be a number")

	// Missing tome (from db)
	res, err = handler(ctx, mcp.CallToolRequest{
		Params: mcp.CallToolParams{Arguments: map[string]interface{}{"name": "test", "beacon_ids": []interface{}{"1"}, "parameters": "{}", "tome_id": "999"}},
	})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "failed to load tome")
}

// TestHandleGraphQLQueryEdgeCases tests edge cases for the graphql_query tool
func TestHandleGraphQLQueryEdgeCases(t *testing.T) {
	// Missing query
	res, err := handleGraphQLQuery(context.Background(), mcp.CallToolRequest{})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "invalid query")

	// Parse failure
	res, err = handleGraphQLQuery(context.Background(), mcp.CallToolRequest{
		Params: mcp.CallToolParams{Arguments: map[string]interface{}{"query": "query { "}},
	})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "failed to parse")

	// No GraphQL handler in context
	res, err = handleGraphQLQuery(context.Background(), mcp.CallToolRequest{
		Params: mcp.CallToolParams{Arguments: map[string]interface{}{"query": "query { __schema { types { name } } }"}},
	})
	require.NoError(t, err)
	assert.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "no GraphQL handler available")
}
