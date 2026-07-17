package mcp

// Export unexported types and functions for testing

type ContextKey = contextKey

var (
	HandleListHosts   = handleListHosts
	HandleListQuests  = handleListQuests
	HandleListTomes   = handleListTomes
	HandleQuestOutput = handleQuestOutput
)
