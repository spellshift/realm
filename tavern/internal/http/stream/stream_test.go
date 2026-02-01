package stream

import (
	"context"
	"fmt"
	"math/rand"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/xpubsub"
)

func newTopicName(base string) string {
	return fmt.Sprintf("%s-%d", base, rand.Int())
}

func TestStream_SendMessage(t *testing.T) {
	t.Parallel()
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	client, err := xpubsub.NewClient(ctx, "test-project", true)
	require.NoError(t, err)
	defer client.Close()

	topicName := newTopicName("stream-test-send")
	require.NoError(t, client.EnsureTopic(ctx, topicName))

	// In mempubsub, subscription name matters for connecting to topic?
	// xpubsub uses pstest, so we must link subscription to topic.
	subName := topicName + "-sub"
	require.NoError(t, client.EnsureSubscription(ctx, topicName, subName, 0))

	pub := client.NewPublisher(topicName)
	defer pub.Close()
	sub := client.NewSubscriber(subName)
	defer sub.Close()

	mux := NewMux(pub, sub)
	stream := New("test-stream")

	// Send a message
	err = stream.SendMessage(ctx, &xpubsub.Message{Body: []byte("test message")}, mux)
	require.NoError(t, err)

	// Receive the message from the subscription to verify
	msg, err := sub.Receive(ctx)
	require.NoError(t, err)
	assert.Equal(t, "test message", string(msg.Body))
	assert.Equal(t, "test-stream", msg.Metadata["id"])
	msg.Ack()
}

func TestStream_MessageOrdering(t *testing.T) {
	t.Parallel()
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	stream := New("ordering-stream")
	go func() {
		// Send 0 first to anchor the stream
		stream.processOneMessage(ctx, &xpubsub.Message{
			Body: []byte("message 0"),
			Metadata: map[string]string{
				"id":               "ordering-stream",
				metadataOrderKey:   "test-key",
				metadataOrderIndex: "0",
			},
		})
		// Then send 2 (buffered)
		stream.processOneMessage(ctx, &xpubsub.Message{
			Body: []byte("message 2"),
			Metadata: map[string]string{
				"id":               "ordering-stream",
				metadataOrderKey:   "test-key",
				metadataOrderIndex: "2",
			},
		})
		// Then send 1 (fills gap)
		stream.processOneMessage(ctx, &xpubsub.Message{
			Body: []byte("message 1"),
			Metadata: map[string]string{
				"id":               "ordering-stream",
				metadataOrderKey:   "test-key",
				metadataOrderIndex: "1",
			},
		})
	}()

	// Expect to receive messages in order
	for i := 0; i < 3; i++ {
		select {
		case msg := <-stream.Messages():
			expectedBody := fmt.Sprintf("message %d", i)
			assert.Equal(t, expectedBody, string(msg.Body))
		case <-time.After(1 * time.Second):
			t.Fatalf("timed out waiting for message %d", i)
		}
	}
}

func TestStream_LateJoin(t *testing.T) {
	t.Parallel()
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	stream := New("late-join-stream")
	go func() {
		// Simulate joining late: receive message 10 first
		stream.processOneMessage(ctx, &xpubsub.Message{
			Body: []byte("message 10"),
			Metadata: map[string]string{
				"id":               "late-join-stream",
				metadataOrderKey:   "test-key",
				metadataOrderIndex: "10",
			},
		})
	}()

	select {
	case msg := <-stream.Messages():
		assert.Equal(t, "message 10", string(msg.Body))
	case <-ctx.Done():
		t.Fatal("timed out waiting for message 10 (late join failed)")
	}
}

func TestStream_Close(t *testing.T) {
	t.Parallel()
	stream := New("closable-stream")
	go func() {
		time.Sleep(10 * time.Millisecond)
		stream.Close()
	}()

	_, ok := <-stream.Messages()
	assert.False(t, ok, "channel should be closed")
}
