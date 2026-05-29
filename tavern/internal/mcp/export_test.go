package mcp

import (
	"context"

	mcpserver "github.com/mark3labs/mcp-go/server"
)

// Export internal tool handlers for testing purposes without polluting the public API.
var (
	HandleListHosts    = handleListHosts
	HandleListQuests   = handleListQuests
	HandleListTomes    = handleListTomes
	HandleQuestOutput  = handleQuestOutput
	HandleWaitForQuest = handleWaitForQuest
)

// HandleCreateQuestForTest is an exported wrapper that provides an MCPServer for the handler.
func HandleCreateQuestForTest(srv *mcpserver.MCPServer) mcpserver.ToolHandlerFunc {
	return handleCreateQuest(srv)
}

// ClientFromContextForTest exports the context key injection for testing.
func ClientFromContextForTest(ctx context.Context, client any) context.Context {
	return context.WithValue(ctx, contextKey{}, client)
}
