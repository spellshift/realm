// Package mcp provides a Model Context Protocol (MCP) server for Tavern.
// It exposes Tavern operations as MCP tools accessible via Streamable HTTP transport.
package mcp

import (
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"strconv"
	"time"

	"github.com/mark3labs/mcp-go/mcp"
	mcpserver "github.com/mark3labs/mcp-go/server"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/quest"
	"realm.pub/tavern/internal/ent/tag"
)

// contextKey is used for storing the ent client in the context.
type contextKey struct{}

// clientFromContext returns the per-request ent client from the context.
func clientFromContext(ctx context.Context) *ent.Client {
	if c, ok := ctx.Value(contextKey{}).(*ent.Client); ok {
		return c
	}
	return nil
}

// NewHandler creates an http.Handler that serves the MCP Streamable HTTP endpoints.
// The handler is meant to be mounted at a prefix (e.g. /mcp) on the main server mux.
// Authentication is handled by the parent Tavern HTTP server; this handler
// expects an authenticated context to already be set.
func NewHandler(client *ent.Client) http.Handler {
	mcpSrv := mcpserver.NewMCPServer(
		"tavern",
		"1.0.0",
		mcpserver.WithToolCapabilities(true),
		mcpserver.WithElicitation(),
	)

	mcpSrv.AddTools(
		listQuestsTool(),
		questOutputTool(),
		listTomesTool(),
		createQuestTool(mcpSrv),
		listHostsTool(),
		waitForQuestTool(),
	)

	httpSrv := mcpserver.NewStreamableHTTPServer(mcpSrv,
		mcpserver.WithEndpointPath("/mcp"),
		mcpserver.WithHTTPContextFunc(func(ctx context.Context, r *http.Request) context.Context {
			return context.WithValue(ctx, contextKey{}, client)
		}),
	)

	return httpSrv
}

// listQuestsTool returns the list_quests MCP tool.
func listQuestsTool() mcpserver.ServerTool {
	return mcpserver.ServerTool{
		Tool: mcp.NewTool("list_quests",
			mcp.WithDescription("List all available quests in Tavern"),
			mcp.WithReadOnlyHintAnnotation(true),
			mcp.WithDestructiveHintAnnotation(false),
		),
		Handler: handleListQuests,
	}
}

func handleListQuests(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
	client := clientFromContext(ctx)
	if client == nil {
		return mcp.NewToolResultError("internal error: no database client"), nil
	}

	quests, err := client.Quest.Query().
		WithTome().
		WithCreator().
		All(ctx)
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to list quests: %v", err)), nil
	}

	type questResult struct {
		ID         int    `json:"id"`
		CreatedAt  string `json:"createdAt"`
		Name       string `json:"name"`
		Creator    string `json:"creator,omitempty"`
		Parameters string `json:"parameters"`
		TomeName   string `json:"tomeName,omitempty"`
		TomeTactic string `json:"tomeTactic,omitempty"`
		TomeDesc   string `json:"tomeDescription,omitempty"`
	}

	results := make([]questResult, 0, len(quests))
	for _, q := range quests {
		r := questResult{
			ID:         q.ID,
			CreatedAt:  q.CreatedAt.Format(time.RFC3339),
			Name:       q.Name,
			Parameters: q.Parameters,
		}
		if q.Edges.Creator != nil {
			r.Creator = q.Edges.Creator.Name
		}
		if q.Edges.Tome != nil {
			r.TomeName = q.Edges.Tome.Name
			r.TomeTactic = string(q.Edges.Tome.Tactic)
			r.TomeDesc = q.Edges.Tome.Description
		}
		results = append(results, r)
	}

	data, err := json.Marshal(results)
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to marshal results: %v", err)), nil
	}
	return mcp.NewToolResultText(string(data)), nil
}

// questOutputTool returns the quest_output MCP tool.
func questOutputTool() mcpserver.ServerTool {
	return mcpserver.ServerTool{
		Tool: mcp.NewTool("quest_output",
			mcp.WithDescription("Get the output of quests by their IDs"),
			mcp.WithReadOnlyHintAnnotation(true),
			mcp.WithDestructiveHintAnnotation(false),
			mcp.WithArray("ids",
				mcp.Required(),
				mcp.Description("List of quest IDs"),
				mcp.WithStringItems(),
			),
		),
		Handler: handleQuestOutput,
	}
}

