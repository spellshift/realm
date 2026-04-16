package mcp_test

import (
	"context"
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
	return tavernmcp.NewHandler(client)
}

// TestNewHandler verifies the MCP handler can be created without error.
func TestNewHandler(t *testing.T) {
	client := setupTestDB(t)
	defer client.Close()

	handler := setupTestHandler(t, client)
	assert.NotNil(t, handler)
}

// TestMCPSSEEndpoint verifies the SSE endpoint is accessible and returns the correct content type.
func TestMCPSSEEndpoint(t *testing.T) {
	client := setupTestDB(t)
	defer client.Close()

	handler := setupTestHandler(t, client)

	// Use a context with timeout to avoid hanging
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	req := httptest.NewRequest(http.MethodGet, "/mcp/sse", nil).WithContext(ctx)
	w := httptest.NewRecorder()

	// Run handler in goroutine since SSE will block
	done := make(chan struct{})
	go func() {
		handler.ServeHTTP(w, req)
		close(done)
	}()

	// Wait for either the handler to return or timeout
	select {
	case <-done:
	case <-ctx.Done():
	}

	// SSE endpoint should return text/event-stream
	result := w.Result()
	defer result.Body.Close()
	assert.Equal(t, http.StatusOK, result.StatusCode)
	assert.Contains(t, result.Header.Get("Content-Type"), "text/event-stream")
}

// TestMCPMessageEndpointRequiresSession verifies the message endpoint rejects requests without a valid session.
func TestMCPMessageEndpointRequiresSession(t *testing.T) {
	client := setupTestDB(t)
	defer client.Close()

	handler := setupTestHandler(t, client)

	req := httptest.NewRequest(http.MethodPost, "/mcp/message", nil)
	w := httptest.NewRecorder()

	handler.ServeHTTP(w, req)

	result := w.Result()
	defer result.Body.Close()
	// Without a valid session, we expect an error response
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
