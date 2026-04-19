package mcp

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/mark3labs/mcp-go/mcp"
	mcpserver "github.com/mark3labs/mcp-go/server"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/quest"
)

// questOutputTool returns the quest_output MCP tool.
func questOutputTool() mcpserver.ServerTool {
	return mcpserver.ServerTool{
		Tool: mcp.NewTool("quest_output",
			mcp.WithDescription("Get the output of quests by their IDs. The 'ids' parameter takes quest IDs (from create_quest), not task IDs returned by wait_for_quest."),
			mcp.WithReadOnlyHintAnnotation(true),
			mcp.WithDestructiveHintAnnotation(false),
			mcp.WithArray("ids",
				mcp.Required(),
				mcp.Description("List of quest IDs (from create_quest) — not task IDs"),
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
