package redirectors

import (
	"context"
	"log/slog"

	"google.golang.org/grpc/metadata"
)

// SetRedirectedForHeader sets the x-redirected-for header in the outgoing context metadata
// with the provided client IP address. This header is used to track the original client IP
// through the redirector chain.
func SetRedirectedForHeader(ctx context.Context, clientIP string) context.Context {
	if clientIP == "" {
		return ctx
	}

	outMd, _ := metadata.FromOutgoingContext(ctx)
	if outMd == nil {
		outMd = metadata.New(nil)
	}
	outMd.Set("x-redirected-for", clientIP)
	slog.Info("Setting redirected-for header", "clientIP", clientIP)
	return metadata.NewOutgoingContext(ctx, outMd)
}
