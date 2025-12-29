package mux

import (
	"context"
	"testing"
	"time"

	"gocloud.dev/pubsub"
	"gocloud.dev/pubsub/mempubsub"
	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/portals/portalpb"
)

// TestOptimisticWrite verifies that Publish sends messages to subscribers immediately
// and that duplicates from PubSub loopback are handled.
func TestOptimisticWrite(t *testing.T) {
	ctx := context.Background()
	topic := mempubsub.NewTopic()
	sub := mempubsub.NewSubscription(topic, time.Second)

	topicName := "test-topic"
	m := New(WithSubscription(sub), WithTopic(topicName, topic), WithHistorySize(10))

	// Start Mux in background
	go func() {
		if err := m.Run(ctx); err != nil {
			// Context canceled is expected
		}
	}()
	defer sub.Shutdown(ctx)
	defer topic.Shutdown(ctx)

	streamID := "test-optimistic"
	ch, cancel := m.Subscribe(streamID, false)
	defer cancel()

	mote := &portalpb.Mote{
		StreamId: streamID,
		SeqId:    100,
		Payload:  &portalpb.Mote_Bytes{Bytes: &portalpb.BytesPayload{Data: []byte("payload")}},
	}

	// Publish optimistically
	err := m.Publish(ctx, topicName, mote)
	if err != nil {
		t.Fatalf("Publish failed: %v", err)
	}

	// 1. Receive Optimistic Message
	select {
	case received := <-ch:
		if received.SeqId != 100 {
			t.Errorf("Expected SeqId 100, got %d", received.SeqId)
		}
	case <-time.After(100 * time.Millisecond):
		t.Fatal("Timeout waiting for optimistic message")
	}

	// 2. Receive PubSub Loopback Message (Duplicate)
	select {
	case received := <-ch:
		if received.SeqId != 100 {
			t.Errorf("Expected duplicate SeqId 100, got %d", received.SeqId)
		} else {
			t.Log("Received duplicate message from PubSub (subscriber should drop this)")
		}
	case <-time.After(1 * time.Second):
		t.Fatal("Timeout waiting for loopback message")
	}
}

// TestMultipleSubscriptions verifies that Mux can receive from multiple subscriptions.
func TestMultipleSubscriptions(t *testing.T) {
	ctx := context.Background()

	// Create two topic/sub pairs
	topic1 := mempubsub.NewTopic()
	sub1 := mempubsub.NewSubscription(topic1, time.Second)
	defer topic1.Shutdown(ctx)
	defer sub1.Shutdown(ctx)

	topic2 := mempubsub.NewTopic()
	sub2 := mempubsub.NewSubscription(topic2, time.Second)
	defer topic2.Shutdown(ctx)
	defer sub2.Shutdown(ctx)

	m := New(WithSubscription(sub1), WithSubscription(sub2))

	go func() {
		if err := m.Run(ctx); err != nil {
		}
	}()

	streamID := "multi-sub-stream"
	ch, cancel := m.Subscribe(streamID, false)
	defer cancel()

	// Send to topic 1
	mote1 := &portalpb.Mote{StreamId: streamID, SeqId: 1}
	body1, _ := proto.Marshal(mote1)
	topic1.Send(ctx, &pubsub.Message{Body: body1})

	// Send to topic 2
	mote2 := &portalpb.Mote{StreamId: streamID, SeqId: 2}
	body2, _ := proto.Marshal(mote2)
	topic2.Send(ctx, &pubsub.Message{Body: body2})

	// Verify we receive both
	count := 0
	for i := 0; i < 2; i++ {
		select {
		case msg := <-ch:
			if msg.SeqId == 1 || msg.SeqId == 2 {
				count++
			}
		case <-time.After(time.Second):
			t.Fatal("Timeout waiting for messages from multiple subs")
		}
	}
	if count != 2 {
		t.Errorf("Expected 2 messages, got %d", count)
	}
}

