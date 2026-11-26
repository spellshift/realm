package c2

import (
	"context"
	"crypto/ed25519"
	"encoding/json"
	"errors"
	"fmt"
	"net"
	"strconv"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/ent/shell"
	"realm.pub/tavern/internal/ent/shelltask"
	"realm.pub/tavern/internal/ent/task"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/redirectors"
)

var (
	c2Log     = logging.For("c2")
	reportLog = logging.For("c2.report")
)

type Server struct {
	MaxFileChunkSize uint64
	graph            *ent.Client
	mux              *stream.Mux
	portalMux        *mux.Mux
	jwtPrivateKey    ed25519.PrivateKey
	jwtPublicKey     ed25519.PublicKey

	c2pb.UnimplementedC2Server
}

func New(graph *ent.Client, mux *stream.Mux, portalMux *mux.Mux, jwtPublicKey ed25519.PublicKey, jwtPrivateKey ed25519.PrivateKey, opts ...Option) *Server {
	srv := &Server{
		MaxFileChunkSize: 1024 * 1024, // 1 MB
		graph:            graph,
		mux:              mux,
		portalMux:        portalMux,
		jwtPrivateKey:    jwtPrivateKey,
		jwtPublicKey:     jwtPublicKey,
	}
	for _, opt := range opts {
		opt(srv)
	}
	return srv
}

// Option configures a C2 Server.
type Option func(*Server)

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
			// Return NOOP directly if set by redirector
			if clientIP == redirectors.ExternalIPNoop {
				return redirectors.ExternalIPNoop
			}
			if validateIP(clientIP) {
				return clientIP
			}
			c2Log.WarnContext(ctx, "invalid x-redirected-for IP", "ip", clientIP)
		}
		if forwardedFor, exists := md["x-forwarded-for"]; exists && len(forwardedFor) > 0 {
			// X-Forwarded-For is a comma-separated list, the first IP is the original client
			clientIP := strings.TrimSpace(strings.Split(forwardedFor[0], ",")[0])
			if validateIP(clientIP) {
				return clientIP
			}
			c2Log.WarnContext(ctx, "invalid x-forwarded-for IP", "ip", clientIP)
		}
	}

	// Fallback to peer address
	remoteIP := getRemoteIP(ctx)
	if validateIP(remoteIP) {
		return remoteIP
	}
	if remoteIP == "unknown" {
		c2Log.TraceContext(ctx, "no remote peer IP in context")
		return "unknown"
	}
	c2Log.WarnContext(ctx, "invalid remote peer IP", "ip", remoteIP)
	return "unknown"
}

func enrichC2Context(ctx context.Context) (context.Context, string) {
	ctx, _, _ = logging.ExtractGRPCTrace(ctx)
	if logging.EntrypointFromContext(ctx) == "" {
		ctx = logging.WithEntrypoint(ctx, "grpc")
	}
	clientIP := GetClientIP(ctx)
	if clientIP != "" && clientIP != "unknown" && clientIP != redirectors.ExternalIPNoop {
		ctx = logging.WithClientIP(ctx, clientIP)
	}
	return ctx, clientIP
}

const (
	ClaimBeaconID   = "beacon_id"
	TaskJWTAudience = "tavern.c2.task"
)

// generateTaskJWT creates a signed JWT token containing the beacon ID
func (srv *Server) generateTaskJWT(beaconID int) (string, error) {
	now := time.Now()
	claims := jwt.MapClaims{
		ClaimBeaconID: beaconID,
		"aud":         jwt.ClaimStrings{TaskJWTAudience},
		"iat":         now.Unix(),
		"exp":         now.Add(1 * time.Hour).Unix(), // Token expires in 1 hour
	}

	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	signedToken, err := token.SignedString(srv.jwtPrivateKey)
	if err != nil {
		return "", fmt.Errorf("failed to sign JWT: %w", err)
	}

	return signedToken, nil
}

