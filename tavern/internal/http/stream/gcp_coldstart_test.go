package stream_test

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/xpubsub"
)

func TestPreventPubSubColdStarts_ValidInterval(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	client, err := xpubsub.NewClient(ctx, "test-project", true)
	require.NoError(t, err)
	defer client.Close()

	topicName := "valid"
	subName := "valid-sub"
	require.NoError(t, client.EnsureTopic(ctx, topicName))
	require.NoError(t, client.EnsureSubscription(ctx, topicName, subName, 0))

	sub := client.NewSubscriber(subName)
	defer sub.Close()

	go stream.PreventPubSubColdStarts(ctx, client, 50*time.Millisecond, topicName, topicName)

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

	client, err := xpubsub.NewClient(ctx, "test-project", true)
	require.NoError(t, err)
	defer client.Close()

	topicName := "zero"
	subName := "zero-sub"
	require.NoError(t, client.EnsureTopic(ctx, topicName))
	require.NoError(t, client.EnsureSubscription(ctx, topicName, subName, 0))

	sub := client.NewSubscriber(subName)
	defer sub.Close()

	go stream.PreventPubSubColdStarts(ctx, client, 0, topicName, topicName)

	// Expect to not receive a message and for the context to timeout
	_, err = sub.Receive(ctx)
	assert.Error(t, err)
	assert.Equal(t, context.DeadlineExceeded, err)
}

func TestPreventPubSubColdStarts_SubMillisecondInterval(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	client, err := xpubsub.NewClient(ctx, "test-project", true)
	require.NoError(t, err)
	defer client.Close()

	topicName := "sub"
	subName := "sub-sub"
	require.NoError(t, client.EnsureTopic(ctx, topicName))
	require.NoError(t, client.EnsureSubscription(ctx, topicName, subName, 0))

	sub := client.NewSubscriber(subName)
	defer sub.Close()

	go stream.PreventPubSubColdStarts(ctx, client, 1*time.Microsecond, topicName, topicName)

	// Expect to receive a message
	msg, err := sub.Receive(ctx)
	assert.NoError(t, err)
	assert.NotNil(t, msg)
	if msg != nil {
		assert.Equal(t, "noop", msg.Metadata["id"])
		msg.Ack()
	}
}
