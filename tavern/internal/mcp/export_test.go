package mcp

import (
	"context"

	"github.com/mark3labs/mcp-go/mcp"
)

// Export for testing
var (
	HandleListHosts    = handleListHosts
	HandleListQuests   = handleListQuests
	HandleListTomes    = handleListTomes
	HandleQuestOutput  = handleQuestOutput
	HandleWaitForQuest = handleWaitForQuest
	ClientFromContext  = clientFromContext
)

// Export internal context keys
type TestContextKey = contextKey
type TestGraphqlHandlerKey = graphqlHandlerKey

// HandleCreateQuestForTest exposes handleCreateQuest which requires the server instance.
func HandleCreateQuestForTest(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
	// The real handler expects a server instance, but in unit tests we might
	// pass a dummy or nil if the internal logic allows it.
	return handleCreateQuest(nil)(ctx, req)
}
