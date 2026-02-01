package stream_test

import (
	"context"
	"fmt"
	"math/rand"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/xpubsub"
)

func newTopicName(base string) string {
	return fmt.Sprintf("%s-%d", base, rand.Int())
}

func TestMux(t *testing.T) {
	t.Parallel()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	client, err := xpubsub.NewClient(ctx, "test-project", true)
	require.NoError(t, err)
	defer client.Close()

	topicName := newTopicName("mux-test")
	subName := topicName + "-sub"
	require.NoError(t, client.EnsureTopic(ctx, topicName))
	require.NoError(t, client.EnsureSubscription(ctx, topicName, subName, 0))

	pub := client.NewPublisher(topicName)
	defer pub.Close()
	sub := client.NewSubscriber(subName)
	defer sub.Close()

	// Create Mux
	mux := stream.NewMux(pub, sub)
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
	err = pub.Publish(ctx, []byte("hello stream 1"), map[string]string{"id": "stream1"})
	require.NoError(t, err)

	// Send a message for stream2
	err = pub.Publish(ctx, []byte("hello stream 2"), map[string]string{"id": "stream2"})
	require.NoError(t, err)

	// Send a message with no id
	err = pub.Publish(ctx, []byte("no id"), nil)
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
	t.Parallel()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	client, err := xpubsub.NewClient(ctx, "test-project", true)
	require.NoError(t, err)
	defer client.Close()

	topicName := newTopicName("mux-history-test")
	subName := topicName + "-sub"
	require.NoError(t, client.EnsureTopic(ctx, topicName))
	require.NoError(t, client.EnsureSubscription(ctx, topicName, subName, 0))

	pub := client.NewPublisher(topicName)
	defer pub.Close()
	sub := client.NewSubscriber(subName)
	defer sub.Close()

	// Create Mux with small history size
	mux := stream.NewMux(pub, sub, stream.WithHistorySize(10))
	go mux.Start(ctx)

	// Create monitor stream to ensure messages are processed
	monitor := stream.New("history_stream")
	mux.Register(monitor)
	defer mux.Unregister(monitor)

	// Send some messages
	// Total 15 bytes. Buffer size 10. Last 10 bytes should be kept.
	messages := []string{"12345", "67890", "ABCDE"}
	for _, m := range messages {
		err = pub.Publish(ctx, []byte(m), map[string]string{"id": "history_stream"})
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
	err = pub.Publish(ctx, []byte("SYNC"), map[string]string{"id": "history_stream"})
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

func TestMuxHistoryOrdering(t *testing.T) {
	t.Parallel()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	client, err := xpubsub.NewClient(ctx, "test-project", true)
	require.NoError(t, err)
	defer client.Close()

	topicName := newTopicName("mux-history-ordering-test")
	subName := topicName + "-sub"
	require.NoError(t, client.EnsureTopic(ctx, topicName))
	require.NoError(t, client.EnsureSubscription(ctx, topicName, subName, 0))

	pub := client.NewPublisher(topicName)
	defer pub.Close()
	sub := client.NewSubscriber(subName)
	defer sub.Close()

	// Create Mux
	mux := stream.NewMux(pub, sub, stream.WithHistorySize(100))
	go mux.Start(ctx)

	// Create monitor stream to wait for processing
	monitor := stream.New("ordering_stream")
	mux.Register(monitor)
	defer mux.Unregister(monitor)

	// Send messages in an order that respects the new "Late Join" logic.
	// We must send the anchor (0) first so the stream knows where it starts.
	orderKey := "session1"

	// 1. Send Anchor (0)
	err = pub.Publish(ctx, []byte("A"), map[string]string{
		"id":          "ordering_stream",
		"order-key":   orderKey,
		"order-index": "0",
	})
	require.NoError(t, err)

	// Wait for Anchor to be processed to ensure stream initialization
	select {
	case msg := <-monitor.Messages():
		assert.Equal(t, "A", string(msg.Body))
	case <-time.After(1 * time.Second):
		t.Fatal("monitor did not receive anchor message in time")
	}

	// 2. Send C (2) - Out of order
	err = pub.Publish(ctx, []byte("C"), map[string]string{
		"id":          "ordering_stream",
		"order-key":   orderKey,
		"order-index": "2",
	})
	require.NoError(t, err)

	// 3. Send B (1) - Fills gap
	// Sleep to ensure C arrives at Mux before B (testing the buffer logic)
	time.Sleep(10 * time.Millisecond)
	err = pub.Publish(ctx, []byte("B"), map[string]string{
		"id":          "ordering_stream",
		"order-key":   orderKey,
		"order-index": "1",
	})
	require.NoError(t, err)

	// Wait for monitor to receive remaining messages.
	received := ""
	for i := 0; i < 2; i++ {
		select {
		case msg := <-monitor.Messages():
			received += string(msg.Body)
		case <-time.After(1 * time.Second):
			t.Fatal("monitor did not receive message in time")
		}
	}
	assert.Equal(t, "BC", received)

	// Now register a new stream to check history
	s := stream.New("ordering_stream")
	mux.Register(s)
	defer mux.Unregister(s)

	// Send SYNC
	err = pub.Publish(ctx, []byte("SYNC"), map[string]string{"id": "ordering_stream"})
	require.NoError(t, err)

	// Expect history: "ABC"
	select {
	case msg := <-s.Messages():
		assert.Equal(t, "ABC", string(msg.Body))
	case <-time.After(1 * time.Second):
		t.Fatal("stream did not receive history message in time")
	}
}
