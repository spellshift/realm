package mcp

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http/httptest"

	"github.com/mark3labs/mcp-go/mcp"
	mcpserver "github.com/mark3labs/mcp-go/server"
	"github.com/vektah/gqlparser/v2/ast"
	"github.com/vektah/gqlparser/v2/parser"
)

// graphqlQueryTool returns the graphql_query MCP tool.
func graphqlQueryTool() mcpserver.ServerTool {
	return mcpserver.ServerTool{
		Tool: mcp.NewTool("graphql_query",
			mcp.WithDescription("Execute a raw GraphQL **query** against the Tavern API. Only read-only queries are permitted — mutations and subscriptions are rejected. Use the introspection query `{ __schema { types { name } } }` to discover the full schema, available types, and field definitions."),
			mcp.WithReadOnlyHintAnnotation(true),
			mcp.WithDestructiveHintAnnotation(false),
			mcp.WithString("query",
				mcp.Required(),
				mcp.Description("The GraphQL query string to execute. Must be a query operation — mutations and subscriptions are not allowed. Supports introspection (e.g. __schema, __type)."),
			),
			mcp.WithObject("variables",
				mcp.Description("Optional map of GraphQL variables to pass with the query"),
			),
		),
		Handler: handleGraphQLQuery,
	}
}

// handleGraphQLQuery executes a validated read-only GraphQL query against the Tavern GraphQL handler.
func handleGraphQLQuery(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
	query, err := request.RequireString("query")
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("invalid query: %v", err)), nil
	}

	// Parse and validate the query — only allow query operations.
	doc, parseErr := parser.ParseQuery(&ast.Source{Input: query})
	if parseErr != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to parse GraphQL query: %v", parseErr)), nil
	}
	for _, op := range doc.Operations {
		if op.Operation != ast.Query {
			return mcp.NewToolResultError(fmt.Sprintf("only queries are allowed, got %s operation", op.Operation)), nil
		}
	}

	gqlHandler := graphqlHandlerFromContext(ctx)
	if gqlHandler == nil {
		return mcp.NewToolResultError("internal error: no GraphQL handler available"), nil
	}

	// Build the GraphQL request body.
	var variables map[string]any
	if args := request.GetArguments(); args["variables"] != nil {
		if v, ok := args["variables"].(map[string]any); ok {
			variables = v
		}
	}

	gqlReqBody := map[string]any{
		"query": query,
	}
	if variables != nil {
		gqlReqBody["variables"] = variables
	}

	bodyBytes, err := json.Marshal(gqlReqBody)
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to build request: %v", err)), nil
	}

	// Execute the query against the in-process GraphQL handler.
	httpReq := httptest.NewRequest("POST", "/graphql", bytes.NewReader(bodyBytes))
	httpReq.Header.Set("Content-Type", "application/json")
	httpReq = httpReq.WithContext(ctx)

	recorder := httptest.NewRecorder()
	gqlHandler.ServeHTTP(recorder, httpReq)

	resp := recorder.Result()
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to read response: %v", err)), nil
	}

	if resp.StatusCode != 200 {
		return mcp.NewToolResultError(fmt.Sprintf("GraphQL request failed (HTTP %d): %s", resp.StatusCode, string(respBody))), nil
	}

	return mcp.NewToolResultText(string(respBody)), nil
}

// HandleGraphQLQueryForTest is an exported wrapper for testing the query validation logic.
func HandleGraphQLQueryForTest(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
	return handleGraphQLQuery(ctx, request)
}
