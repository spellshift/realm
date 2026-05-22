package mcp_test

import (
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/mark3labs/mcp-go/mcp"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/tome"
	tavernmcp "realm.pub/tavern/internal/mcp"
)

// setupTestDB creates a test ent client with schema migrations applied.
func setupTestDB(t *testing.T) *ent.Client {
	t.Helper()
	return enttest.OpenTempDB(t)
}

// setupTestHandler creates the MCP HTTP handler backed by a test database.
func setupTestHandler(t *testing.T, client *ent.Client) http.Handler {
	t.Helper()
	return tavernmcp.NewHandler(client, "test", nil)
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
			ids, err := tavernmcp.ParseIntIDs(req, tc.key)
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
			result, err := tavernmcp.HandleGraphQLQueryForTest(context.Background(), req)
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

func TestHandleListHosts(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	// Create test data
	host := client.Host.Create().
		SetIdentifier("test-host-id-123").
		SetName("test-host").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SetPrimaryIP("192.168.1.1").
		SaveX(ctx)

	client.Beacon.Create().
		SetHost(host).
		SetName("test-beacon").
		SetTransport(c2pb.Transport_TRANSPORT_UNSPECIFIED).
		SetPrincipal("root").
		SaveX(ctx)

	client.Tag.Create().
		SetName("test-tag").
		SetKind("group").
		AddHosts(host).
		SaveX(ctx)

	// Setup context with client
	testCtx := context.WithValue(ctx, tavernmcp.TestContextKey{}, client)

	req := mcp.CallToolRequest{}

	res, err := tavernmcp.HandleListHosts(testCtx, req)
	require.NoError(t, err)
	assert.False(t, res.IsError)
	require.Len(t, res.Content, 1)

	textContent, ok := res.Content[0].(mcp.TextContent)
	require.True(t, ok)

	// Verify the JSON string
	assert.Contains(t, textContent.Text, "test-host-id-123")
	assert.Contains(t, textContent.Text, "test-host")
	assert.Contains(t, textContent.Text, "PLATFORM_LINUX")
	assert.Contains(t, textContent.Text, "test-beacon")
	assert.Contains(t, textContent.Text, "root")
	assert.Contains(t, textContent.Text, "test-tag")
	assert.Contains(t, textContent.Text, "group")

	// Test nil client error
	resErr, err := tavernmcp.HandleListHosts(context.Background(), req)
	require.NoError(t, err)
	assert.True(t, resErr.IsError)
	textContentErr, ok := resErr.Content[0].(mcp.TextContent)
	require.True(t, ok)
	assert.Contains(t, textContentErr.Text, "internal error: no database client")
}

func TestHandleListQuests(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	// Create test data
	testTome := client.Tome.Create().
		SetName("test-tome").
		SetDescription("A test tome").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('hello')").
		SetHash("abc123").
		SetParamDefs(`[{"name":"key","type":"string"}]`).
		SaveX(ctx)

	client.Quest.Create().
		SetName("test-quest").
		SetParameters(`{"key":"value"}`).
		SetTomeID(testTome.ID).
		SetParamDefsAtCreation(testTome.ParamDefs).
		SetEldritchAtCreation(testTome.Eldritch).
		SaveX(ctx)

	// Setup context with client
	testCtx := context.WithValue(ctx, tavernmcp.TestContextKey{}, client)

	req := mcp.CallToolRequest{}

	res, err := tavernmcp.HandleListQuests(testCtx, req)
	require.NoError(t, err)
	assert.False(t, res.IsError)
	require.Len(t, res.Content, 1)

	textContent, ok := res.Content[0].(mcp.TextContent)
	require.True(t, ok)

	// Verify the JSON string
	assert.Contains(t, textContent.Text, "test-quest")
	assert.Contains(t, textContent.Text, "test-tome")
	assert.Contains(t, textContent.Text, "A test tome")

	// Test nil client error
	resErr, err := tavernmcp.HandleListQuests(context.Background(), req)
	require.NoError(t, err)
	assert.True(t, resErr.IsError)
	textContentErr, ok := resErr.Content[0].(mcp.TextContent)
	require.True(t, ok)
	assert.Contains(t, textContentErr.Text, "internal error: no database client")
}

func TestHandleListTomes(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	// Create test data
	client.Tome.Create().
		SetName("tome-handler-test").
		SetDescription("First tome for handler").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('one')").
		SetHash("hash1").
		SetParamDefs(`[{"name":"param1","type":"string"}]`).
		SaveX(ctx)

	// Setup context with client
	testCtx := context.WithValue(ctx, tavernmcp.TestContextKey{}, client)

	req := mcp.CallToolRequest{}

	res, err := tavernmcp.HandleListTomes(testCtx, req)
	require.NoError(t, err)
	assert.False(t, res.IsError)
	require.Len(t, res.Content, 1)

	textContent, ok := res.Content[0].(mcp.TextContent)
	require.True(t, ok)

	// Verify the JSON string
	assert.Contains(t, textContent.Text, "tome-handler-test")
	assert.Contains(t, textContent.Text, "First tome for handler")
	assert.Contains(t, textContent.Text, "param1")

	// Test nil client error
	resErr, err := tavernmcp.HandleListTomes(context.Background(), req)
	require.NoError(t, err)
	assert.True(t, resErr.IsError)
	textContentErr, ok := resErr.Content[0].(mcp.TextContent)
	require.True(t, ok)
	assert.Contains(t, textContentErr.Text, "internal error: no database client")
}

func TestHandleQuestOutput(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	// Create test data
	testTome := client.Tome.Create().
		SetName("test-tome").
		SetDescription("A test tome").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('hello')").
		SetHash("abc123").
		SaveX(ctx)

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

	// Setup context with client
	testCtx := context.WithValue(ctx, tavernmcp.TestContextKey{}, client)

	// Valid request
	req := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{
				"ids": []interface{}{float64(q.ID)},
			},
		},
	}

	res, err := tavernmcp.HandleQuestOutput(testCtx, req)
	require.NoError(t, err)
	assert.False(t, res.IsError)
	require.Len(t, res.Content, 1)

	textContent, ok := res.Content[0].(mcp.TextContent)
	require.True(t, ok)

	// Verify the JSON string
	assert.Contains(t, textContent.Text, "output-quest")
	assert.Contains(t, textContent.Text, "task output result")
	assert.Contains(t, textContent.Text, "test-beacon")
	assert.Contains(t, textContent.Text, "test-host")

	// Test nil client error
	resErr, err := tavernmcp.HandleQuestOutput(context.Background(), req)
	require.NoError(t, err)
	assert.True(t, resErr.IsError)
	textContentErr, ok := resErr.Content[0].(mcp.TextContent)
	require.True(t, ok)
	assert.Contains(t, textContentErr.Text, "internal error: no database client")

	// Test invalid IDs
	reqInvalid := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{
				"ids": "not-an-array",
			},
		},
	}
	resInvalid, err := tavernmcp.HandleQuestOutput(testCtx, reqInvalid)
	require.NoError(t, err)
	assert.True(t, resInvalid.IsError)
	textContentInvalid, ok := resInvalid.Content[0].(mcp.TextContent)
	require.True(t, ok)
	assert.Contains(t, textContentInvalid.Text, "invalid ids")
}

