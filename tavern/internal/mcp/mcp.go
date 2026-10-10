// Package mcp provides a Model Context Protocol (MCP) server for Tavern.
// It exposes Tavern operations as MCP tools accessible via Streamable HTTP transport.
package mcp

import (
	"context"
	"fmt"
	"net/http"
	"strconv"

	"github.com/mark3labs/mcp-go/mcp"
	mcpserver "github.com/mark3labs/mcp-go/server"
	"realm.pub/tavern/internal/ent"
)

// contextKey is used for storing the ent client in the context.
type contextKey struct{}

// graphqlHandlerKey is used for storing the GraphQL HTTP handler in the context.
type graphqlHandlerKey struct{}

// clientFromContext returns the per-request ent client from the context.
func clientFromContext(ctx context.Context) *ent.Client {
	if c, ok := ctx.Value(contextKey{}).(*ent.Client); ok {
		return c
	}
	return nil
}

// graphqlHandlerFromContext returns the GraphQL HTTP handler from the context.
func graphqlHandlerFromContext(ctx context.Context) http.Handler {
	if h, ok := ctx.Value(graphqlHandlerKey{}).(http.Handler); ok {
		return h
	}
	return nil
}

// NewHandler creates an http.Handler that serves the MCP Streamable HTTP endpoints.
// The handler is meant to be mounted at a prefix (e.g. /mcp) on the main server mux.
// Authentication is handled by the parent Tavern HTTP server; this handler
// expects an authenticated context to already be set.
func NewHandler(client *ent.Client, version string, gqlHandler http.Handler) http.Handler {
	mcpSrv := mcpserver.NewMCPServer(
		"tavern",
		version,
		mcpserver.WithToolCapabilities(true),
		mcpserver.WithElicitation(),
	)

	mcpSrv.AddTools(
		listQuestsTool(),
		questOutputTool(),
		listTomesTool(),
		createQuestTool(mcpSrv),
		listHostsTool(),
		waitForQuestTool(),
		graphqlQueryTool(),
	)

	httpSrv := mcpserver.NewStreamableHTTPServer(mcpSrv,
		mcpserver.WithEndpointPath("/mcp"),
		mcpserver.WithHTTPContextFunc(func(ctx context.Context, r *http.Request) context.Context {
			ctx = context.WithValue(ctx, contextKey{}, client)
			ctx = context.WithValue(ctx, graphqlHandlerKey{}, gqlHandler)
			return ctx
		}),
	)

	return httpSrv
}

// ParseIntIDs extracts an array of string IDs from the request arguments and converts them to ints.
func ParseIntIDs(request mcp.CallToolRequest, key string) ([]int, error) {
	args := request.GetArguments()
	raw, ok := args[key]
	if !ok {
		return nil, fmt.Errorf("required argument %q not found", key)
	}
	arr, ok := raw.([]any)
	if !ok {
		return nil, fmt.Errorf("argument %q must be an array", key)
	}
	ids := make([]int, 0, len(arr))
	for _, v := range arr {
		s, ok := v.(string)
		if !ok {
			// Also handle numeric values from JSON
			if n, ok := v.(float64); ok {
				ids = append(ids, int(n))
				continue
			}
			return nil, fmt.Errorf("array element must be a string, got %T", v)
		}
		id, err := strconv.Atoi(s)
		if err != nil {
			return nil, fmt.Errorf("invalid id %q: %v", s, err)
		}
		ids = append(ids, id)
	}
	return ids, nil
}
