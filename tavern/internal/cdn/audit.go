package cdn

import (
	"context"
	"net/http"
	"strconv"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/logging"
)

var (
	uploadLog   = logging.For("cdn.upload")
	downloadLog = logging.For("cdn.download")
	linkLog     = logging.For("cdn.link")
)

// cdnAuditCtx ensures CDN request contexts carry http entrypoint, client IP,
// trace/request correlation IDs, and fallback Actor attribution when invoked
// directly without outer Tavern HTTP middleware.
func cdnAuditCtx(req *http.Request) context.Context {
	ctx := context.Background()
	if req != nil && req.Context() != nil {
		ctx = req.Context()
	}
	if logging.EntrypointFromContext(ctx) == "" {
		ctx = logging.WithEntrypoint(ctx, "http")
	}
	if req != nil && logging.ClientIPFromContext(ctx) == "" {
		ip := logging.NormalizeClientIP(
			req.RemoteAddr,
			req.Header.Get("X-Redirected-For"),
			req.Header.Get("X-Forwarded-For"),
		)
		ctx = logging.WithClientIP(ctx, ip)
	}
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
