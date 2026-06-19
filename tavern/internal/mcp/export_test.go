package mcp

import "context"

// ContextWithClient exports the internal client injection for testing.
func ContextWithClient(ctx context.Context, client interface{}) context.Context {
	return context.WithValue(ctx, contextKey{}, client)
}

// Handlers exported for testing
var (
	HandleListHosts    = handleListHosts
	HandleListQuests   = handleListQuests
	HandleListTomes    = handleListTomes
	HandleQuestOutput  = handleQuestOutput
	HandleWaitForQuest = handleWaitForQuest
)