func handleQuestOutput(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
	client := clientFromContext(ctx)
	if client == nil {
		return mcp.NewToolResultError("internal error: no database client"), nil
	}

	ids, err := ParseIntIDs(request, "ids")
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("invalid ids: %v", err)), nil
	}

	quests, err := client.Quest.Query().
		Where(quest.IDIn(ids...)).
		WithTasks(func(q *ent.TaskQuery) {
			q.WithBeacon(func(bq *ent.BeaconQuery) {
				bq.WithHost()
			})
		}).
		All(ctx)
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to query quests: %v", err)), nil
	}

	type beaconInfo struct {
		ID       int    `json:"id"`
		Name     string `json:"name"`
		HostID   int    `json:"hostId,omitempty"`
		HostName string `json:"hostName,omitempty"`
	}
	type taskOutput struct {
		Output string     `json:"output"`
		Beacon beaconInfo `json:"beacon"`
	}
	type questInfo struct {
		ID    int          `json:"id"`
		Name  string       `json:"name"`
		Tasks []taskOutput `json:"tasks"`
	}

	results := make([]questInfo, 0, len(quests))
	for _, q := range quests {
		qi := questInfo{ID: q.ID, Name: q.Name}
		for _, t := range q.Edges.Tasks {
			to := taskOutput{Output: t.Output}
			if t.Edges.Beacon != nil {
				to.Beacon.ID = t.Edges.Beacon.ID
				to.Beacon.Name = t.Edges.Beacon.Name
				if t.Edges.Beacon.Edges.Host != nil {
					to.Beacon.HostID = t.Edges.Beacon.Edges.Host.ID
					to.Beacon.HostName = t.Edges.Beacon.Edges.Host.Name
				}
			}
			qi.Tasks = append(qi.Tasks, to)
		}
		results = append(results, qi)
	}

	data, err := json.Marshal(results)
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to marshal results: %v", err)), nil
	}
	return mcp.NewToolResultText(string(data)), nil
}

// listTomesTool returns the list_tomes MCP tool.
func listTomesTool() mcpserver.ServerTool {
	return mcpserver.ServerTool{
		Tool: mcp.NewTool("list_tomes",
			mcp.WithDescription("List all available tomes in Tavern and their required parameters"),
			mcp.WithReadOnlyHintAnnotation(true),
			mcp.WithDestructiveHintAnnotation(false),
		),
		Handler: handleListTomes,
	}
}

func handleListTomes(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
	client := clientFromContext(ctx)
	if client == nil {
		return mcp.NewToolResultError("internal error: no database client"), nil
	}

	tomes, err := client.Tome.Query().All(ctx)
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to list tomes: %v", err)), nil
	}

	type tomeResult struct {
		ID          int    `json:"id"`
		Name        string `json:"name"`
		Tactic      string `json:"tactic"`
		Description string `json:"description"`
		ParamDefs   string `json:"paramDefs"`
	}

	results := make([]tomeResult, 0, len(tomes))
	for _, t := range tomes {
		results = append(results, tomeResult{
			ID:          t.ID,
			Name:        t.Name,
			Tactic:      string(t.Tactic),
			Description: t.Description,
			ParamDefs:   t.ParamDefs,
		})
	}

	data, err := json.Marshal(results)
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to marshal results: %v", err)), nil
	}
	return mcp.NewToolResultText(string(data)), nil
}

