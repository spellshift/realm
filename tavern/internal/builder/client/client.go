package client

import (
	"context"
	"fmt"
	"log/slog"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"realm.pub/tavern/internal/builder/builderpb"
)

// Client wraps a Builder gRPC client connection.
type Client struct {
	conn    *grpc.ClientConn
	Builder builderpb.BuilderClient
}

// New creates a new Builder client connected to the given upstream address.
func New(ctx context.Context, upstream string) (*Client, error) {
	conn, err := grpc.NewClient(
		upstream,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to builder server: %w", err)
	}

	slog.InfoContext(ctx, "connected to builder server", "upstream", upstream)

	return &Client{
		conn:    conn,
		Builder: builderpb.NewBuilderClient(conn),
	}, nil
}

// Close closes the underlying gRPC connection.
func (c *Client) Close() error {
	return c.conn.Close()
}
