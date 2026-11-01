package c2

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"encoding/json"
	"fmt"
	"log/slog"
	"net"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/peer"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestGetClientIP(t *testing.T) {
	tests := []struct {
		name         string
		setupContext func() context.Context
		expectedIP   string
	}{
		{
			name: "X-Forwarded-For_Only",
			setupContext: func() context.Context {
				ctx := context.Background()
				md := metadata.New(map[string]string{
					"x-forwarded-for": "203.0.113.42",
				})
				return metadata.NewIncomingContext(ctx, md)
			},
			expectedIP: "203.0.113.42",
		},
		{
			name: "X-Redirected-For_With_X-Forwarded-For",
			setupContext: func() context.Context {
				ctx := context.Background()
				md := metadata.New(map[string]string{
					"x-forwarded-for":  "203.0.113.42",
					"x-redirected-for": "198.51.100.99",
				})
				return metadata.NewIncomingContext(ctx, md)
			},
			expectedIP: "198.51.100.99",
		},
		{
			name: "Neither_Header_Set_Uses_Peer_IP",
			setupContext: func() context.Context {
				ctx := context.Background()
				p := &peer.Peer{
					Addr: &net.TCPAddr{
						IP:   net.ParseIP("1.1.1.1"),
						Port: 12345,
					},
				}
				return peer.NewContext(ctx, p)
			},
			expectedIP: "1.1.1.1",
		},
		{
			name: "X-Forwarded-For_With_Multiple_IPs",
			setupContext: func() context.Context {
				ctx := context.Background()
				md := metadata.New(map[string]string{
					"x-forwarded-for": "203.0.113.42, 198.51.100.1, 192.0.2.5",
				})
				return metadata.NewIncomingContext(ctx, md)
			},
			expectedIP: "203.0.113.42",
		},
		{
			name: "X-Forwarded-For_With_Whitespace",
			setupContext: func() context.Context {
				ctx := context.Background()
				md := metadata.New(map[string]string{
					"x-forwarded-for": "  203.0.113.42  ",
				})
				return metadata.NewIncomingContext(ctx, md)
			},
			expectedIP: "203.0.113.42",
		},
		{
			name: "X-Redirected-For_Precedence_Over_Peer",
			setupContext: func() context.Context {
				ctx := context.Background()
				p := &peer.Peer{
					Addr: &net.TCPAddr{
						IP:   net.ParseIP("1.1.1.1"),
						Port: 12345,
					},
				}
				ctx = peer.NewContext(ctx, p)
				md := metadata.New(map[string]string{
					"x-redirected-for": "198.51.100.99",
				})
				return metadata.NewIncomingContext(ctx, md)
			},
			expectedIP: "198.51.100.99",
		},
		{
			name: "Invalid_X-Forwarded-For_Fallback_To_Peer",
			setupContext: func() context.Context {
				ctx := context.Background()
				p := &peer.Peer{
					Addr: &net.TCPAddr{
						IP:   net.ParseIP("1.1.1.1"),
						Port: 12345,
					},
				}
				ctx = peer.NewContext(ctx, p)
				md := metadata.New(map[string]string{
					"x-forwarded-for": "invalid-ip-address",
				})
				return metadata.NewIncomingContext(ctx, md)
			},
			expectedIP: "1.1.1.1",
		},
		{
			name: "No_Metadata_No_Peer_Returns_Unknown",
			setupContext: func() context.Context {
				return context.Background()
			},
			expectedIP: "unknown",
		},
		{
			name: "Malformed_X-Redirected-For_Returns_As_Is",
			setupContext: func() context.Context {
				ctx := context.Background()
				p := &peer.Peer{
					Addr: &net.TCPAddr{
						IP:   net.ParseIP("1.1.1.1"),
						Port: 12345,
					},
				}
				ctx = peer.NewContext(ctx, p)
				md := metadata.New(map[string]string{
					"x-redirected-for": "not-an-ip",
				})
				return metadata.NewIncomingContext(ctx, md)
			},
			expectedIP: "1.1.1.1",
		},
		{
			name: "Malformed_X-Forwarded-For_Without_Peer_Returns_Unknown",
			setupContext: func() context.Context {
				ctx := context.Background()
				md := metadata.New(map[string]string{
					"x-forwarded-for": "not-an-ip",
				})
				return metadata.NewIncomingContext(ctx, md)
			},
			expectedIP: "unknown",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx := tt.setupContext()
			result := GetClientIP(ctx)
			if result != tt.expectedIP {
				t.Errorf("GetClientIP() = %v, want %v", result, tt.expectedIP)
			}
		})
	}
}

func TestJWTValidate(t *testing.T) {
	pubkey, privKey, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		slog.Error(fmt.Sprintf("failed to generate ED25519 key pair: %v\n", err))
	}
	srv := Server{
		jwtPrivateKey: privKey,
		jwtPublicKey:  pubkey,
	}

	claims := jwt.MapClaims{
		ClaimBeaconID: 1234,
		"iat":         time.Now().Unix(),
		"exp":         time.Now().Add(1 * time.Hour).Unix(), // Token expires in 1 hour
	}

	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, claims)
	tokenStr, err := token.SignedString(privKey)
	if err != nil {
		t.Fatalf("failed to sign JWT: %v", err)
	}
	// Verify bound token succeeds and returns the beacon ID
	beaconID, err := srv.ValidateJWT(tokenStr)
	assert.Nil(t, err)
	assert.Equal(t, 1234, beaconID)

	// Verify unbound (legacy) token without beacon_id is rejected
	legacyClaims := jwt.MapClaims{
		"iat": time.Now().Unix(),
		"exp": time.Now().Add(1 * time.Hour).Unix(),
	}
	legacyToken := jwt.NewWithClaims(jwt.SigningMethodEdDSA, legacyClaims)
	legacyStr, err := legacyToken.SignedString(privKey)
	if err != nil {
		t.Fatalf("failed to sign JWT: %v", err)
	}
	_, err = srv.ValidateJWT(legacyStr)
	assert.NotNil(t, err)

	for _, bad := range []json.Number{"1.5", "9223372036854775808"} {
		badToken := jwt.NewWithClaims(jwt.SigningMethodEdDSA, jwt.MapClaims{
			ClaimBeaconID: bad,
			"exp":         time.Now().Add(time.Hour).Unix(),
		})
		badStr, err := badToken.SignedString(privKey)
		assert.NoError(t, err)
		_, err = srv.ValidateJWT(badStr)
		assert.Error(t, err, "expected rejection for beacon_id claim %s", bad)
	}
}

func TestLoadShellTaskForBeacon(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()
	host := graph.Host.Create().SetIdentifier("host").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	beacon := graph.Beacon.Create().SetHost(host).SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)
	foreign := graph.Beacon.Create().SetHost(host).SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SaveX(ctx)
	user := graph.User.Create().SetOauthID("user").SetName("user").SetPhotoURL("photo").SaveX(ctx)
	shell := graph.Shell.Create().SetBeacon(beacon).SetOwner(user).SetData([]byte{}).SaveX(ctx)
	shellTask := graph.ShellTask.Create().SetShell(shell).SetCreator(user).SetInput("id").SetStreamID("stream").SetSequenceID(1).SaveX(ctx)
	srv := Server{graph: graph}

	_, err := srv.loadShellTaskForBeacon(ctx, shellTask.ID, beacon.ID)
	require.NoError(t, err)
	_, err = srv.loadShellTaskForBeacon(ctx, shellTask.ID, foreign.ID)
	require.True(t, ent.IsNotFound(err))
}