func classifyJWTError(jwttoken string, err error) string {
	if strings.TrimSpace(jwttoken) == "" {
		return "empty_token"
	}
	switch {
	case errors.Is(err, jwt.ErrTokenExpired):
		return "token_expired"
	case errors.Is(err, jwt.ErrTokenSignatureInvalid):
		return "invalid_signature"
	case errors.Is(err, jwt.ErrTokenMalformed):
		return "malformed_token"
	default:
		return "invalid_token"
	}
}

func (srv *Server) ValidateJWT(jwttoken string) (int, error) {
	_, id, err := srv.ValidateJWTContext(context.Background(), jwttoken)
	return id, err
}

// ValidateJWTContext verifies a C2 task JWT without ever logging the raw token,
// emits c2.jwt_denied on failure, and attaches ActorBeacon attribution to ctx on success.
func (srv *Server) ValidateJWTContext(ctx context.Context, jwttoken string) (context.Context, int, error) {
	token, err := jwt.Parse(jwttoken, func(token *jwt.Token) (any, error) {
		// 1. Verify the signing method is EdDSA
		if _, ok := token.Method.(*jwt.SigningMethodEd25519); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
		}
		// 2. Return the PUBLIC key for verification
		return srv.jwtPublicKey, nil
	},
		jwt.WithJSONNumber(),
		jwt.WithValidMethods([]string{jwt.SigningMethodEdDSA.Alg()}),
		jwt.WithExpirationRequired(),
		jwt.WithIssuedAt(),
		jwt.WithAudience(TaskJWTAudience),
	)

	if err != nil || !token.Valid {
		logging.AuditDenied(ctx, "c2", "c2.jwt_denied", "invalid C2 task JWT",
			"reason", classifyJWTError(jwttoken, err),
		)
		return ctx, 0, status.Errorf(codes.PermissionDenied, "invalid token: %v", err)
	}

	claims, ok := token.Claims.(jwt.MapClaims)
	if !ok {
		logging.AuditDenied(ctx, "c2", "c2.jwt_denied", "invalid C2 task JWT",
			"reason", "invalid_claims",
		)
		return ctx, 0, status.Error(codes.PermissionDenied, "invalid beacon ID claim")
	}

	beaconID, ok := claims[ClaimBeaconID].(json.Number)
	if !ok {
		logging.AuditDenied(ctx, "c2", "c2.jwt_denied", "invalid C2 task JWT",
			"reason", "invalid_beacon_id_claim",
		)
		return ctx, 0, status.Error(codes.PermissionDenied, "invalid beacon ID claim")
	}
	id, err := strconv.Atoi(string(beaconID))
	if err != nil || id <= 0 {
		logging.AuditDenied(ctx, "c2", "c2.jwt_denied", "invalid C2 task JWT",
			"reason", "invalid_beacon_id_claim",
		)
		return ctx, 0, status.Error(codes.PermissionDenied, "invalid beacon ID claim")
	}

	actor := logging.Actor{
		Kind:       logging.ActorBeacon,
		ID:         strconv.Itoa(id),
		AuthMethod: "task_jwt",
	}
	if srv.graph != nil {
		if b, err := srv.graph.Beacon.Get(ctx, id); err == nil && b != nil {
			actor.Name = b.Name
		}
	}
	ctx = logging.WithActor(ctx, actor)
	c2Log.TraceContext(ctx, "verified task JWT", "beacon_id", id)
	return ctx, id, nil
}

func (srv *Server) loadTaskForBeacon(ctx context.Context, taskID, beaconID int) (*ent.Task, error) {
	return srv.graph.Task.Query().
		Where(task.ID(taskID), task.HasBeaconWith(beacon.ID(beaconID))).
		Only(ctx)
}

func (srv *Server) loadShellTaskForBeacon(ctx context.Context, shellTaskID, beaconID int) (*ent.ShellTask, error) {
	return srv.graph.ShellTask.Query().
		Where(shelltask.ID(shellTaskID), shelltask.HasShellWith(shell.HasBeaconWith(beacon.ID(beaconID)))).
		Only(ctx)
}
