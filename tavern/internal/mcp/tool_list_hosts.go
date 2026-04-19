package mcp

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/mark3labs/mcp-go/mcp"
	mcpserver "github.com/mark3labs/mcp-go/server"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/tag"
)

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
