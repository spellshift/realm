package mcp

import (
	"context"
	"encoding/json"
	"fmt"
	"strconv"
	"time"

	"github.com/mark3labs/mcp-go/mcp"
	mcpserver "github.com/mark3labs/mcp-go/server"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/quest"
)

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
