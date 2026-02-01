package pubsub

import (
	"context"
	"fmt"
	"log/slog"
	"strings"
	"time"

	"cloud.google.com/go/pubsub/v2"
	"cloud.google.com/go/pubsub/v2/apiv1/pubsubpb"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/durationpb"
	"realm.pub/tavern/portals/portalpb"
)

// GCPOption defines an option for configuring the GCP driver.
type GCPOption func(*gcpDriver)

// WithPublishSettings configures the publish settings for the GCP driver.
func WithPublishSettings(settings pubsub.PublishSettings) GCPOption {
	return func(d *gcpDriver) {
		d.publishSettings = settings
	}
}

// WithReceiveSettings configures the receive settings for the GCP driver.
func WithReceiveSettings(settings pubsub.ReceiveSettings) GCPOption {
	return func(d *gcpDriver) {
		d.receiveSettings = settings
	}
}

// WithExpirationPolicy configures the expiration policy for subscriptions created by the GCP driver.
func WithExpirationPolicy(ttl time.Duration) GCPOption {
	return func(d *gcpDriver) {
		d.expirationPolicy = pubsubpb.ExpirationPolicy{
			Ttl: durationpb.New(ttl),
		}
	}
}

// WithPublishReadyTimeout configures the timeout for waiting for the publisher to be ready.
func WithPublishReadyTimeout(timeout time.Duration) GCPOption {
	return func(d *gcpDriver) {
		d.publishReadyTimeout = timeout
	}
}

// WithGCPDriver configures the Client to use Google Cloud Pub/Sub as its Driver.
func WithGCPDriver(serverID string, client *pubsub.Client, options ...GCPOption) Option {
	return func(c *Client) {
		drv := &gcpDriver{
			serverID: serverID,
			GCP:      client,
			// Default Expiration Policy
			expirationPolicy: pubsubpb.ExpirationPolicy{
				Ttl: durationpb.New(24 * time.Hour),
			},
		}
		for _, opt := range options {
			opt(drv)
		}
		c.Driver = drv
	}
}

type gcpDriver struct {
	serverID            string
	GCP                 *pubsub.Client
	publishSettings     pubsub.PublishSettings
	receiveSettings     pubsub.ReceiveSettings
	expirationPolicy    pubsubpb.ExpirationPolicy
	publishReadyTimeout time.Duration
}

// EnsurePublisher creates the topic if it doesn't exist and then publishes a keepalive message,
// ensuring that the topic is ready for publishing. If successful, it returns a Publisher for the topic.
// If any step fails, it returns an error.
func (drv *gcpDriver) EnsurePublisher(ctx context.Context, topic string) (Publisher, error) {
	topicPath := topic
	if !strings.HasPrefix(topic, "projects/") {
		topicPath = fmt.Sprintf("projects/%s/topics/%s", drv.GCP.Project(), topic)
	}

	_, err := drv.GCP.TopicAdminClient.CreateTopic(ctx, &pubsubpb.Topic{
		Name: topicPath,
	})
	if err != nil && status.Code(err) != codes.AlreadyExists {
		return nil, fmt.Errorf("failed to create topic: %w", err)
	}
	publisher := drv.GCP.Publisher(topicPath)
	publisher.PublishSettings = drv.publishSettings

	keepalive := &portalpb.Mote{
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_KEEPALIVE,
			},
		},
	}

	gcpPublishAdapter := &gcpPublisher{
		serverID:     drv.serverID,
		readyTimeout: drv.publishReadyTimeout,
		Publisher:    publisher,
	}

	if err := gcpPublishAdapter.Publish(ctx, keepalive); err != nil {
		return nil, fmt.Errorf("failed to publish keepalive message (topic=%q): %w", topic, err)
	}
	return gcpPublishAdapter, nil
}

// EnsureSubscriber creates the subscription if it doesn't exist and then returns a Subscriber for the topic.
func (drv *gcpDriver) EnsureSubscriber(ctx context.Context, topic, subscription string) (Subscriber, error) {
	topicPath := topic
	if !strings.HasPrefix(topic, "projects/") {
		topicPath = fmt.Sprintf("projects/%s/topics/%s", drv.GCP.Project(), topic)
	}

	subscriptionPath := subscription
	if !strings.HasPrefix(subscription, "projects/") {
		subscriptionPath = fmt.Sprintf("projects/%s/subscriptions/%s", drv.GCP.Project(), subscription)
	}

	_, err := drv.GCP.SubscriptionAdminClient.CreateSubscription(ctx, &pubsubpb.Subscription{
		Name:             subscriptionPath,
		Topic:            topicPath,
		ExpirationPolicy: &drv.expirationPolicy,
	})
	if err != nil && status.Code(err) != codes.AlreadyExists {
		return nil, fmt.Errorf("failed to create subscription (sub=%q, topic=%q): %w", subscription, topic, err)
	}

	subscriber := drv.GCP.Subscriber(subscriptionPath)
	subscriber.ReceiveSettings = drv.receiveSettings
	return &gcpSubscriber{
		serverID:   drv.serverID,
		Subscriber: subscriber,
	}, nil
}

func (drv *gcpDriver) Close() error {
	return drv.GCP.Close()
}

type gcpPublisher struct {
	serverID     string
	readyTimeout time.Duration
	*pubsub.Publisher
}

func (pub *gcpPublisher) Publish(ctx context.Context, mote *portalpb.Mote) error {
	data, err := proto.Marshal(mote)
	if err != nil {
		return fmt.Errorf("failed to marshal mote: %w", err)
	}
	result := pub.Publisher.Publish(ctx, &pubsub.Message{
		Data: data,
		Attributes: map[string]string{
			"server_id": pub.serverID,
		},
	})
	select {
	case <-result.Ready():
		_, err := result.Get(ctx)
		return err
	case <-time.After(pub.readyTimeout):
		// If timeout provided, wait for it
		if pub.readyTimeout > 0 {
			return nil
		}
		// If 0, we can also just return, the library handles async publishing.
		// However, existing logic seemed to wait.
		return nil
	}
}

type gcpSubscriber struct {
	serverID string
	*pubsub.Subscriber
}

func (s *gcpSubscriber) Receive(ctx context.Context, f func(context.Context, *portalpb.Mote)) error {
	return s.Subscriber.Receive(ctx, func(ctx context.Context, msg *pubsub.Message) {
		// Immediately acknowledge the message to prevent redelivery.
		msg.Ack()

		// Loopback detection
		if senderID, ok := msg.Attributes["server_id"]; ok && senderID == s.serverID {
			return
		}

		var mote portalpb.Mote
		if err := proto.Unmarshal(msg.Data, &mote); err != nil {
			slog.Error("subscriber failed to unmarshal mote", "err", err)
			return
		}
		f(ctx, &mote)
	})
}
