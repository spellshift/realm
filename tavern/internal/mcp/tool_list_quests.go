package mcp

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/mark3labs/mcp-go/mcp"
	mcpserver "github.com/mark3labs/mcp-go/server"
)

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