// createQuestTool returns the create_quest MCP tool.
// It accepts the MCPServer to enable elicitation (human-in-the-loop confirmation).
func createQuestTool(mcpSrv *mcpserver.MCPServer) mcpserver.ServerTool {
	return mcpserver.ServerTool{
		Tool: mcp.NewTool("create_quest",
			mcp.WithDescription("Create a new quest in Tavern"),
			mcp.WithReadOnlyHintAnnotation(false),
			mcp.WithDestructiveHintAnnotation(true),
			mcp.WithString("name",
				mcp.Required(),
				mcp.Description("The name of the quest"),
			),
			mcp.WithArray("beacon_ids",
				mcp.Required(),
				mcp.Description("List of beacon IDs to target"),
				mcp.WithStringItems(),
			),
			mcp.WithString("parameters",
				mcp.Required(),
				mcp.Description("Quest parameters as a JSON string"),
			),
			mcp.WithString("tome_id",
				mcp.Required(),
				mcp.Description("The ID of the tome to use for this quest"),
			),
		),
		Handler: handleCreateQuest(mcpSrv),
	}
}

// confirmSchema is the JSON Schema used to request a boolean confirmation from the user.
var confirmSchema = map[string]any{
	"type": "object",
	"properties": map[string]any{
		"confirm": map[string]any{
			"type":        "boolean",
			"description": "Set to true to confirm quest creation",
		},
	},
	"required": []string{"confirm"},
}

// handleCreateQuest returns a tool handler that creates a quest after eliciting user confirmation.
func handleCreateQuest(mcpSrv *mcpserver.MCPServer) mcpserver.ToolHandlerFunc {
	return func(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		client := clientFromContext(ctx)
		if client == nil {
			return mcp.NewToolResultError("internal error: no database client"), nil
		}

		name, err := request.RequireString("name")
		if err != nil {
			return mcp.NewToolResultError(fmt.Sprintf("invalid name: %v", err)), nil
		}

		beaconIDs, err := ParseIntIDs(request, "beacon_ids")
		if err != nil {
			return mcp.NewToolResultError(fmt.Sprintf("invalid beacon_ids: %v", err)), nil
		}
		if len(beaconIDs) == 0 {
			return mcp.NewToolResultError("must provide at least one beacon id"), nil
		}

		params, err := request.RequireString("parameters")
		if err != nil {
			return mcp.NewToolResultError(fmt.Sprintf("invalid parameters: %v", err)), nil
		}

		tomeIDStr, err := request.RequireString("tome_id")
		if err != nil {
			return mcp.NewToolResultError(fmt.Sprintf("invalid tome_id: %v", err)), nil
		}
		tomeID, err := strconv.Atoi(tomeIDStr)
		if err != nil {
			return mcp.NewToolResultError(fmt.Sprintf("tome_id must be a number: %v", err)), nil
		}

		// Load the tome to get eldritch and param defs
		questTome, err := client.Tome.Get(ctx, tomeID)
		if err != nil {
			return mcp.NewToolResultError(fmt.Sprintf("failed to load tome: %v", err)), nil
		}

		// Request human-in-the-loop confirmation before creating the quest.
		// If elicitation is not supported by the client session, proceed without it.
		elicitResult, err := mcpSrv.RequestElicitation(ctx, mcp.ElicitationRequest{
			Params: mcp.ElicitationParams{
				Message:         fmt.Sprintf("Create quest %q using tome %q targeting %d beacon(s)?", name, questTome.Name, len(beaconIDs)),
				RequestedSchema: confirmSchema,
			},
		})
		if err == nil {
			// Elicitation was supported; check the user's response.
			if elicitResult.Action != mcp.ElicitationResponseActionAccept {
				return mcp.NewToolResultError("quest creation cancelled by user"), nil
			}
			// Verify the user explicitly confirmed.
			if content, ok := elicitResult.Content.(map[string]any); ok {
				if confirmed, _ := content["confirm"].(bool); !confirmed {
					return mcp.NewToolResultError("quest creation cancelled by user"), nil
				}
			}
		} else if err != mcpserver.ErrElicitationNotSupported && err != mcpserver.ErrNoActiveSession {
			return mcp.NewToolResultError(fmt.Sprintf("elicitation failed: %v", err)), nil
		}
		// If elicitation is not supported, proceed without confirmation.

		// Get creator from context (if available)
		var creatorID *int
		if creator := auth.UserFromContext(ctx); creator != nil {
			creatorID = &creator.ID
		}

		// Create the quest
		questCreate := client.Quest.Create().
			SetName(name).
			SetParameters(params).
			SetTomeID(tomeID).
			SetParamDefsAtCreation(questTome.ParamDefs).
			SetEldritchAtCreation(questTome.Eldritch)

		if creatorID != nil {
			questCreate = questCreate.SetCreatorID(*creatorID)
		}

		newQuest, err := questCreate.Save(ctx)
		if err != nil {
			return mcp.NewToolResultError(fmt.Sprintf("failed to create quest: %v", err)), nil
		}

		// Create tasks for each beacon
		for _, beaconID := range beaconIDs {
			_, err := client.Task.Create().
				SetQuestID(newQuest.ID).
				SetBeaconID(beaconID).
				Save(ctx)
			if err != nil {
				slog.ErrorContext(ctx, "failed to create task for beacon", "beacon_id", beaconID, "err", err)
			}
		}

		result := map[string]any{
			"id":   newQuest.ID,
			"name": newQuest.Name,
		}
		data, err := json.Marshal(result)
		if err != nil {
			return mcp.NewToolResultError(fmt.Sprintf("failed to marshal result: %v", err)), nil
		}
		return mcp.NewToolResultText(string(data)), nil
	}
}

