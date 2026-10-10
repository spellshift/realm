package mcp

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/mark3labs/mcp-go/mcp"
	mcpserver "github.com/mark3labs/mcp-go/server"
)

// listTomesTool returns the list_tomes MCP tool.
func listTomesTool() mcpserver.ServerTool {
	return mcpserver.ServerTool{
		Tool: mcp.NewTool("list_tomes",
			mcp.WithDescription("List all available tomes in Tavern and their required parameters. Always call this before create_quest to obtain the correct numeric tome ID — IDs are large Ent global IDs (e.g. 107374182403), not sequential integers starting at 1. When the result is large it may be written to a file path; read that file to access the data."),
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
