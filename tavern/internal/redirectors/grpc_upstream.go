package redirectors

import (
	"context"
	"crypto/tls"
	"fmt"
	"net"
	"net/url"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
)

// ConnectToUpstream creates a gRPC client connection to the upstream address
func ConnectToUpstream(upstream string) (*grpc.ClientConn, error) {
	// Parse host:port to determine if TLS should be used
	url, err := url.Parse(upstream)
	if err != nil {
		return nil, fmt.Errorf("failed to parse upstream address: %v", err)
	}

	// Default to TLS on 443
	var (
		tc   = credentials.NewTLS(&tls.Config{})
		port = "443"
	)

	// If scheme is http, use insecure credentials and default to port 80
	if url.Scheme == "http" {
		port = "80"
		tc = insecure.NewCredentials()
	}

	// If port is specified, use it
	if url.Port() != "" {
		port = url.Port()
	}

	return grpc.NewClient(
		url.Host,
		grpc.WithTransportCredentials(tc),
		grpc.WithContextDialer(func(ctx context.Context, _ string) (net.Conn, error) {
			// Resolve using IPv4 only (A records, not AAAA records)
			ips, err := net.DefaultResolver.LookupIP(ctx, "ip4", url.Hostname())
			if err != nil {
				return nil, err
			}
			if len(ips) == 0 {
				return nil, fmt.Errorf("no IPv4 addresses found for %s", url.Hostname())
			}

			// Force IPv4 by using "tcp4" instead of "tcp"
			dialer := &net.Dialer{}
			tcpConn, err := dialer.DialContext(ctx, "tcp4", net.JoinHostPort(ips[0].String(), port))
			if err != nil {
				return nil, err
			}

			return tcpConn, nil
		}),
	)
}