// listHostsTool returns the list_hosts MCP tool.
func listHostsTool() mcpserver.ServerTool {
	return mcpserver.ServerTool{
		Tool: mcp.NewTool("list_hosts",
			mcp.WithDescription("List all available hosts in Tavern"),
			mcp.WithReadOnlyHintAnnotation(true),
			mcp.WithDestructiveHintAnnotation(false),
		),
		Handler: handleListHosts,
	}
}

func handleListHosts(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
	client := clientFromContext(ctx)
	if client == nil {
		return mcp.NewToolResultError("internal error: no database client"), nil
	}

	hosts, err := client.Host.Query().
		WithTags(func(q *ent.TagQuery) {
			q.Order(ent.Asc(tag.FieldID))
		}).
		WithBeacons().
		All(ctx)
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to list hosts: %v", err)), nil
	}

	type tagInfo struct {
		ID   int    `json:"id"`
		Name string `json:"name"`
		Kind string `json:"kind"`
	}
	type beaconInfo struct {
		ID        int    `json:"id"`
		Name      string `json:"name"`
		Principal string `json:"principal"`
	}
	type hostResult struct {
		ID         int          `json:"id"`
		Identifier string       `json:"identifier"`
		Name       string       `json:"name"`
		Platform   string       `json:"platform"`
		PrimaryIP  string       `json:"primaryIP"`
		ExternalIP string       `json:"externalIP"`
		LastSeenAt string       `json:"lastSeenAt"`
		Tags       []tagInfo    `json:"tags"`
		Beacons    []beaconInfo `json:"beacons"`
	}

	results := make([]hostResult, 0, len(hosts))
	for _, h := range hosts {
		hr := hostResult{
			ID:         h.ID,
			Identifier: h.Identifier,
			Name:       h.Name,
			Platform:   h.Platform.String(),
			PrimaryIP:  h.PrimaryIP,
			ExternalIP: h.ExternalIP,
			LastSeenAt: h.LastSeenAt.Format(time.RFC3339),
		}
		for _, t := range h.Edges.Tags {
			hr.Tags = append(hr.Tags, tagInfo{
				ID:   t.ID,
				Name: t.Name,
				Kind: string(t.Kind),
			})
		}
		for _, b := range h.Edges.Beacons {
			hr.Beacons = append(hr.Beacons, beaconInfo{
				ID:        b.ID,
				Name:      b.Name,
				Principal: b.Principal,
			})
		}
		results = append(results, hr)
	}

	data, err := json.Marshal(results)
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to marshal results: %v", err)), nil
	}
	return mcp.NewToolResultText(string(data)), nil
}

// waitForQuestTool returns the wait_for_quest MCP tool.
func waitForQuestTool() mcpserver.ServerTool {
	return mcpserver.ServerTool{
		Tool: mcp.NewTool("wait_for_quest",
			mcp.WithDescription("Wait for all tasks in a quest to finish. Polls every 5 seconds for up to 10 minutes."),
			mcp.WithReadOnlyHintAnnotation(true),
			mcp.WithDestructiveHintAnnotation(false),
			mcp.WithString("quest_id",
				mcp.Required(),
				mcp.Description("The ID of the quest to wait for"),
			),
		),
		Handler: handleWaitForQuest,
	}
}