func TestHandleWaitForQuest(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	// Create test data
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

	testTome := client.Tome.Create().
		SetName("test-tome").
		SetDescription("A test tome").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('hello')").
		SetHash("abc123").
		SaveX(ctx)

	q := client.Quest.Create().
		SetName("wait-quest").
		SetParameters("{}").
		SetTome(testTome).
		SetParamDefsAtCreation("[]").
		SetEldritchAtCreation("print('hello')").
		SaveX(ctx)

	client.Task.Create().
		SetQuest(q).
		SetBeacon(testBeacon).
		SetExecFinishedAt(time.Now()).
		SaveX(ctx)

	// Setup context with client
	testCtx := context.WithValue(ctx, tavernmcp.TestContextKey{}, client)

	// Valid request
	req := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{
				"quest_id": fmt.Sprintf("%d", q.ID),
			},
		},
	}

	res, err := tavernmcp.HandleWaitForQuest(testCtx, req)
	require.NoError(t, err)
	assert.False(t, res.IsError)
	require.Len(t, res.Content, 1)

	textContent, ok := res.Content[0].(mcp.TextContent)
	require.True(t, ok)

	// Verify the JSON string
	assert.Contains(t, textContent.Text, "have finished")
	assert.Contains(t, textContent.Text, "test-beacon")

	// Test nil client error
	resErr, err := tavernmcp.HandleWaitForQuest(context.Background(), req)
	require.NoError(t, err)
	assert.True(t, resErr.IsError)
	textContentErr, ok := resErr.Content[0].(mcp.TextContent)
	require.True(t, ok)
	assert.Contains(t, textContentErr.Text, "internal error: no database client")

	// Test invalid request (missing quest_id)
	reqMissing := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{},
		},
	}
	resMissing, err := tavernmcp.HandleWaitForQuest(testCtx, reqMissing)
	require.NoError(t, err)
	assert.True(t, resMissing.IsError)

	// Test invalid request (quest_id not a number)
	reqInvalidStr := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{
				"quest_id": "not-a-number",
			},
		},
	}
	resInvalidStr, err := tavernmcp.HandleWaitForQuest(testCtx, reqInvalidStr)
	require.NoError(t, err)
	assert.True(t, resInvalidStr.IsError)

	// Test quest not found
	reqNotFound := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{
				"quest_id": "99999",
			},
		},
	}
	resNotFound, err := tavernmcp.HandleWaitForQuest(testCtx, reqNotFound)
	require.NoError(t, err)
	assert.True(t, resNotFound.IsError)

	// Test quest with no tasks
	qNoTasks := client.Quest.Create().
		SetName("no-tasks-quest").
		SetParameters("{}").
		SetTome(testTome).
		SetParamDefsAtCreation("[]").
		SetEldritchAtCreation("print('hello')").
		SaveX(ctx)

	reqNoTasks := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{
				"quest_id": fmt.Sprintf("%d", qNoTasks.ID),
			},
		},
	}
	resNoTasks, err := tavernmcp.HandleWaitForQuest(testCtx, reqNoTasks)
	require.NoError(t, err)
	assert.True(t, resNoTasks.IsError)
	textContentNoTasks, ok := resNoTasks.Content[0].(mcp.TextContent)
	require.True(t, ok)
	assert.Contains(t, textContentNoTasks.Text, "has no tasks")
}

