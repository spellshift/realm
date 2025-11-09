package stream

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gocloud.dev/pubsub"
	_ "gocloud.dev/pubsub/mempubsub"
)

func TestMux(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	// Setup Topic and Subscription
	topic, err := pubsub.OpenTopic(ctx, "mem://mux-test")
	require.NoError(t, err)
	defer topic.Shutdown(ctx)
	sub, err := pubsub.OpenSubscription(ctx, "mem://mux-test")
	require.NoError(t, err)
	defer sub.Shutdown(ctx)

	// Create Mux
	mux := NewMux(topic, sub)
	go mux.Start(ctx)

	// Create and Register Streams
	stream1 := New("stream1")
	stream2 := New("stream2")

	mux.Register(stream1)
	mux.Register(stream2)

	// Give the mux a moment to register the streams
	time.Sleep(10 * time.Millisecond)

	// Send a message for stream1
	err = topic.Send(ctx, &pubsub.Message{
		Body:     []byte("hello stream 1"),
		Metadata: map[string]string{"id": "stream1"},
	})
	require.NoError(t, err)

	// Send a message for stream2
	err = topic.Send(ctx, &pubsub.Message{
		Body:     []byte("hello stream 2"),
		Metadata: map[string]string{"id": "stream2"},
	})
	require.NoError(t, err)

	// Send a message with no id
	err = topic.Send(ctx, &pubsub.Message{
		Body: []byte("no id"),
	})
	require.NoError(t, err)

	// Assert messages are received by the correct stream
	select {
	case msg1 := <-stream1.Messages():
		assert.Equal(t, "hello stream 1", string(msg1.Body))
	case <-time.After(1 * time.Second):
		t.Fatal("stream1 did not receive message in time")
	}

	select {
	case msg2 := <-stream2.Messages():
		assert.Equal(t, "hello stream 2", string(msg2.Body))
	case <-time.After(1 * time.Second):
		t.Fatal("stream2 did not receive message in time")
	}

	// Unregister stream1
	mux.Unregister(stream1)

	// Give the mux a moment to unregister the stream
	time.Sleep(10 * time.Millisecond)

	// Send another message for stream1
	err = topic.Send(ctx, &pubsub.Message{
		Body:     []byte("goodbye stream 1"),
		Metadata: map[string]string{"id": "stream1"},
	})
	require.NoError(t, err)

	// Assert stream1 does not receive the message
	select {
	case <-stream1.Messages():
		t.Fatal("stream1 received message after being unregistered")
	case <-time.After(100 * time.Millisecond):
		// This is expected
	}
}