func handleWaitForQuest(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
	client := clientFromContext(ctx)
	if client == nil {
		return mcp.NewToolResultError("internal error: no database client"), nil
	}

	questIDStr, err := request.RequireString("quest_id")
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("invalid quest_id: %v", err)), nil
	}
	questID, err := strconv.Atoi(questIDStr)
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("quest_id must be a number: %v", err)), nil
	}

	const (
		pollInterval = 5 * time.Second
		timeout      = 10 * time.Minute
	)
	deadline := time.Now().Add(timeout)
	var pendingCount int

	for time.Now().Before(deadline) {
		q, err := client.Quest.Query().
			Where(quest.ID(questID)).
			WithTasks(func(tq *ent.TaskQuery) {
				tq.WithBeacon()
			}).
			Only(ctx)
		if err != nil {
			if ent.IsNotFound(err) {
				return mcp.NewToolResultError(fmt.Sprintf("quest with ID %d not found", questID)), nil
			}
			return mcp.NewToolResultError(fmt.Sprintf("failed to query quest: %v", err)), nil
		}

		tasks := q.Edges.Tasks
		if len(tasks) == 0 {
			return mcp.NewToolResultError(fmt.Sprintf("quest %d has no tasks", questID)), nil
		}

		allFinished := true
		pendingCount = 0
		for _, t := range tasks {
			if t.ExecFinishedAt.IsZero() {
				allFinished = false
				pendingCount++
			}
		}

		if allFinished {
			type taskDetail struct {
				TaskID     int    `json:"task_id"`
				BeaconID   int    `json:"beacon_id,omitempty"`
				BeaconName string `json:"beacon_name,omitempty"`
				FinishedAt string `json:"finished_at"`
				Error      string `json:"error,omitempty"`
			}
			details := make([]taskDetail, 0, len(tasks))
			for _, t := range tasks {
				td := taskDetail{
					TaskID:     t.ID,
					FinishedAt: t.ExecFinishedAt.Format(time.RFC3339),
					Error:      t.Error,
				}
				if t.Edges.Beacon != nil {
					td.BeaconID = t.Edges.Beacon.ID
					td.BeaconName = t.Edges.Beacon.Name
				}
				details = append(details, td)
			}

			result := map[string]any{
				"message": fmt.Sprintf("All %d tasks in quest '%s' have finished.", len(tasks), q.Name),
				"tasks":   details,
			}
			data, err := json.Marshal(result)
			if err != nil {
				return mcp.NewToolResultError(fmt.Sprintf("failed to marshal result: %v", err)), nil
			}
			return mcp.NewToolResultText(string(data)), nil
		}

		select {
		case <-ctx.Done():
			return mcp.NewToolResultError("request cancelled"), nil
		case <-time.After(pollInterval):
		}
	}

	return mcp.NewToolResultError(fmt.Sprintf("timeout: not all tasks in quest %d finished within 10 minutes. %d tasks still pending", questID, pendingCount)), nil
}

// ParseIntIDs extracts an array of string IDs from the request arguments and converts them to ints.
func ParseIntIDs(request mcp.CallToolRequest, key string) ([]int, error) {
	args := request.GetArguments()
	raw, ok := args[key]
	if !ok {
		return nil, fmt.Errorf("required argument %q not found", key)
	}
	arr, ok := raw.([]any)
	if !ok {
		return nil, fmt.Errorf("argument %q must be an array", key)
	}
	ids := make([]int, 0, len(arr))
	for _, v := range arr {
		s, ok := v.(string)
		if !ok {
			// Also handle numeric values from JSON
			if n, ok := v.(float64); ok {
				ids = append(ids, int(n))
				continue
			}
			return nil, fmt.Errorf("array element must be a string, got %T", v)
		}
		id, err := strconv.Atoi(s)
		if err != nil {
			return nil, fmt.Errorf("invalid id %q: %v", s, err)
		}
		ids = append(ids, id)
	}
	return ids, nil
}
