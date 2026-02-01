package pubsub

import (
	"context"
	"fmt"
	"log/slog"
	"time"

	"cloud.google.com/go/pubsub/v2"
	"cloud.google.com/go/pubsub/v2/apiv1/pubsubpb"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/portals/portalpb"
)

// GCPOption defines an option for configuring the GCP driver.
type GCPOption func(*gcpDriver)

// WithGCPDriver configures the Client to use Google Cloud Pub/Sub as its Driver.
func WithGCPDriver(serverID string, client *pubsub.Client, options ...GCPOption) Option {
	return func(c *Client) {
		drv := &gcpDriver{
			serverID: serverID,
			GCP:      client,
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
	// Create the topic if it doesn't exist
	_, err := drv.GCP.TopicAdminClient.CreateTopic(ctx, &pubsubpb.Topic{
		Name: topic,
	})
	if err != nil && status.Code(err) != codes.AlreadyExists {
		return nil, fmt.Errorf("failed to create topic: %w", err)
	}
	publisher := drv.GCP.Publisher(topic)
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
		return nil, fmt.Errorf("failed to publish keepalive message: %w", err)
	}
	return gcpPublishAdapter, nil
}

// EnsureSubscriber creates the subscription if it doesn't exist and then returns a Subscriber for the topic.
func (drv *gcpDriver) EnsureSubscriber(ctx context.Context, topic, subscription string) (Subscriber, error) {
	_, err := drv.GCP.SubscriptionAdminClient.CreateSubscription(ctx, &pubsubpb.Subscription{
		Name:             subscription,
		Topic:            topic,
		ExpirationPolicy: &drv.expirationPolicy,
	})
	if err != nil && status.Code(err) != codes.AlreadyExists {
		return nil, fmt.Errorf("failed to create subscription: %w", err)
	}

	subscriber := drv.GCP.Subscriber(subscription)
	subscriber.ReceiveSettings = drv.receiveSettings
	return &gcpSubscriber{Subscriber: subscriber}, nil
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
		return nil
	}
}

type gcpSubscriber struct {
	*pubsub.Subscriber
}

func (s *gcpSubscriber) Receive(ctx context.Context, f func(context.Context, *portalpb.Mote)) error {
	return s.Subscriber.Receive(ctx, func(ctx context.Context, msg *pubsub.Message) {
		// Immediately acknowledge the message to prevent redelivery.
		msg.Ack()

		var mote portalpb.Mote
		if err := proto.Unmarshal(msg.Data, &mote); err != nil {
			slog.Error("subscriber failed to unmarshal mote", "err", err)
			return
		}
		f(ctx, &mote)
	})
}
