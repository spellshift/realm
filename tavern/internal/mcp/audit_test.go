package mcp_test

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strconv"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	mcpclient "github.com/mark3labs/mcp-go/client"
	"github.com/mark3labs/mcp-go/client/transport"
	"github.com/mark3labs/mcp-go/mcp"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/tome"
	"realm.pub/tavern/internal/logging"
	tavernmcp "realm.pub/tavern/internal/mcp"
)

type testElicitationHandler struct {
	fn atomic.Value // holds func(context.Context, mcp.ElicitationRequest) (*mcp.ElicitationResult, error)
}

func (h *testElicitationHandler) set(fn func(context.Context, mcp.ElicitationRequest) (*mcp.ElicitationResult, error)) {
	h.fn.Store(fn)
}

func (h *testElicitationHandler) Elicit(ctx context.Context, req mcp.ElicitationRequest) (*mcp.ElicitationResult, error) {
	if fn, ok := h.fn.Load().(func(context.Context, mcp.ElicitationRequest) (*mcp.ElicitationResult, error)); ok && fn != nil {
		return fn(ctx, req)
	}
	return &mcp.ElicitationResult{
		ElicitationResponse: mcp.ElicitationResponse{
			Action:  mcp.ElicitationResponseActionAccept,
			Content: map[string]any{"confirm": true},
		},
	}, nil
}

type mcpAuditRecorder struct {
	mu      sync.Mutex
	entries []logging.AuditEntry
}

func (r *mcpAuditRecorder) Enqueue(entry logging.AuditEntry) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.entries = append(r.entries, entry)
}

func (r *mcpAuditRecorder) findOne(t *testing.T, predicate func(logging.AuditEntry) bool) logging.AuditEntry {
	t.Helper()
	r.mu.Lock()
	defer r.mu.Unlock()
	var matches []logging.AuditEntry
	for _, entry := range r.entries {
		if predicate(entry) {
			matches = append(matches, entry)
		}
	}
	require.Len(t, matches, 1)
	return matches[0]
}

func marshalMCPDetails(t *testing.T, raw map[string]any) (string, map[string]any) {
	t.Helper()
	b, err := json.Marshal(raw)
	require.NoError(t, err)
	var out map[string]any
	require.NoError(t, json.Unmarshal(b, &out))
	return string(b), out
}