// TestMultipleTopics verifies publishing to specific topics.
func TestMultipleTopics(t *testing.T) {
	ctx := context.Background()
	topicA := mempubsub.NewTopic()
	topicB := mempubsub.NewTopic()
	defer topicA.Shutdown(ctx)
	defer topicB.Shutdown(ctx)

	// We'll use subscriptions just to verify where the message went
	subA := mempubsub.NewSubscription(topicA, time.Second)
	subB := mempubsub.NewSubscription(topicB, time.Second)
	defer subA.Shutdown(ctx)
	defer subB.Shutdown(ctx)

	m := New(WithTopic("A", topicA), WithTopic("B", topicB))

	mote := &portalpb.Mote{StreamId: "stream", SeqId: 99}

	// Publish to A
	if err := m.Publish(ctx, "A", mote); err != nil {
		t.Fatalf("Publish to A failed: %v", err)
	}

	// Verify subA received it
	msgA, err := subA.Receive(ctx)
	if err != nil {
		t.Fatal(err)
	}
	msgA.Ack()
	if len(msgA.Body) == 0 {
		t.Error("Received empty body on A")
	}

	// Verify subB did NOT receive it (timeout expected)
	// mempubsub receive blocks.
	ctxTimeout, cancel := context.WithTimeout(ctx, 100*time.Millisecond)
	defer cancel()
	if _, err := subB.Receive(ctxTimeout); err == nil {
		t.Error("Unexpected message on B")
	}

	// Publish to B
	if err := m.Publish(ctx, "B", mote); err != nil {
		t.Fatalf("Publish to B failed: %v", err)
	}
	msgB, err := subB.Receive(ctx)
	if err != nil {
		t.Fatal(err)
	}
	msgB.Ack()
}

// BenchmarkDispatch measures the performance of processing a message and dispatching it
// to a large number of subscribers.
func BenchmarkDispatch(b *testing.B) {
	// Setup Mux
	m := New(WithHistorySize(1024))

	streamID := "bench-stream"
	numSubs := 100 // 100 subscribers for the same stream

	// Register subscribers
	for i := 0; i < numSubs; i++ {
		ch, _ := m.Subscribe(streamID, false)

		// Start a goroutine to drain channel so we benchmark the "send" path
		go func(c <-chan *portalpb.Mote) {
			for range c {
			}
		}(ch)
	}

	// Prepare a message
	mote := &portalpb.Mote{
		StreamId: streamID,
		SeqId:    12345,
		Payload:  &portalpb.Mote_Bytes{Bytes: &portalpb.BytesPayload{Data: []byte("benchmark payload")}},
	}
	body, err := proto.Marshal(mote)
	if err != nil {
		b.Fatalf("failed to marshal mote: %v", err)
	}
	msg := &pubsub.Message{Body: body}

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		m.processMessage(msg)
	}
}

// BenchmarkDispatchNoSubscribers measures overhead when no subscribers exist.
func BenchmarkDispatchNoSubscribers(b *testing.B) {
	m := New(WithHistorySize(1024))
	streamID := "bench-stream"

	mote := &portalpb.Mote{
		StreamId: streamID,
		SeqId:    1,
		Payload:  &portalpb.Mote_Bytes{Bytes: &portalpb.BytesPayload{Data: []byte("payload")}},
	}
	body, _ := proto.Marshal(mote)
	msg := &pubsub.Message{Body: body}

	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		m.processMessage(msg)
	}
}

// BenchmarkDispatchHistoryReplay benchmarks the Subscribe cost with history replay.
func BenchmarkDispatchHistoryReplay(b *testing.B) {
	historySize := 1024
	m := New(WithHistorySize(historySize))
	streamID := "bench-stream"

	// Fill history
	mote := &portalpb.Mote{
		StreamId: streamID,
		SeqId:    1,
		Payload:  &portalpb.Mote_Bytes{Bytes: &portalpb.BytesPayload{Data: []byte("payload")}},
	}
	body, _ := proto.Marshal(mote)
	msg := &pubsub.Message{Body: body}

	for i := 0; i < historySize; i++ {
		m.processMessage(msg)
	}

	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		// Subscribe with history
		_, cancel := m.Subscribe(streamID, true)
		b.StopTimer()
		cancel() // Cleanup
		b.StartTimer()
	}
}

// Mock WaitGroup for subscribers
func BenchmarkOptimisticWrite(b *testing.B) {
	// Setup Mux with dummy topic/sub (we won't use them fully since we test Publish)
	topic := mempubsub.NewTopic()
	sub := mempubsub.NewSubscription(topic, time.Second)
	topicName := "bench"
	m := New(WithSubscription(sub), WithTopic(topicName, topic), WithHistorySize(0)) // No history for this bench

	streamID := "bench-optimistic"
	// One subscriber
	ch, _ := m.Subscribe(streamID, false)
	go func() {
		for range ch {}
	}()

	mote := &portalpb.Mote{
		StreamId: streamID,
		SeqId:    1,
		Payload:  &portalpb.Mote_Bytes{Bytes: &portalpb.BytesPayload{Data: []byte("payload")}},
	}
	ctx := context.Background()

	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		m.Publish(ctx, topicName, mote)
	}
}
