package redirectors

import (
	"context"

	"google.golang.org/grpc/metadata"
)

// ExternalIPNoop is a special value used to indicate that the external IP should not be updated.
// This is used by redirectors (e.g., DNS) that cannot determine the client's external IP.
const ExternalIPNoop = "NOOP"

// SetRedirectedForHeader sets the x-redirected-for header in the outgoing context metadata
// with the provided client IP address. This header is used to track the original client IP
// through the redirector chain.
func SetRedirectedForHeader(ctx context.Context, clientIP string) context.Context {
	if clientIP == "" {
		return ctx
	}

	outMd, ok := metadata.FromOutgoingContext(ctx)
	if !ok || outMd == nil {
		outMd = metadata.New(nil)
	} else {
		outMd = outMd.Copy()
	}
	outMd.Set("x-redirected-for", clientIP)
	log.TraceContext(ctx, "setting x-redirected-for header", "client_ip", clientIP)
	return metadata.NewOutgoingContext(ctx, outMd)
}
