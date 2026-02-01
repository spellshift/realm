package pubsub

import (
	"context"
	"fmt"
	"strings"
	"time"

	"cloud.google.com/go/pubsub/v2"
	"cloud.google.com/go/pubsub/v2/apiv1/pubsubpb"
	"cloud.google.com/go/pubsub/v2/pstest"
	"google.golang.org/api/option"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/protobuf/types/known/durationpb"
)

// WithInMemoryDriver configures the Client to use an in-memory driver (pstest).
// It accepts GCPOptions to configure the underlying in-memory GCP driver.
func WithInMemoryDriver(options ...GCPOption) Option {
	return func(c *Client) {
		// Start a new pstest server
		srv := pstest.NewServer()

		// Connect to the server
		conn, err := grpc.NewClient(srv.Addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
		if err != nil {
			panic(fmt.Errorf("failed to dial pstest server: %w", err))
		}

		// Create the pubsub client
		client, err := pubsub.NewClient(context.Background(), "in-memory-project", option.WithGRPCConn(conn))
		if err != nil {
			panic(fmt.Errorf("failed to create pubsub client: %w", err))
		}

		drv := &memDriver{
			server: srv,
			conn:   conn,
			gcp: &gcpDriver{
				serverID: "in-memory",
				GCP:      client,
				// Default Expiration Policy
				expirationPolicy: pubsubpb.ExpirationPolicy{
					Ttl: durationpb.New(24 * time.Hour),
				},
			},
		}

		for _, opt := range options {
			opt(drv.gcp)
		}
		c.Driver = drv
	}
}

type memDriver struct {
	server *pstest.Server
	conn   *grpc.ClientConn
	gcp    *gcpDriver
}

// EnsurePublisher creates and returns an in-memory Publisher for the specified topic.
func (drv *memDriver) EnsurePublisher(ctx context.Context, topic string) (Publisher, error) {
	return drv.gcp.EnsurePublisher(ctx, topic)
}

// EnsureSubscriber creates and returns an in-memory Subscriber for the specified topic and subscription.
func (drv *memDriver) EnsureSubscriber(ctx context.Context, topic, subscription string) (Subscriber, error) {
	return drv.gcp.EnsureSubscriber(ctx, topic, subscription)
}

// Close closes the in-memory driver and releases resources.
func (drv *memDriver) Close() error {
	var errs []string
	if err := drv.gcp.Close(); err != nil {
		errs = append(errs, fmt.Sprintf("gcp close: %v", err))
	}
	if err := drv.conn.Close(); err != nil {
		errs = append(errs, fmt.Sprintf("conn close: %v", err))
	}
	if err := drv.server.Close(); err != nil {
		errs = append(errs, fmt.Sprintf("server close: %v", err))
	}
	if len(errs) > 0 {
		return fmt.Errorf("failed to close memDriver: %s", strings.Join(errs, "; "))
	}
	return nil
}
