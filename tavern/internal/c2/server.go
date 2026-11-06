package c2

import (
	"context"
	"crypto/ed25519"
	"encoding/json"
	"fmt"
	"log/slog"
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
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/redirectors"
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

const ClaimBeaconID = "beacon_id"

// generateTaskJWT creates a signed JWT token containing the beacon ID
func (srv *Server) generateTaskJWT(beaconID int) (string, error) {
	claims := jwt.MapClaims{
		ClaimBeaconID: beaconID,
		"iat":         time.Now().Unix(),
		"exp":         time.Now().Add(1 * time.Hour).Unix(), // Token expires in 1 hour
	}

	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	signedToken, err := token.SignedString(srv.jwtPrivateKey)
	if err != nil {
		return "", fmt.Errorf("failed to sign JWT: %w", err)
	}

	return signedToken, nil
}

func (srv *Server) ValidateJWT(jwttoken string) (int, error) {
	token, err := jwt.Parse(jwttoken, func(token *jwt.Token) (any, error) {
		// 1. Verify the signing method is EdDSA
		if _, ok := token.Method.(*jwt.SigningMethodEd25519); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
		}
		// 2. Return the PUBLIC key for verification
		return srv.jwtPublicKey, nil
	}, jwt.WithJSONNumber())

	if err != nil || !token.Valid {
		return 0, status.Errorf(codes.PermissionDenied, "invalid token: %v", err)
	}

	beaconID, ok := token.Claims.(jwt.MapClaims)[ClaimBeaconID].(json.Number)
	if !ok {
		return 0, status.Error(codes.PermissionDenied, "invalid beacon ID claim")
	}
	id, err := strconv.Atoi(string(beaconID))
	if err != nil || id <= 0 {
		return 0, status.Error(codes.PermissionDenied, "invalid beacon ID claim")
	}
	slog.Info(fmt.Sprintf("received valid JWT: %s", jwttoken))
	return id, nil
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
