package mux

import (
	"testing"

	"gocloud.dev/pubsub"
	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/portals/portalpb"
)

// BenchmarkDispatch measures the performance of processing a message and dispatching it
// to a large number of subscribers.
func BenchmarkDispatch(b *testing.B) {
	// Setup Mux
	m := New(nil, WithHistorySize(1024))

	streamID := "bench-stream"
	numSubs := 100 // 100 subscribers for the same stream

	// Register subscribers
	for i := 0; i < numSubs; i++ {
		// Subscribe returns a channel. We need to drain it to prevent blocking (if blocking was used)
		// or just let it fill and drop (if backpressure is used).
		// Since we want to measure dispatch cost, we should try to keep channels drained
		// or accept that they will fill and "default" case will be hit.
		// "Backpressure: ... drop the message ... do not block".
		// So if channels fill, the cost is checking select default.
		// If channels are empty, the cost is sending.
		// We want to measure the "send" path primarily.
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
		m.process(msg)
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
		m.process(msg)
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
		m.process(msg)
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
