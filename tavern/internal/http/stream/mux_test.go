package stream_test

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gocloud.dev/pubsub"
	_ "gocloud.dev/pubsub/mempubsub"
	"realm.pub/tavern/internal/http/stream"
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
	mux := stream.NewMux(topic, sub)
	go mux.Start(ctx)

	// Create and Register Streams
	stream1 := stream.New("stream1")
	stream2 := stream.New("stream2")

	mux.Register(stream1)
	defer mux.Unregister(stream1)
	mux.Register(stream2)
	defer mux.Unregister(stream2)

	// Give the mux a moment to register the streams
	time.Sleep(50 * time.Millisecond)

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
	case <-time.After(5 * time.Second):
		t.Fatal("stream1 did not receive message in time")
	}

	select {
	case msg2 := <-stream2.Messages():
		assert.Equal(t, "hello stream 2", string(msg2.Body))
	case <-time.After(5 * time.Second):
		t.Fatal("stream2 did not receive message in time")
	}
}

func TestMuxHistory(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	// Setup Topic and Subscription
	topic, err := pubsub.OpenTopic(ctx, "mem://mux-history-test")
	require.NoError(t, err)
	defer topic.Shutdown(ctx)
	sub, err := pubsub.OpenSubscription(ctx, "mem://mux-history-test")
	require.NoError(t, err)
	defer sub.Shutdown(ctx)

	// Create Mux with small history size
	mux := stream.NewMux(topic, sub, stream.WithHistorySize(10))
	go mux.Start(ctx)

	// Create monitor stream to ensure messages are processed
	monitor := stream.New("history_stream")
	mux.Register(monitor)
	defer mux.Unregister(monitor)

	// Send some messages
	// Total 15 bytes. Buffer size 10. Last 10 bytes should be kept.
	messages := []string{"12345", "67890", "ABCDE"}
	for _, m := range messages {
		err = topic.Send(ctx, &pubsub.Message{
			Body:     []byte(m),
			Metadata: map[string]string{"id": "history_stream"},
		})
		require.NoError(t, err)

		// Wait for monitor to receive it
		select {
		case msg := <-monitor.Messages():
			assert.Equal(t, m, string(msg.Body))
		case <-time.After(1 * time.Second):
			t.Fatal("monitor did not receive message in time")
		}
	}

	// Register a new stream
	s := stream.New("history_stream")
	mux.Register(s)
	defer mux.Unregister(s)

	// Send SYNC message to trigger registration loop in Mux
	err = topic.Send(ctx, &pubsub.Message{
		Body:     []byte("SYNC"),
		Metadata: map[string]string{"id": "history_stream"},
	})
	require.NoError(t, err)

	// Expect history message immediately
	// "12345" (5) + "67890" (5) = 10.
	// "ABCDE" (5). Total 15.
	// Last 10 bytes: "67890ABCDE"
	select {
	case msg := <-s.Messages():
		assert.Equal(t, "67890ABCDE", string(msg.Body))
	case <-time.After(1 * time.Second):
		t.Fatal("stream did not receive history message in time")
	}

	// Expect SYNC message
	select {
	case msg := <-s.Messages():
		assert.Equal(t, "SYNC", string(msg.Body))
	case <-time.After(1 * time.Second):
		t.Fatal("stream did not receive new message in time")
	}
}
