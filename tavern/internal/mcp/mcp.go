// Package mcp provides a Model Context Protocol (MCP) server for Tavern.
// It exposes Tavern operations as MCP tools accessible via Streamable HTTP transport.
package mcp

import (
	"context"
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/mark3labs/mcp-go/mcp"
	mcpserver "github.com/mark3labs/mcp-go/server"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/logging"
)

var (
	serverLog = logging.For("mcp.server")
	toolsLog  = logging.For("mcp.tools")
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
		mcpserver.WithToolHandlerMiddleware(auditToolMiddleware),
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
		mcpserver.WithLogger(mcpServerLogger{}),
		mcpserver.WithHTTPContextFunc(func(ctx context.Context, r *http.Request) context.Context {
			ctx = enrichMCPHTTPContext(ctx, r)
			ctx = context.WithValue(ctx, contextKey{}, client)
			ctx = context.WithValue(ctx, graphqlHandlerKey{}, gqlHandler)
			serverLog.DebugContext(ctx, "mcp http request received", "method", r.Method, "path", r.URL.Path)
			return ctx
		}),
	)

	return httpSrv
}

type mcpServerLogger struct{}

func (mcpServerLogger) Infof(format string, v ...any) {
	serverLog.Debug(fmt.Sprintf(format, v...))
}

func (mcpServerLogger) Errorf(format string, v ...any) {
	serverLog.Error(fmt.Sprintf(format, v...))
}

func enrichMCPHTTPContext(ctx context.Context, r *http.Request) context.Context {
	if ctx == nil {
		ctx = context.Background()
	}
	ctx = logging.WithEntrypoint(ctx, "mcp")
	if r != nil && logging.ClientIPFromContext(ctx) == "" {
		ip := logging.NormalizeClientIP(
			r.RemoteAddr,
			r.Header.Get("X-Redirected-For"),
			r.Header.Get("X-Forwarded-For"),
		)
		ctx = logging.WithClientIP(ctx, ip)
	}
	return ensureMCPToolContext(ctx)
}

func ensureMCPToolContext(ctx context.Context) context.Context {
	if ctx == nil {
		ctx = context.Background()
	}
	ctx = logging.WithEntrypoint(ctx, "mcp")
	if logging.ActorFromContext(ctx).ID == "" {
		if u := auth.UserFromContext(ctx); u != nil {
			ctx = logging.WithActor(ctx, logging.Actor{
				Kind:    logging.ActorUser,
				ID:      strconv.Itoa(u.ID),
				Name:    u.Name,
				IsAdmin: u.IsAdmin,
			})
		}
	}
	ctx, _, _ = logging.EnsureTrace(ctx)
	return ctx
}

func auditToolMiddleware(next mcpserver.ToolHandlerFunc) mcpserver.ToolHandlerFunc {
	return func(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		ctx = ensureMCPToolContext(ctx)
		toolName := request.Params.Name
		scrubbedArgs := scrubMCPArguments(request.GetArguments())
		start := time.Now()

		toolsLog.DebugContext(ctx, "invoking mcp tool", "tool_name", toolName)

		res, err := next(ctx, request)
		durationMS := time.Since(start).Milliseconds()

		outcome := logging.OutcomeSuccess
		var errMsg string
		if err != nil {
			outcome = logging.OutcomeFailure
			errMsg = err.Error()
		} else if res != nil && res.IsError {
			errMsg = extractToolErrorText(res)
			if isDeniedToolError(errMsg) {
				outcome = logging.OutcomeDenied
			} else {
				outcome = logging.OutcomeFailure
			}
		}

		attrs := append(logging.TargetAttrs("mcp_tool", toolName, toolName),
			"tool_name", toolName,
			"arguments", scrubbedArgs,
			"duration_ms", durationMS,
		)
		if errMsg != "" {
			attrs = append(attrs, "error", errMsg)
		}

		logging.AuditRecord(ctx, "mcp", "mcp.tool_called", outcome, fmt.Sprintf("MCP tool %s called", toolName), attrs...)
		return res, err
	}
}

func scrubMCPArguments(args map[string]any) map[string]any {
	if args == nil {
		return map[string]any{}
	}
	scrubbed := logging.ScrubMap(args)
	for k, v := range scrubbed {
		s, ok := v.(string)
		if !ok || s == logging.RedactedPlaceholder {
			continue
		}
		trimmed := strings.TrimSpace(s)
		if strings.HasPrefix(trimmed, "{") || strings.HasPrefix(trimmed, "[") {
			scrubbed[k] = logging.ScrubJSONValue(s)
		}
	}
	return scrubbed
}

func extractToolErrorText(res *mcp.CallToolResult) string {
	if res == nil {
		return ""
	}
	var parts []string
	for _, c := range res.Content {
		if txt, ok := c.(mcp.TextContent); ok && txt.Text != "" {
			parts = append(parts, txt.Text)
		}
	}
	return strings.Join(parts, "; ")
}

func isDeniedToolError(msg string) bool {
	lower := strings.ToLower(msg)
	return strings.Contains(lower, "cancelled by user") ||
		strings.Contains(lower, "only queries are allowed") ||
		strings.Contains(lower, "permission denied") ||
		strings.Contains(lower, "unauthorized")
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
