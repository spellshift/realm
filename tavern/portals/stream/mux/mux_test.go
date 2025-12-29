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
// and that duplicates from PubSub loopback are handled (by the subscriber logic in this case).
func TestOptimisticWrite(t *testing.T) {
	ctx := context.Background()
	topic := mempubsub.NewTopic()
	sub := mempubsub.NewSubscription(topic, time.Second)

	m := New(sub, WithTopic(topic), WithHistorySize(10))

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
	err := m.Publish(ctx, mote)
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
	// Since mempubsub is fast, it should arrive shortly.
	select {
	case received := <-ch:
		if received.SeqId != 100 {
			t.Errorf("Expected duplicate SeqId 100, got %d", received.SeqId)
		} else {
			t.Log("Received duplicate message from PubSub (subscriber should drop this)")
		}
	case <-time.After(1 * time.Second):
		// It's possible mempubsub is slow or configured differently, but usually it loops back.
		// If we don't get it, it might mean Mux filtered it (which it shouldn't in this impl)
		// or mempubsub didn't deliver.
		t.Fatal("Timeout waiting for loopback message")
	}

	// Ensure no more messages
	select {
	case msg := <-ch:
		t.Errorf("Received unexpected message: %v", msg)
	default:
	}
}

// BenchmarkDispatch measures the performance of processing a message and dispatching it
// to a large number of subscribers.
func BenchmarkDispatch(b *testing.B) {
	// Setup Mux
	m := New(nil, WithHistorySize(1024))

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
	m := New(nil, WithHistorySize(1024))
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
	m := New(nil, WithHistorySize(historySize))
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
	// But Publish calls topic.Send.
	// We need a mock topic that is fast. mempubsub is fast.
	topic := mempubsub.NewTopic()
	sub := mempubsub.NewSubscription(topic, time.Second)
	m := New(sub, WithTopic(topic), WithHistorySize(0)) // No history for this bench

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
		m.Publish(ctx, mote)
	}
}
