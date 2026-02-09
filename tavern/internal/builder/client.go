package builder

import (
	"context"
	"fmt"
	"log/slog"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	"realm.pub/tavern/internal/builder/builderpb"
)

// Run starts the builder process using the provided configuration.
// It connects to the configured upstream server and sends a ping request.
func Run(ctx context.Context, cfg *Config) error {
	slog.InfoContext(ctx, "builder started",
		"supported_targets", cfg.SupportedTargets,
		"upstream", cfg.Upstream,
	)

	conn, err := grpc.NewClient(cfg.Upstream,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		return fmt.Errorf("failed to connect to upstream %q: %w", cfg.Upstream, err)
	}
	defer conn.Close()

	client := builderpb.NewBuilderClient(conn)
	_, err = client.Ping(ctx, &builderpb.PingRequest{})
	if err != nil {
		return fmt.Errorf("failed to ping upstream: %w", err)
	}

	slog.InfoContext(ctx, "successfully pinged upstream", "upstream", cfg.Upstream)

	// Wait for context cancellation
	<-ctx.Done()
	return ctx.Err()
}
