package c2

import (
	"context"
	"crypto/ed25519"
	"fmt"
	"log/slog"
	"net"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
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

// ClaimBeaconID is the JWT claim carrying the beacon DB ID the token is bound to.
const ClaimBeaconID = "beacon_id"

// generateTaskJWT creates a signed JWT token bound to the given beacon ID.
// Beacons may only use the token to report data for tasks owned by that beacon.
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

// ValidateJWT verifies the token signature + expiry and returns the beacon ID
// the token is bound to. Tokens without a valid beacon_id claim are rejected so
// a beacon can only report data attached to its own beacon.
func (srv *Server) ValidateJWT(jwttoken string) (int, error) {
	token, err := jwt.Parse(jwttoken, func(token *jwt.Token) (any, error) {
		// 1. Verify the signing method is EdDSA
		if _, ok := token.Method.(*jwt.SigningMethodEd25519); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
		}
		// 2. Return the PUBLIC key for verification
		return srv.jwtPublicKey, nil
	})

	if err != nil || !token.Valid {
		return 0, status.Errorf(codes.PermissionDenied, "invalid token: %v", err)
	}

	claims, ok := token.Claims.(jwt.MapClaims)
	if !ok {
		return 0, status.Errorf(codes.PermissionDenied, "invalid token claims")
	}

	beaconID, err := parseBeaconIDClaim(claims[ClaimBeaconID])
	if err != nil {
		return 0, status.Errorf(codes.PermissionDenied, "invalid token: %v", err)
	}

	return beaconID, nil
}

// authorizeTaskForBeacon ensures the task belongs to the beacon bound to the JWT.
func (srv *Server) authorizeTaskForBeacon(ctx context.Context, t *ent.Task, beaconID int) error {
	ownerID, err := t.QueryBeacon().OnlyID(ctx)
	if err != nil {
		return status.Errorf(codes.Internal, "failed to load task owner: %v", err)
	}
	if ownerID != beaconID {
		return status.Errorf(codes.PermissionDenied, "task %d does not belong to this beacon", t.ID)
	}
	return nil
}

// authorizeShellTaskForBeacon ensures the shell task belongs (via its shell) to the beacon bound to the JWT.
func (srv *Server) authorizeShellTaskForBeacon(ctx context.Context, st *ent.ShellTask, beaconID int) error {
	ownerID, err := st.QueryShell().QueryBeacon().OnlyID(ctx)
	if err != nil {
		return status.Errorf(codes.Internal, "failed to load shell task owner: %v", err)
	}
	if ownerID != beaconID {
		return status.Errorf(codes.PermissionDenied, "shell task %d does not belong to this beacon", st.ID)
	}
	return nil
}

// parseBeaconIDClaim extracts the beacon ID from the JWT beacon_id claim.
// Numeric JSON claims decode as float64, so accept float64, int variants, and strings.
func parseBeaconIDClaim(v any) (int, error) {
	switch id := v.(type) {
	case float64:
		if id <= 0 || id != float64(int(id)) {
			return 0, fmt.Errorf("missing or invalid %q claim", ClaimBeaconID)
		}
		return int(id), nil
	case float32:
		if id <= 0 || id != float32(int(id)) {
			return 0, fmt.Errorf("missing or invalid %q claim", ClaimBeaconID)
		}
		return int(id), nil
	case int:
		if id <= 0 {
			return 0, fmt.Errorf("missing or invalid %q claim", ClaimBeaconID)
		}
		return id, nil
	case int64:
		if id <= 0 {
			return 0, fmt.Errorf("missing or invalid %q claim", ClaimBeaconID)
		}
		return int(id), nil
	case string:
		var parsed int
		if _, err := fmt.Sscanf(id, "%d", &parsed); err != nil || parsed <= 0 {
			return 0, fmt.Errorf("missing or invalid %q claim", ClaimBeaconID)
		}
		return parsed, nil
	default:
		return 0, fmt.Errorf("missing or invalid %q claim", ClaimBeaconID)
	}
}