func TestMCPAuditLogging(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
	defer cancel()

	dbClient := setupTestDB(t)
	defer dbClient.Close()

	rec := &mcpAuditRecorder{}
	logging.SetAuditPersister(rec)
	t.Cleanup(func() {
		logging.SetAuditPersister(nil)
	})

	mcpUser := dbClient.User.Create().
		SetName("mcp-operator").
		SetOauthID("oauth-mcp-op").
		SetPhotoURL("https://example.com/mcp.png").
		SetSessionToken("mcp-session").
		SetAccessToken("mcp-token").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	testTome := dbClient.Tome.Create().
		SetName("MCP Audit Tome").
		SetDescription("Tome for MCP audit testing").
		SetAuthor("tester").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetTactic(tome.TacticRECON).
		SetEldritch(`print("mcp")`).
		SetHash("mcpaudithash").
		SaveX(ctx)

	testHost := dbClient.Host.Create().
		SetIdentifier("mcp-audit-host").
		SetName("target-box").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)

	testBeacon := dbClient.Beacon.Create().
		SetHost(testHost).
		SetName("mcp-beacon-1").
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
		SaveX(ctx)

	rawHandler := tavernmcp.NewHandler(dbClient, "test-v1", nil)
	authedHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		r = r.WithContext(auth.ContextFromUser(r.Context(), mcpUser))
		rawHandler.ServeHTTP(w, r)
	})

	srv := httptest.NewServer(authedHandler)
	defer srv.Close()

	elicitHandler := &testElicitationHandler{}
	tr, err := transport.NewStreamableHTTP(srv.URL+"/mcp", transport.WithContinuousListening())
	require.NoError(t, err)

	cli := mcpclient.NewClient(tr, mcpclient.WithElicitationHandler(elicitHandler))
	require.NoError(t, cli.Start(ctx))
	defer cli.Close()

	initReq := mcp.InitializeRequest{}
	initReq.Params.ProtocolVersion = mcp.LATEST_PROTOCOL_VERSION
	initReq.Params.ClientInfo = mcp.Implementation{
		Name:    "mcp-audit-test-client",
		Version: "1.0.0",
	}
	initReq.Params.Capabilities = mcp.ClientCapabilities{
		Elicitation: &mcp.ElicitationCapability{},
	}
	_, err = cli.Initialize(ctx, initReq)
	require.NoError(t, err)

	t.Run("ToolMiddlewareEmitsAuditMCP", func(t *testing.T) {
		callReq := mcp.CallToolRequest{}
		callReq.Params.Name = "list_tomes"
		callReq.Params.Arguments = map[string]any{}

		res, err := cli.CallTool(ctx, callReq)
		require.NoError(t, err)
		require.False(t, res.IsError)

		entry := rec.findOne(t, func(e logging.AuditEntry) bool {
			return e.Subsystem == "audit.mcp" && e.Action == "mcp.tool_called" && e.TargetName == "list_tomes"
		})

		assert.Equal(t, logging.OutcomeSuccess, entry.Outcome)
		assert.Equal(t, "mcp", entry.Entrypoint)
		assert.Equal(t, logging.ActorUser, entry.Actor.Kind)
		assert.Equal(t, strconv.Itoa(mcpUser.ID), entry.Actor.ID)
		assert.Equal(t, mcpUser.Name, entry.Actor.Name)
		assert.Equal(t, "mcp_tool", entry.TargetType)

		_, details := marshalMCPDetails(t, entry.Details)
		assert.Equal(t, "list_tomes", details["tool_name"])
		_, hasDuration := details["duration_ms"]
		assert.True(t, hasDuration)
	})

	t.Run("CreateQuestConfirmedEmitsAuditQuestAndMCPWithScrubbing", func(t *testing.T) {
		elicitHandler.set(func(_ context.Context, req mcp.ElicitationRequest) (*mcp.ElicitationResult, error) {
			assert.Contains(t, req.Params.Message, "MCPConfirmedQuest")
			return &mcp.ElicitationResult{
				ElicitationResponse: mcp.ElicitationResponse{
					Action:  mcp.ElicitationResponseActionAccept,
					Content: map[string]any{"confirm": true},
				},
			}, nil
		})

		secretPW := "mcp-cleartext-password-999"
		secretTok := "mcp-bearer-secret-777"
		callReq := mcp.CallToolRequest{}
		callReq.Params.Name = "create_quest"
		callReq.Params.Arguments = map[string]any{
			"name":       "MCPConfirmedQuest",
			"beacon_ids": []any{strconv.Itoa(testBeacon.ID)},
			"parameters": `{"password":"` + secretPW + `","access_token":"` + secretTok + `","cmd":"whoami"}`,
			"tome_id":    strconv.Itoa(testTome.ID),
		}

		res, err := cli.CallTool(ctx, callReq)
		require.NoError(t, err)
		require.False(t, res.IsError, "expected create_quest to succeed")

		require.Equal(t, 1, dbClient.Quest.Query().CountX(ctx))
		createdQuest := dbClient.Quest.Query().OnlyX(ctx)

		questEntry := rec.findOne(t, func(e logging.AuditEntry) bool {
			return e.Subsystem == "audit.quest" && e.Action == "quest.created" && e.Outcome == logging.OutcomeSuccess
		})

		assert.Equal(t, "quest created via MCP", questEntry.Summary)
		assert.Equal(t, "mcp", questEntry.Entrypoint)
		assert.Equal(t, strconv.Itoa(mcpUser.ID), questEntry.Actor.ID)
		assert.Equal(t, "quest", questEntry.TargetType)
		assert.Equal(t, strconv.Itoa(createdQuest.ID), questEntry.TargetID)
		assert.Equal(t, "MCPConfirmedQuest", questEntry.TargetName)

		qDetailsJSON, qDetails := marshalMCPDetails(t, questEntry.Details)
		assert.NotContains(t, qDetailsJSON, secretPW)
		assert.NotContains(t, qDetailsJSON, secretTok)
		assert.Equal(t, true, qDetails["elicitation_confirmed"])
		assert.Equal(t, float64(1), qDetails["task_count"])
		assert.Equal(t, float64(testTome.ID), qDetails["tome_id"])
		assert.Equal(t, []any{float64(testBeacon.ID)}, qDetails["beacon_ids"])
		assert.Equal(t, []any{float64(testHost.ID)}, qDetails["host_ids"])

		paramsObj, ok := qDetails["parameters"].(map[string]any)
		require.True(t, ok)
		assert.Equal(t, logging.RedactedPlaceholder, paramsObj["password"])
		assert.Equal(t, logging.RedactedPlaceholder, paramsObj["access_token"])
		assert.Equal(t, "whoami", paramsObj["cmd"])

		mcpEntry := rec.findOne(t, func(e logging.AuditEntry) bool {
			return e.Subsystem == "audit.mcp" &&
				e.Action == "mcp.tool_called" &&
				e.TargetName == "create_quest" &&
				e.Outcome == logging.OutcomeSuccess
		})
		mcpDetailsJSON, _ := marshalMCPDetails(t, mcpEntry.Details)
		assert.NotContains(t, mcpDetailsJSON, secretPW)
		assert.NotContains(t, mcpDetailsJSON, secretTok)
	})

	t.Run("CreateQuestDeclinedEmitsDeniedAudit", func(t *testing.T) {
		elicitHandler.set(func(_ context.Context, _ mcp.ElicitationRequest) (*mcp.ElicitationResult, error) {
			return &mcp.ElicitationResult{
				ElicitationResponse: mcp.ElicitationResponse{
					Action: mcp.ElicitationResponseActionDecline,
				},
			}, nil
		})

		declinedSecret := "declined-quest-secret-123"
		callReq := mcp.CallToolRequest{}
		callReq.Params.Name = "create_quest"
		callReq.Params.Arguments = map[string]any{
			"name":       "MCPDeclinedQuest",
			"beacon_ids": []any{strconv.Itoa(testBeacon.ID)},
			"parameters": `{"api_key":"` + declinedSecret + `"}`,
			"tome_id":    strconv.Itoa(testTome.ID),
		}

		res, err := cli.CallTool(ctx, callReq)
		require.NoError(t, err)
		require.True(t, res.IsError, "expected create_quest to return tool error when declined")

		// Quest count should still be 1 (the earlier confirmed quest only)
		assert.Equal(t, 1, dbClient.Quest.Query().CountX(ctx))

		deniedQuestEntry := rec.findOne(t, func(e logging.AuditEntry) bool {
			return e.Subsystem == "audit.quest" &&
				e.Action == "quest.created" &&
				e.Outcome == logging.OutcomeDenied
		})
		dDetailsJSON, dDetails := marshalMCPDetails(t, deniedQuestEntry.Details)
		assert.NotContains(t, dDetailsJSON, declinedSecret)
		assert.Equal(t, false, dDetails["elicitation_confirmed"])
		assert.Equal(t, string(mcp.ElicitationResponseActionDecline), dDetails["elicitation_action"])

		deniedMCPEntry := rec.findOne(t, func(e logging.AuditEntry) bool {
			return e.Subsystem == "audit.mcp" &&
				e.Action == "mcp.tool_called" &&
				e.TargetName == "create_quest" &&
				e.Outcome == logging.OutcomeDenied
		})
		deniedMCPJSON, _ := marshalMCPDetails(t, deniedMCPEntry.Details)
		assert.NotContains(t, deniedMCPJSON, declinedSecret)
	})

	t.Run("GraphQLQueryMutationDenied", func(t *testing.T) {
		callReq := mcp.CallToolRequest{}
		callReq.Params.Name = "graphql_query"
		callReq.Params.Arguments = map[string]any{
			"query": "mutation { dropAllData }",
		}

		res, err := cli.CallTool(ctx, callReq)
		require.NoError(t, err)
		require.True(t, res.IsError)

		entry := rec.findOne(t, func(e logging.AuditEntry) bool {
			return e.Subsystem == "audit.mcp" &&
				e.Action == "mcp.tool_called" &&
				e.TargetName == "graphql_query" &&
				e.Outcome == logging.OutcomeDenied
		})
		detailsJSON, _ := marshalMCPDetails(t, entry.Details)
		assert.Contains(t, detailsJSON, "only queries are allowed")
	})
}
