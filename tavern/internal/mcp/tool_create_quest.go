package mcp

import (
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"strconv"
	"strings"

	"github.com/mark3labs/mcp-go/mcp"
	mcpserver "github.com/mark3labs/mcp-go/server"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent/beacon"
)

// createQuestTool returns the create_quest MCP tool.
// It accepts the MCPServer to enable elicitation (human-in-the-loop confirmation).
func createQuestTool(mcpSrv *mcpserver.MCPServer) mcpserver.ServerTool {
	return mcpserver.ServerTool{
		Tool: mcp.NewTool("create_quest",
			mcp.WithDescription("Create a new quest in Tavern. tome_id must be a numeric Ent global ID (e.g. 107374182403) obtained from list_tomes — do not pass a name string or guess small integers like 1."),
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
		if params == "" {
			params = "{}"
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

		// Look up beacon names for the elicitation message.
		const maxBeaconNames = 8
		beacons, err := client.Beacon.Query().
			Where(beacon.IDIn(beaconIDs...)).
			Limit(maxBeaconNames + 1).
			All(ctx)
		if err != nil {
			return mcp.NewToolResultError(fmt.Sprintf("failed to look up beacons: %v", err)), nil
		}
		beaconNames := make([]string, 0, len(beacons))
		for _, b := range beacons {
			beaconNames = append(beaconNames, b.Name)
		}
		beaconSummary := strings.Join(beaconNames, ", ")
		if len(beaconIDs) > maxBeaconNames {
			beaconSummary = strings.Join(beaconNames[:min(len(beaconNames), maxBeaconNames)], ", ") + fmt.Sprintf(" and %d more", len(beaconIDs)-maxBeaconNames)
		}

		// Request human-in-the-loop confirmation before creating the quest.
		// If elicitation is not supported by the client session, proceed without it.
		elicitResult, err := mcpSrv.RequestElicitation(ctx, mcp.ElicitationRequest{
			Params: mcp.ElicitationParams{
				Message:         fmt.Sprintf("Create quest %q using tome %q targeting beacon(s): %s?", name, questTome.Name, beaconSummary),
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
