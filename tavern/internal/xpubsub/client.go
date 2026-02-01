package xpubsub

import (
	"context"
	"fmt"
	"sync"
	"time"

	"cloud.google.com/go/pubsub/v2"
	"cloud.google.com/go/pubsub/v2/apiv1/pubsubpb"
	"cloud.google.com/go/pubsub/v2/pstest"
	"google.golang.org/api/option"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/durationpb"
)

// Publisher interface
type Publisher interface {
	Publish(ctx context.Context, body []byte, metadata map[string]string) error
	Close()
	SetSettings(settings pubsub.PublishSettings)
}

// Subscriber interface
type Subscriber interface {
	Receive(ctx context.Context) (*Message, error)
	Close()
	SetSettings(settings pubsub.ReceiveSettings)
}

// Client wraps pubsub.Client
type Client struct {
	client       *pubsub.Client
	pstestServer *pstest.Server
}

// NewClient creates a new Pub/Sub client.
// If useInMem is true, it starts a local pstest server and connects to it.
func NewClient(ctx context.Context, projectID string, useInMem bool) (*Client, error) {
	if useInMem {
		srv := pstest.NewServer()

		conn, err := grpc.Dial(srv.Addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
		if err != nil {
			srv.Close()
			return nil, fmt.Errorf("failed to dial pstest server: %w", err)
		}

		client, err := pubsub.NewClient(ctx, projectID, option.WithGRPCConn(conn), option.WithEndpoint(srv.Addr), option.WithoutAuthentication())
		if err != nil {
			srv.Close()
			return nil, fmt.Errorf("failed to create pubsub client: %w", err)
		}

		return &Client{client: client, pstestServer: srv}, nil
	}

	client, err := pubsub.NewClient(ctx, projectID)
	if err != nil {
		return nil, err
	}
	return &Client{client: client}, nil
}

func (c *Client) Close() error {
	var err error
	if c.client != nil {
		err = c.client.Close()
	}
	if c.pstestServer != nil {
		c.pstestServer.Close()
	}
	return err
}

// Native returns the underlying pubsub client.
func (c *Client) Native() *pubsub.Client {
	return c.client
}

// EnsureTopic checks if a topic exists, creating it if not.
func (c *Client) EnsureTopic(ctx context.Context, topicID string) error {
	fullTopicName := fmt.Sprintf("projects/%s/topics/%s", c.client.Project(), topicID)
	_, err := c.client.TopicAdminClient.GetTopic(ctx, &pubsubpb.GetTopicRequest{Topic: fullTopicName})
	if err == nil {
		return nil
	}
	if status.Code(err) != codes.NotFound {
		return fmt.Errorf("failed to check topic existence: %w", err)
	}

	_, err = c.client.TopicAdminClient.CreateTopic(ctx, &pubsubpb.Topic{Name: fullTopicName})
	if err != nil {
		if status.Code(err) == codes.AlreadyExists {
			return nil
		}
		return fmt.Errorf("failed to create topic: %w", err)
	}
	return nil
}

// EnsureSubscription checks if a subscription exists, creating it if not.
func (c *Client) EnsureSubscription(ctx context.Context, topicID, subID string, ttl time.Duration) error {
	fullSubName := fmt.Sprintf("projects/%s/subscriptions/%s", c.client.Project(), subID)
	_, err := c.client.SubscriptionAdminClient.GetSubscription(ctx, &pubsubpb.GetSubscriptionRequest{Subscription: fullSubName})
	if err == nil {
		return nil
	}
	if status.Code(err) != codes.NotFound {
		return fmt.Errorf("failed to check subscription existence: %w", err)
	}

	fullTopicName := fmt.Sprintf("projects/%s/topics/%s", c.client.Project(), topicID)

	subConfig := &pubsubpb.Subscription{
		Name:  fullSubName,
		Topic: fullTopicName,
	}

	if ttl > 0 {
		subConfig.ExpirationPolicy = &pubsubpb.ExpirationPolicy{
			Ttl: durationpb.New(ttl),
		}
	}

	_, err = c.client.SubscriptionAdminClient.CreateSubscription(ctx, subConfig)
	if err != nil {
		if status.Code(err) == codes.AlreadyExists {
			return nil
		}
		return fmt.Errorf("failed to create subscription: %w", err)
	}
	return nil
}

// NewPublisher creates a new publisher for the given topic.
func (c *Client) NewPublisher(topicID string) Publisher {
	return &gcpPublisher{topic: c.client.Publisher(topicID)}
}

// NewSubscriber creates a new subscriber for the given subscription.
func (c *Client) NewSubscriber(subID string) Subscriber {
	return &gcpSubscriber{
		sub: c.client.Subscriber(subID),
		ch:  make(chan *Message, 100),
	}
}

// gcpPublisher implements Publisher
type gcpPublisher struct {
	topic *pubsub.Publisher
}

func (p *gcpPublisher) SetSettings(settings pubsub.PublishSettings) {
	p.topic.PublishSettings = settings
}

func (p *gcpPublisher) Publish(ctx context.Context, body []byte, metadata map[string]string) error {
	res := p.topic.Publish(ctx, &pubsub.Message{
		Data:       body,
		Attributes: metadata,
	})
	_, err := res.Get(ctx)
	return err
}

func (p *gcpPublisher) Close() {
	p.topic.Stop()
}

// gcpSubscriber implements Subscriber
type gcpSubscriber struct {
	sub    *pubsub.Subscriber
	ch     chan *Message
	cancel context.CancelFunc
	once   sync.Once
}

func (s *gcpSubscriber) SetSettings(settings pubsub.ReceiveSettings) {
	s.sub.ReceiveSettings = settings
}

func (s *gcpSubscriber) start() {
	ctx, cancel := context.WithCancel(context.Background())
	s.cancel = cancel
	go func() {
		// v2 Receive blocks
		_ = s.sub.Receive(ctx, func(ctx context.Context, msg *pubsub.Message) {
			wrapped := &Message{
				Body:       msg.Data,
				Metadata:   msg.Attributes,
				LoggableID: msg.ID,
				Ack:        msg.Ack,
				Nack:       msg.Nack,
			}
			select {
			case s.ch <- wrapped:
			case <-ctx.Done():
				msg.Nack()
			}
		})
		close(s.ch)
	}()
}

func (s *gcpSubscriber) Receive(ctx context.Context) (*Message, error) {
	s.once.Do(s.start)

	select {
	case msg, ok := <-s.ch:
		if !ok {
			return nil, fmt.Errorf("subscription closed")
		}
		return msg, nil
	case <-ctx.Done():
		return nil, ctx.Err()
	}
}

func (s *gcpSubscriber) Close() {
	if s.cancel != nil {
		s.cancel()
	}
}
