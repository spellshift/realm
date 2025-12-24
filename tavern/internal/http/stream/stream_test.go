package stream

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gocloud.dev/pubsub"
	_ "gocloud.dev/pubsub/mempubsub"
)

func TestStream_SendMessage(t *testing.T) {
	t.Parallel()
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	topicName := fmt.Sprintf("mem://stream-test-send-%d", time.Now().UnixNano())
	topic, err := pubsub.OpenTopic(ctx, topicName)
	require.NoError(t, err)
	defer topic.Shutdown(ctx)
	sub, err := pubsub.OpenSubscription(ctx, topicName)
	require.NoError(t, err)
	defer sub.Shutdown(ctx)

	mux := NewMux(topic, sub)
	stream := New("test-stream")

	// Send a message
	err = stream.SendMessage(ctx, &pubsub.Message{Body: []byte("test message")}, mux)
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
		// Send messages out of order
		stream.processOneMessage(ctx, &pubsub.Message{
			Body: []byte("message 2"),
			Metadata: map[string]string{
				"id":               "ordering-stream",
				metadataOrderKey:   "test-key",
				metadataOrderIndex: "2",
			},
		})
		stream.processOneMessage(ctx, &pubsub.Message{
			Body: []byte("message 1"),
			Metadata: map[string]string{
				"id":               "ordering-stream",
				metadataOrderKey:   "test-key",
				metadataOrderIndex: "1",
			},
		})
		stream.processOneMessage(ctx, &pubsub.Message{
			Body: []byte("message 0"),
			Metadata: map[string]string{
				"id":               "ordering-stream",
				metadataOrderKey:   "test-key",
				metadataOrderIndex: "0",
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
