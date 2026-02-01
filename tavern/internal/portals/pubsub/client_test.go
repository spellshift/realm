package pubsub_test

import (
	"context"
	"testing"
	"time"

	"cloud.google.com/go/pubsub/v2"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	xpubsub "realm.pub/tavern/internal/portals/pubsub"
)

func TestInMemoryClient(t *testing.T) {
	ctx := context.Background()

	client, err := xpubsub.NewClient(ctx, xpubsub.WithInMemoryDriver(
		xpubsub.WithPublishSettings(pubsub.PublishSettings{
			ByteThreshold: 1,
		}),
	))
	require.NoError(t, err)
	defer client.Close()

	topicID := "test-topic"
	subID := "test-sub"

	// 1. Ensure Topic
	publisher, err := client.EnsureTopic(ctx, topicID)
	require.NoError(t, err)
	require.NotNil(t, publisher)

	// 2. Ensure Subscription
	subscriber, err := client.EnsureSubscription(ctx, subID, topicID)
	require.NoError(t, err)
	require.NotNil(t, subscriber)

	// 3. Publish
	res := publisher.Publish(ctx, &pubsub.Message{
		Data: []byte("hello world"),
		Attributes: map[string]string{
			"key": "val",
		},
	})
	id, err := res.Get(ctx)
	require.NoError(t, err)
	assert.NotEmpty(t, id)

	// 4. Receive
	received := make(chan *pubsub.Message, 1)
	ctxCancel, cancel := context.WithCancel(ctx)
	defer cancel()

	go func() {
		err := subscriber.Receive(ctxCancel, func(ctx context.Context, msg *pubsub.Message) {
			select {
			case received <- msg:
			case <-ctx.Done():
			}
			msg.Ack()
		})
		if err != nil {
			// context canceled is expected
		}
	}()

	select {
	case msg := <-received:
		assert.Equal(t, "hello world", string(msg.Data))
		assert.Equal(t, "val", msg.Attributes["key"])
	case <-time.After(5 * time.Second):
		t.Fatal("timed out waiting for message")
	}
}
