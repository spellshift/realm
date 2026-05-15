package mcp

import (
	"context"
	"net/http"

	mcpserver "github.com/mark3labs/mcp-go/server"
	"realm.pub/tavern/internal/ent"
)

// Exported wrappers for tests
func ContextWithClientForTest(ctx context.Context, client *ent.Client) context.Context {
	return context.WithValue(ctx, contextKey{}, client)
}

func ContextWithGraphQLHandlerForTest(ctx context.Context, handler http.Handler) context.Context {
	return context.WithValue(ctx, graphqlHandlerKey{}, handler)
}

var HandleListQuestsForTest = handleListQuests
var HandleListHostsForTest = handleListHosts
var HandleListTomesForTest = handleListTomes
var HandleQuestOutputForTest = handleQuestOutput
var HandleWaitForQuestForTest = handleWaitForQuest

func HandleCreateQuestForTest(mcpSrv *mcpserver.MCPServer) mcpserver.ToolHandlerFunc {
	return handleCreateQuest(mcpSrv)
}
var HandleGraphQLQueryForTest = handleGraphQLQuery
