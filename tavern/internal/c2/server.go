package c2

import (
	"context"
	"log/slog"
	"net"
	"strings"

	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/peer"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/http/stream"
)

type Server struct {
	MaxFileChunkSize uint64
	graph            *ent.Client
	mux              *stream.Mux
	c2pb.UnimplementedC2Server
}

func New(graph *ent.Client, mux *stream.Mux) *Server {
	return &Server{
		MaxFileChunkSize: 1024 * 1024, // 1 MB
		graph:            graph,
		mux:              mux,
	}
}

func validateIP(ipaddr string) bool {
	return net.ParseIP(ipaddr) != nil || ipaddr == "unknown"
}

func getRemoteIP(ctx context.Context) string {
	p, ok := peer.FromContext(ctx)
	if !ok {
		return "unknown"
	}

	host, _, err := net.SplitHostPort(p.Addr.String())
	if err != nil {
		return "unknown"
	}

	return host
}

func GetClientIP(ctx context.Context) string {
	md, ok := metadata.FromIncomingContext(ctx)
	if ok {
		if redirectedFor, exists := md["x-redirected-for"]; exists && len(redirectedFor) > 0 {
			clientIP := strings.TrimSpace(redirectedFor[0])
			if validateIP(clientIP) {
				return clientIP
			} else {
				slog.Error("bad redirect for ip", "ip", clientIP)
			}
		}
		if forwardedFor, exists := md["x-forwarded-for"]; exists && len(forwardedFor) > 0 {
			// X-Forwarded-For is a comma-separated list, the first IP is the original client
			clientIP := strings.TrimSpace(strings.Split(forwardedFor[0], ",")[0])
			if validateIP(clientIP) {
				return clientIP
			} else {
				slog.Error("bad forwarded for ip", "ip", clientIP)
			}
		}
	}

	// Fallback to peer address
	remoteIp := getRemoteIP(ctx)
	if validateIP(remoteIp) {
		return remoteIp
	} else {
		slog.Error("Bad remote IP", "ip", remoteIp)
	}
	return "unknown"
}
