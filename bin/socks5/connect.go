package main

import (
	"crypto/tls"
	"fmt"
	"net"
	"net/url"
	"strings"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
)

// normalizeUpstreamURL ensures upstream has an explicit HTTP(S) scheme.
// When no scheme is provided, loopback hosts (localhost, 127.0.0.0/8, ::1)
// default to http://, while all other hosts default to https:// so credentials
// are not silently transmitted in cleartext.
func normalizeUpstreamURL(upstream string) string {
	upstream = strings.TrimSpace(upstream)
	lower := strings.ToLower(upstream)
	if strings.HasPrefix(lower, "http://") || strings.HasPrefix(lower, "https://") {
		return upstream
	}

	hostPort := upstream
	suffix := ""
	if idx := strings.IndexAny(hostPort, "/?#"); idx != -1 {
		suffix = hostPort[idx:]
		hostPort = hostPort[:idx]
	}

	host := hostPort
	if h, _, err := net.SplitHostPort(hostPort); err == nil {
		host = h
	} else {
		host = strings.Trim(hostPort, "[]")
		if ip := net.ParseIP(host); ip != nil && strings.Contains(host, ":") && !strings.HasPrefix(hostPort, "[") {
			hostPort = "[" + host + "]"
		}
	}

	scheme := "https"
	if isLoopbackHost(host) {
		scheme = "http"
	}
	return fmt.Sprintf("%s://%s%s", scheme, hostPort, suffix)
}

func isLoopbackHost(host string) bool {
	if strings.EqualFold(host, "localhost") {
		return true
	}
	ip := net.ParseIP(host)
	return ip != nil && ip.IsLoopback()
}

func parseUpstream(upstream string) (*url.URL, credentials.TransportCredentials, error) {
	normalized := normalizeUpstreamURL(upstream)

	// Parse host:port to determine if TLS should be used
	u, err := url.Parse(normalized)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to parse upstream address: %v", err)
	}

	// Default to TLS
	tc := credentials.NewTLS(&tls.Config{})

	// Only use insecure credentials when scheme is http (explicit or loopback default)
	if u.Scheme == "http" || u.Scheme == "" {
		tc = insecure.NewCredentials()
	}

	return u, tc, nil
}

func connect(upstream string) (*grpc.ClientConn, error) {
	u, tc, err := parseUpstream(upstream)
	if err != nil {
		return nil, err
	}

	conn, err := grpc.NewClient(u.Host, grpc.WithTransportCredentials(tc), grpc.WithWriteBufferSize(maxBuffSize), grpc.WithReadBufferSize(maxBuffSize))
	if err != nil {
		return nil, fmt.Errorf("failed to connect to upstream: %w", err)
	}

	return conn, nil
}
