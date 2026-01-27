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
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/peer"
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
	jwtPublicKey	 ed25519.PublicKey

	c2pb.UnimplementedC2Server
}

func New(graph *ent.Client, mux *stream.Mux, portalMux *mux.Mux, jwtPublicKey ed25519.PublicKey, jwtPrivateKey ed25519.PrivateKey) *Server {
	return &Server{
		MaxFileChunkSize: 1024 * 1024, // 1 MB
		graph:            graph,
		mux:              mux,
		portalMux:        portalMux,
		jwtPrivateKey:    jwtPrivateKey,
		jwtPublicKey:	  jwtPublicKey,
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

// generateTaskJWT creates a signed JWT token containing the beacon ID
func (srv *Server) generateTaskJWT() (string, error) {
	claims := jwt.MapClaims{
		"iat": jwt.NewNumericDate(time.Now()),
		"exp": jwt.NewNumericDate(time.Now().Add(1 * time.Hour)), // Token expires in 1 hour
	}

	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	signedToken, err := token.SignedString(srv.jwtPrivateKey)
	if err != nil {
		return "", fmt.Errorf("failed to sign JWT: %w", err)
	}

	return signedToken, nil
}

func (srv *Server) ValidateJWT(jwttoken string) error {
    token, err := jwt.Parse(jwttoken, func(token *jwt.Token) (any, error) {
        // 1. Verify the signing method is EdDSA
        if _, ok := token.Method.(*jwt.SigningMethodEd25519); !ok {
			// TODO: Uncomment with imixv1 delete
            // return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
			slog.Warn(fmt.Sprintf("unexpected signing method: %v", token.Header["alg"]))
		}
        // 2. Return the PUBLIC key for verification
        return srv.jwtPublicKey, nil
    })

    if err != nil || !token.Valid {
		// TODO: Uncomment with imixv1 delete
        // return status.Errorf(codes.PermissionDenied, "invalid token: %v", err)
		slog.Warn(fmt.Sprintf("invalid token: %v", err))
		return nil
    }

	slog.Info(fmt.Sprintf("recieved valid JWT: %s", jwttoken))
    return nil
}
