package main

import (
	"crypto/tls"
	"fmt"
	"net/url"
	"strings"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
)

func connect(upstream string) (*grpc.ClientConn, error) {
	// Default to http if no scheme set
	if !strings.HasPrefix(upstream, "https://") && !strings.HasPrefix(upstream, "http://") {
		upstream = fmt.Sprintf("http://%s", upstream)
	}

	// Parse host:port to determine if TLS should be used
	url, err := url.Parse(upstream)
	if err != nil {
		return nil, fmt.Errorf("failed to parse upstream address: %v", err)
	}

	// Default to TLS on 443
	tc := credentials.NewTLS(&tls.Config{})

	// If scheme is http or unset, use insecure credentials and default to port 80
	if url.Scheme == "http" || url.Scheme == "" {
		tc = insecure.NewCredentials()
	}

	conn, err := grpc.NewClient(url.Host, grpc.WithTransportCredentials(tc), grpc.WithWriteBufferSize(maxBuffSize), grpc.WithReadBufferSize(maxBuffSize))
	if err != nil {
		return nil, fmt.Errorf("failed to connect to upstream: %w", err)
	}

	return conn, nil
}
