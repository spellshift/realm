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
	"realm.pub/tavern/internal/jwt"
	"realm.pub/tavern/internal/portals/mux"
)

type Server struct {
	MaxFileChunkSize uint64
	graph            *ent.Client
	mux              *stream.Mux
	portalMux        *mux.Mux
	jwtService       *jwt.Service

	c2pb.UnimplementedC2Server
}

func New(graph *ent.Client, mux *stream.Mux, portalMux *mux.Mux, jwtService *jwt.Service) *Server {
	return &Server{
		MaxFileChunkSize: 1024 * 1024, // 1 MB
		graph:            graph,
		mux:              mux,
		portalMux:        portalMux,
		jwtService:       jwtService,
	}
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
				slog.Error("bad x-redirected-for ip", "ip", clientIP)
			}
		}
		if forwardedFor, exists := md["x-forwarded-for"]; exists && len(forwardedFor) > 0 {
			// X-Forwarded-For is a comma-separated list, the first IP is the original client
			clientIP := strings.TrimSpace(strings.Split(forwardedFor[0], ",")[0])
			if validateIP(clientIP) {
				return clientIP
			} else {
				slog.Error("bad x-forwarded-for ip", "ip", clientIP)
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

// validateTaskJWT validates a JWT token for a specific task
// Returns true if valid, false otherwise
// Logs a warning if the JWT is invalid
func (srv *Server) validateTaskJWT(ctx context.Context, jwtToken string, expectedTaskID int64) bool {
	if jwtToken == "" {
		slog.WarnContext(ctx, "missing JWT in request", "task_id", expectedTaskID)
		return false
	}

	claims, err := srv.jwtService.ValidateTaskToken(jwtToken)
	if err != nil {
		slog.WarnContext(ctx, "invalid JWT in request", "task_id", expectedTaskID, "jwt", jwtToken, "err", err)
		return false
	}

	if claims.TaskID != expectedTaskID {
		slog.WarnContext(ctx, "JWT task ID mismatch", "expected_task_id", expectedTaskID, "jwt_task_id", claims.TaskID, "jwt", jwtToken)
		return false
	}

	return true
}

// validateJWT validates a JWT token without checking task ID
// Returns the task ID from the JWT claims if valid, or -1 if invalid
// Logs a warning if the JWT is invalid
func (srv *Server) validateJWT(ctx context.Context, jwtToken string) int64 {
	if jwtToken == "" {
		slog.WarnContext(ctx, "missing JWT in request", "jwt", jwtToken)
		return -1
	}

	claims, err := srv.jwtService.ValidateTaskToken(jwtToken)
	if err != nil {
		slog.WarnContext(ctx, "invalid JWT in request", "jwt", jwtToken, "err", err)
		return -1
	}

	return claims.TaskID
}