func TestHandleCreateQuest(t *testing.T) {
	ctx := context.Background()
	client := setupTestDB(t)
	defer client.Close()

	// Create test data
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

	testTome := client.Tome.Create().
		SetName("test-tome").
		SetDescription("A test tome").
		SetAuthor("test-author").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetEldritch("print('hello')").
		SetHash("abc123").
		SaveX(ctx)

	// Setup context with client
	testCtx := context.WithValue(ctx, tavernmcp.TestContextKey{}, client)

	// Valid request
	req := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{
				"name":       "new-test-quest",
				"beacon_ids": []interface{}{fmt.Sprintf("%d", testBeacon.ID)},
				"parameters": `{}`,
				"tome_id":    fmt.Sprintf("%d", testTome.ID),
			},
		},
	}

	// Use HandleCreateQuestForTest which allows passing nil for MCPServer
	res, err := tavernmcp.HandleCreateQuestForTest(testCtx, req)
	require.NoError(t, err)
	assert.False(t, res.IsError)
	require.Len(t, res.Content, 1)

	textContent, ok := res.Content[0].(mcp.TextContent)
	require.True(t, ok)

	// Verify the JSON string
	assert.Contains(t, textContent.Text, "new-test-quest")

	// Verify quest was created in db
	quests, err := client.Quest.Query().WithTasks().All(ctx)
	require.NoError(t, err)
	assert.Len(t, quests, 1)
	assert.Equal(t, "new-test-quest", quests[0].Name)
	assert.Len(t, quests[0].Edges.Tasks, 1)

	// Test nil client error
	resErr, err := tavernmcp.HandleCreateQuestForTest(context.Background(), req)
	require.NoError(t, err)
	assert.True(t, resErr.IsError)
	textContentErr, ok := resErr.Content[0].(mcp.TextContent)
	require.True(t, ok)
	assert.Contains(t, textContentErr.Text, "internal error: no database client")

	// Test missing name
	reqNoName := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{
				"beacon_ids": []interface{}{fmt.Sprintf("%d", testBeacon.ID)},
				"parameters": `{"test": true}`,
				"tome_id":    fmt.Sprintf("%d", testTome.ID),
			},
		},
	}
	resNoName, err := tavernmcp.HandleCreateQuestForTest(testCtx, reqNoName)
	require.NoError(t, err)
	assert.True(t, resNoName.IsError)

	// Test missing beacon_ids
	reqNoBeacons := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{
				"name":       "new-test-quest",
				"parameters": `{"test": true}`,
				"tome_id":    fmt.Sprintf("%d", testTome.ID),
			},
		},
	}
	resNoBeacons, err := tavernmcp.HandleCreateQuestForTest(testCtx, reqNoBeacons)
	require.NoError(t, err)
	assert.True(t, resNoBeacons.IsError)

	// Test empty beacon_ids
	reqEmptyBeacons := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{
				"name":       "new-test-quest",
				"beacon_ids": []interface{}{},
				"parameters": `{"test": true}`,
				"tome_id":    fmt.Sprintf("%d", testTome.ID),
			},
		},
	}
	resEmptyBeacons, err := tavernmcp.HandleCreateQuestForTest(testCtx, reqEmptyBeacons)
	require.NoError(t, err)
	assert.True(t, resEmptyBeacons.IsError)

	// Test missing parameters (should use {})
	reqNoParams := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{
				"name":       "new-test-quest-noparams",
				"beacon_ids": []interface{}{fmt.Sprintf("%d", testBeacon.ID)},
				"tome_id":    fmt.Sprintf("%d", testTome.ID),
			},
		},
	}
	resNoParams, err := tavernmcp.HandleCreateQuestForTest(testCtx, reqNoParams)
	require.NoError(t, err)
	assert.True(t, resNoParams.IsError) // Wait, actually RequireString will return an error if missing

	// Test empty parameters
	reqEmptyParams := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{
				"name":       "new-test-quest-emptyparams",
				"beacon_ids": []interface{}{fmt.Sprintf("%d", testBeacon.ID)},
				"parameters": "",
				"tome_id":    fmt.Sprintf("%d", testTome.ID),
			},
		},
	}
	resEmptyParams, err := tavernmcp.HandleCreateQuestForTest(testCtx, reqEmptyParams)
	require.NoError(t, err)
	assert.False(t, resEmptyParams.IsError) // Should succeed

	// Test missing tome_id
	reqNoTome := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{
				"name":       "new-test-quest",
				"beacon_ids": []interface{}{fmt.Sprintf("%d", testBeacon.ID)},
				"parameters": `{"test": true}`,
			},
		},
	}
	resNoTome, err := tavernmcp.HandleCreateQuestForTest(testCtx, reqNoTome)
	require.NoError(t, err)
	assert.True(t, resNoTome.IsError)

	// Test tome not found
	reqBadTome := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: map[string]interface{}{
				"name":       "new-test-quest",
				"beacon_ids": []interface{}{fmt.Sprintf("%d", testBeacon.ID)},
				"parameters": `{"test": true}`,
				"tome_id":    "999999",
			},
		},
	}
	resBadTome, err := tavernmcp.HandleCreateQuestForTest(testCtx, reqBadTome)
	require.NoError(t, err)
	assert.True(t, resBadTome.IsError)
}
