package stream_test

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"gocloud.dev/pubsub"
	_ "gocloud.dev/pubsub/mempubsub"
	"realm.pub/tavern/internal/http/stream"
)

func TestPreventPubSubColdStarts_ValidInterval(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	// Create a mock topic and subscription.
	topic, err := pubsub.OpenTopic(ctx, "mem://valid")
	if err != nil {
		t.Fatalf("Failed to open topic: %v", err)
	}
	defer topic.Shutdown(ctx)
	sub, err := pubsub.OpenSubscription(ctx, "mem://valid")
	if err != nil {
		t.Fatalf("Failed to open subscription: %v", err)
	}
	defer sub.Shutdown(ctx)

	go stream.PreventPubSubColdStarts(ctx, 50*time.Millisecond, "mem://valid", "mem://valid")

	// Expect to receive a message
	msg, err := sub.Receive(ctx)
	assert.NoError(t, err)
	assert.NotNil(t, msg)
	if msg != nil {
		assert.Equal(t, "noop", msg.Metadata["id"])
		msg.Ack()
	}
}

func TestPreventPubSubColdStarts_ZeroInterval(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 1*time.Second)
	defer cancel()

	topic, err := pubsub.OpenTopic(ctx, "mem://zero")
	if err != nil {
		t.Fatalf("Failed to open topic: %v", err)
	}
	defer topic.Shutdown(ctx)
	sub, err := pubsub.OpenSubscription(ctx, "mem://zero")
	if err != nil {
		t.Fatalf("Failed to open subscription: %v", err)
	}
	defer sub.Shutdown(ctx)

	go stream.PreventPubSubColdStarts(ctx, 0, "mem://zero", "mem://zero")

	// Expect to not receive a message and for the context to timeout
	_, err = sub.Receive(ctx)
	assert.Error(t, err)
	assert.Equal(t, context.DeadlineExceeded, err)
}

func TestPreventPubSubColdStarts_SubMillisecondInterval(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	topic, err := pubsub.OpenTopic(ctx, "mem://sub")
	if err != nil {
		t.Fatalf("Failed to open topic: %v", err)
	}
	defer topic.Shutdown(ctx)
	sub, err := pubsub.OpenSubscription(ctx, "mem://sub")
	if err != nil {
		t.Fatalf("Failed to open subscription: %v", err)
	}
	defer sub.Shutdown(ctx)

	go stream.PreventPubSubColdStarts(ctx, 1*time.Microsecond, "mem://sub", "mem://sub")

	// Expect to receive a message
	msg, err := sub.Receive(ctx)
	assert.NoError(t, err)
	assert.NotNil(t, msg)
	if msg != nil {
		assert.Equal(t, "noop", msg.Metadata["id"])
		msg.Ack()
	}
}
