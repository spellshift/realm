package mcp

import (
	"context"
	"net/http"

	"realm.pub/tavern/internal/ent"
)

var (
	HandleListHosts    = handleListHosts
	HandleListQuests   = handleListQuests
	HandleListTomes    = handleListTomes
	HandleCreateQuest  = handleCreateQuest
	HandleQuestOutput  = handleQuestOutput
	HandleWaitForQuest = handleWaitForQuest
)

// CtxWithClient exposes the contextKey for testing.
func CtxWithClient(ctx context.Context, client *ent.Client) context.Context {
	return context.WithValue(ctx, contextKey{}, client)
}

// CtxWithGraphQLHandler exposes the graphqlHandlerKey for testing.
func CtxWithGraphQLHandler(ctx context.Context, handler http.Handler) context.Context {
	return context.WithValue(ctx, graphqlHandlerKey{}, handler)
}
