package c2

import (
	"context"
	"net"
	"testing"

	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/peer"
)

func TestGetClientIP(t *testing.T) {
	tests := []struct {
		name             string
		setupContext     func() context.Context
		expectedIP       string
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
