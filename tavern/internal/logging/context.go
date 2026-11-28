package logging

import (
	"context"
	"log/slog"
	"net"
	"strings"
)

type metaCtxKey struct{}

// ActorKind identifies the category of principal performing an action.
type ActorKind string

const (
	ActorUnknown    ActorKind = ""
	ActorUser       ActorKind = "user"
	ActorBeacon     ActorKind = "beacon"
	ActorBuilder    ActorKind = "builder"
	ActorRedirector ActorKind = "redirector"
	ActorScheduler  ActorKind = "scheduler"
	ActorAnonymous  ActorKind = "anonymous"
	ActorSystem     ActorKind = "system"
)

// Actor describes the authenticated (or anonymous/system) principal associated with a context.
type Actor struct {
	Kind       ActorKind
	ID         string
	Name       string
	IsAdmin    bool
	AuthMethod string
}

// ContextMeta holds request correlation, attribution, and stream-scoped logging attributes.
type ContextMeta struct {
	TraceID     string
	RequestID   string
	Entrypoint  string
	ClientIP    string
	Actor       Actor
	StreamAttrs []slog.Attr
}

// MetaFromContext returns the ContextMeta stored on ctx (or an empty value if none).
func MetaFromContext(ctx context.Context) ContextMeta {
	if ctx == nil {
		return ContextMeta{}
	}
	m, _ := ctx.Value(metaCtxKey{}).(ContextMeta)
	return m
}

func withMeta(ctx context.Context, m ContextMeta) context.Context {
	if ctx == nil {
		ctx = context.Background()
	}
	return context.WithValue(ctx, metaCtxKey{}, m)
}

// WithTrace returns a child context carrying traceID and requestID.
func WithTrace(ctx context.Context, traceID, requestID string) context.Context {
	m := MetaFromContext(ctx)
	if traceID != "" {
		m.TraceID = traceID
	}
	if requestID != "" {
		m.RequestID = requestID
	}
	return withMeta(ctx, m)
}

// EnsureTrace guarantees ctx has both a TraceID and RequestID, generating missing ones.
func EnsureTrace(ctx context.Context) (context.Context, string, string) {
	m := MetaFromContext(ctx)
	changed := false
	if m.TraceID == "" {
		m.TraceID = NewTraceID()
		changed = true
	}
	if m.RequestID == "" {
		m.RequestID = NewRequestID()
		changed = true
	}
	if changed {
		ctx = withMeta(ctx, m)
	}
	return ctx, m.TraceID, m.RequestID
}

// WithEntrypoint sets the ingress entrypoint ("http", "graphql", "grpc", "websocket", "mcp", "scheduler", "internal").
func WithEntrypoint(ctx context.Context, entrypoint string) context.Context {
	m := MetaFromContext(ctx)
	m.Entrypoint = entrypoint
	return withMeta(ctx, m)
}

// WithClientIP attaches a normalized client IP to ctx.
func WithClientIP(ctx context.Context, clientIP string) context.Context {
	if clientIP == "" {
		return ctx
	}
	m := MetaFromContext(ctx)
	m.ClientIP = clientIP
	return withMeta(ctx, m)
}

// WithActor attaches Actor attribution to ctx, preserving an earlier AuthMethod if the new Actor leaves it blank.
func WithActor(ctx context.Context, actor Actor) context.Context {
	m := MetaFromContext(ctx)
	if actor.AuthMethod == "" && m.Actor.AuthMethod != "" && m.Actor.Kind == actor.Kind && m.Actor.ID == actor.ID {
		actor.AuthMethod = m.Actor.AuthMethod
	}
	m.Actor = actor
	return withMeta(ctx, m)
}

// WithAuthMethod updates the Actor's AuthMethod on ctx (e.g. "bypass", "cookie", "access_token").
func WithAuthMethod(ctx context.Context, method string) context.Context {
	m := MetaFromContext(ctx)
	m.Actor.AuthMethod = method
	return withMeta(ctx, m)
}

// WithStreamAttrs binds persistent attributes (such as portal_id, shell_id, pivot_id) to a stream context.
func WithStreamAttrs(ctx context.Context, attrs ...slog.Attr) context.Context {
	if len(attrs) == 0 {
		return ctx
	}
	m := MetaFromContext(ctx)
	combined := make([]slog.Attr, 0, len(m.StreamAttrs)+len(attrs))
	combined = append(combined, m.StreamAttrs...)
	combined = append(combined, attrs...)
	m.StreamAttrs = combined
	return withMeta(ctx, m)
}

// DetachContext returns a context that preserves TraceID, RequestID, Actor, and stream metadata
// without inheriting parent cancellation or deadlines (for async background tasks).
func DetachContext(ctx context.Context) context.Context {
	if ctx == nil {
		return context.Background()
	}
	return context.WithoutCancel(ctx)
}

// TraceIDFromContext returns the trace_id stored in ctx, if any.
func TraceIDFromContext(ctx context.Context) string {
	return MetaFromContext(ctx).TraceID
}

// RequestIDFromContext returns the request_id stored in ctx, if any.
func RequestIDFromContext(ctx context.Context) string {
	return MetaFromContext(ctx).RequestID
}

// ActorFromContext returns the Actor stored in ctx, if any.
func ActorFromContext(ctx context.Context) Actor {
	return MetaFromContext(ctx).Actor
}

// ClientIPFromContext returns the client IP stored in ctx, if any.
func ClientIPFromContext(ctx context.Context) string {
	return MetaFromContext(ctx).ClientIP
}

// EntrypointFromContext returns the entrypoint stored in ctx, if any.
func EntrypointFromContext(ctx context.Context) string {
	return MetaFromContext(ctx).Entrypoint
}

// NormalizeClientIP resolves the best client IP from x-redirected-for, x-forwarded-for, or RemoteAddr,
// stripping any host:port suffix when valid.
func NormalizeClientIP(remoteAddr, redirectedFor, forwardedFor string) string {
	for _, candidate := range []string{redirectedFor, forwardedFor, remoteAddr} {
		candidate = strings.TrimSpace(candidate)
		if candidate == "" {
			continue
		}
		if comma := strings.IndexByte(candidate, ','); comma >= 0 {
			candidate = strings.TrimSpace(candidate[:comma])
		}
		if host, _, err := net.SplitHostPort(candidate); err == nil && host != "" {
			return host
		}
		return candidate
	}
	return ""
}
