package pubsub

import (
	"context"
	"fmt"

	"cloud.google.com/go/pubsub/v2"
	"cloud.google.com/go/pubsub/v2/apiv1/pubsubpb"
	"cloud.google.com/go/pubsub/v2/pstest"
	"google.golang.org/api/option"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/durationpb"
	"time"
)

// Client wraps the GCP PubSub client and provides helper methods.
type Client struct {
	Client *pubsub.Client

	// Internal fields for in-memory driver
	pstestServer *pstest.Server

	// Config settings to apply to new topics/subscriptions
	publishSettings *pubsub.PublishSettings
	receiveSettings *pubsub.ReceiveSettings
}

// config holds the configuration for the client.
type config struct {
	projectID       string
	useInMemory     bool
	publishSettings *pubsub.PublishSettings
	receiveSettings *pubsub.ReceiveSettings
	clientOptions   []option.ClientOption
}

// Option configures the Client.
type Option func(*config)

// DriverOption configures the driver specific settings.
type DriverOption func(*config)

// WithPublishSettings sets the publish settings for the client.
func WithPublishSettings(s pubsub.PublishSettings) DriverOption {
	return func(c *config) {
		c.publishSettings = &s
	}
}

// WithReceiveSettings sets the receive settings for the client.
func WithReceiveSettings(s pubsub.ReceiveSettings) DriverOption {
	return func(c *config) {
		c.receiveSettings = &s
	}
}

// WithInMemoryDriver configures the client to use an in-memory driver (pstest).
func WithInMemoryDriver(opts ...DriverOption) Option {
	return func(c *config) {
		c.useInMemory = true
		c.projectID = "test-project" // Default project ID for in-memory
		for _, opt := range opts {
			opt(c)
		}
	}
}

// WithGCPDriver configures the client to use the GCP driver.
func WithGCPDriver(projectID string, opts ...DriverOption) Option {
	return func(c *config) {
		c.useInMemory = false
		c.projectID = projectID
		for _, opt := range opts {
			opt(c)
		}
	}
}

// WithClientOptions adds extra options to the underlying PubSub client creation.
func WithClientOptions(opts ...option.ClientOption) DriverOption {
	return func(c *config) {
		c.clientOptions = append(c.clientOptions, opts...)
	}
}

// NewClient creates a new PubSub client.
func NewClient(ctx context.Context, opts ...Option) (*Client, error) {
	cfg := &config{}
	for _, opt := range opts {
		opt(cfg)
	}

	c := &Client{
		publishSettings: cfg.publishSettings,
		receiveSettings: cfg.receiveSettings,
	}

	var clientOpts []option.ClientOption
	clientOpts = append(clientOpts, cfg.clientOptions...)

	if cfg.useInMemory {
		// Start in-memory server
		srv := pstest.NewServer()
		c.pstestServer = srv

		// Connect to the in-memory server
		conn, err := grpc.NewClient(srv.Addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
		if err != nil {
			return nil, fmt.Errorf("failed to dial in-memory server: %w", err)
		}
		clientOpts = append(clientOpts, option.WithGRPCConn(conn))
	}

	pcl, err := pubsub.NewClient(ctx, cfg.projectID, clientOpts...)
	if err != nil {
		if c.pstestServer != nil {
			c.pstestServer.Close()
		}
		return nil, fmt.Errorf("failed to create pubsub client: %w", err)
	}
	c.Client = pcl

	return c, nil
}

// Close closes the underlying client and the in-memory server if it exists.
func (c *Client) Close() error {
	var err error
	if c.Client != nil {
		if e := c.Client.Close(); e != nil {
			err = e
		}
	}
	if c.pstestServer != nil {
		if e := c.pstestServer.Close(); e != nil {
			if err == nil {
				err = e
			} else {
				err = fmt.Errorf("%v; %v", err, e)
			}
		}
	}
	return err
}

// EnsureTopic checks if a topic exists and creates it if it doesn't.
// It returns a Publisher for the topic with configured settings.
func (c *Client) EnsureTopic(ctx context.Context, topicID string) (*pubsub.Publisher, error) {
	fullTopicName := fmt.Sprintf("projects/%s/topics/%s", c.Client.Project(), topicID)

	_, err := c.Client.TopicAdminClient.GetTopic(ctx, &pubsubpb.GetTopicRequest{Topic: fullTopicName})
	if err != nil {
		if status.Code(err) == codes.NotFound {
			_, err = c.Client.TopicAdminClient.CreateTopic(ctx, &pubsubpb.Topic{Name: fullTopicName})
			if err != nil && status.Code(err) != codes.AlreadyExists {
				return nil, fmt.Errorf("failed to create topic: %w", err)
			}
		} else {
			return nil, fmt.Errorf("failed to check topic existence: %w", err)
		}
	}

	// Create Publisher
	p := c.Client.Publisher(topicID)

	// Apply settings
	if c.publishSettings != nil {
		p.PublishSettings = *c.publishSettings
	}

	return p, nil
}

// EnsureSubscription checks if a subscription exists and creates it if it doesn't.
// It returns a Subscriber for the subscription with configured settings.
func (c *Client) EnsureSubscription(ctx context.Context, subID string, topicID string) (*pubsub.Subscriber, error) {
	fullSubName := fmt.Sprintf("projects/%s/subscriptions/%s", c.Client.Project(), subID)
	fullTopicName := fmt.Sprintf("projects/%s/topics/%s", c.Client.Project(), topicID)

	_, err := c.Client.SubscriptionAdminClient.GetSubscription(ctx, &pubsubpb.GetSubscriptionRequest{Subscription: fullSubName})
	if err != nil {
		if status.Code(err) == codes.NotFound {
			// Ensure topic exists first? The caller should ensure it, but for safety in ensureSub logic:
			// But CreateSubscription requires topic.
			// Let's assume topic exists or try to create sub will fail/succeed.
			// Actually EnsureSubscription usually implies ensuring the topic dependency if logical?
			// But sticking to strict responsibility, we assume topic exists or CreateSubscription will fail if topic doesn't exist.
			// However, for user convenience, we might want to ensure topic?
			// Let's call EnsureTopic.
			if _, err := c.EnsureTopic(ctx, topicID); err != nil {
				return nil, fmt.Errorf("failed to ensure topic before creating sub: %w", err)
			}

			_, err = c.Client.SubscriptionAdminClient.CreateSubscription(ctx, &pubsubpb.Subscription{
				Name:  fullSubName,
				Topic: fullTopicName,
				ExpirationPolicy: &pubsubpb.ExpirationPolicy{
					Ttl: durationpb.New(24 * time.Hour),
				},
			})
			if err != nil && status.Code(err) != codes.AlreadyExists {
				return nil, fmt.Errorf("failed to create subscription: %w", err)
			}
		} else {
			return nil, fmt.Errorf("failed to check subscription existence: %w", err)
		}
	}

	// Create Subscriber
	s := c.Client.Subscriber(subID)

	// Apply settings
	if c.receiveSettings != nil {
		s.ReceiveSettings = *c.receiveSettings
	}

	return s, nil
}
