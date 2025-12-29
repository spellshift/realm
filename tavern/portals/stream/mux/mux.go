package mux

import (
	"context"
	"fmt"
	"log"
	"sync"
	"sync/atomic"

	"gocloud.dev/pubsub"
	"golang.org/x/sync/errgroup"
	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/portals/portalpb"
)

const defaultBufferSize = 64

// Subscriber receives Motes for a specific stream.
type Subscriber struct {
	ch chan<- *portalpb.Mote
}

// Mux dispatches PubSub messages to dynamic stream subscribers.
type Mux struct {
	mu           sync.RWMutex
	subs         []*pubsub.Subscription
	topics       map[string]*pubsub.Topic
	subscribers  map[string][]Subscriber

	// history is a global circular buffer of recent messages.
	history      []*portalpb.Mote
	historyIndex int // Next write position
	historySize  int

	bufferSize   int
	droppedCount atomic.Uint64
}

// Option configures the Mux.
type Option func(*Mux)

// WithHistorySize sets the size of the global circular buffer for history.
func WithHistorySize(size int) Option {
	return func(m *Mux) {
		m.historySize = size
	}
}

// WithBufferSize sets the channel buffer size for new subscribers.
func WithBufferSize(size int) Option {
	return func(m *Mux) {
		m.bufferSize = size
	}
}

// WithTopic adds a PubSub topic for optimistic writes with a given name.
func WithTopic(name string, topic *pubsub.Topic) Option {
	return func(m *Mux) {
		m.topics[name] = topic
	}
}

// WithSubscription adds a PubSub subscription to listen on.
func WithSubscription(sub *pubsub.Subscription) Option {
	return func(m *Mux) {
		m.subs = append(m.subs, sub)
	}
}

// New creates a new Mux.
func New(opts ...Option) *Mux {
	m := &Mux{
		topics:      make(map[string]*pubsub.Topic),
		subscribers: make(map[string][]Subscriber),
		bufferSize:  defaultBufferSize,
	}
	for _, opt := range opts {
		opt(m)
	}
	if m.historySize > 0 {
		m.history = make([]*portalpb.Mote, m.historySize)
	}
	return m
}

// Subscribe adds a subscriber for the given streamID.
// If enableHistory is true, recent matching messages from the global buffer are replayed immediately.
// Returns a read-only channel and a cancel function to unsubscribe.
func (m *Mux) Subscribe(streamID string, enableHistory bool) (<-chan *portalpb.Mote, func()) {
	// Use configured buffer size
	ch := make(chan *portalpb.Mote, m.bufferSize)
	sub := Subscriber{ch: ch}

	m.mu.Lock()
	defer m.mu.Unlock()

	// Replay history if enabled
	if enableHistory && m.historySize > 0 {
		// Iterate from oldest to newest in the ring buffer
		for i := 0; i < m.historySize; i++ {
			idx := (m.historyIndex + i) % m.historySize
			mote := m.history[idx]
			if mote != nil && mote.StreamId == streamID {
				select {
				case ch <- mote:
				default:
					// Drop if channel full during replay
				}
			}
		}
	}

	m.subscribers[streamID] = append(m.subscribers[streamID], sub)

	cancel := func() {
		m.mu.Lock()
		defer m.mu.Unlock()
		subs := m.subscribers[streamID]
		for i, s := range subs {
			if s.ch == ch {
				// Remove by swapping with last
				last := len(subs) - 1
				subs[i] = subs[last]
				// Avoid memory leak by zeroing the moved element
				subs[last] = Subscriber{}
				m.subscribers[streamID] = subs[:last]
				close(ch)
				if len(m.subscribers[streamID]) == 0 {
					delete(m.subscribers, streamID)
				}
				return
			}
		}
	}

	return ch, cancel
}

// Publish optimistically dispatches the mote to subscribers and then publishes it to the named PubSub topic.
func (m *Mux) Publish(ctx context.Context, topicName string, mote *portalpb.Mote) error {
	topic, ok := m.topics[topicName]
	if !ok {
		return fmt.Errorf("pubsub topic %q not found", topicName)
	}

	// 1. Optimistic dispatch (local)
	m.dispatch(mote)

	// 2. Publish to PubSub (remote)
	body, err := proto.Marshal(mote)
	if err != nil {
		return fmt.Errorf("marshal failed: %w", err)
	}

	return topic.Send(ctx, &pubsub.Message{
		Body: body,
	})
}

// DroppedCount returns the total number of dropped messages.
func (m *Mux) DroppedCount() uint64 {
	return m.droppedCount.Load()
}

// Run starts the dispatch loop(s). It blocks until context is done or error occurs.
func (m *Mux) Run(ctx context.Context) error {
	g, ctx := errgroup.WithContext(ctx)

	for _, sub := range m.subs {
		sub := sub // capture loop var
		g.Go(func() error {
			for {
				msg, err := sub.Receive(ctx)
				if err != nil {
					return err
				}

				m.processMessage(msg)
				msg.Ack()
			}
		})
	}

	return g.Wait()
}

// processMessage unmarshals the PubSub message and calls dispatch.
func (m *Mux) processMessage(msg *pubsub.Message) {
	var mote portalpb.Mote
	if err := proto.Unmarshal(msg.Body, &mote); err != nil {
		log.Printf("Mux: unmarshal failed: %v", err)
		return
	}
	m.dispatch(&mote)
}

// dispatch updates history and sends the mote to subscribers.
func (m *Mux) dispatch(mote *portalpb.Mote) {
	// 1. Update history (Write Lock)
	if m.historySize > 0 {
		m.mu.Lock()
		m.history[m.historyIndex] = mote
		m.historyIndex = (m.historyIndex + 1) % m.historySize
		m.mu.Unlock()
	}

	// 2. Dispatch to subscribers (Read Lock)
	m.mu.RLock()
	defer m.mu.RUnlock()

	subs := m.subscribers[mote.StreamId]
	for _, s := range subs {
		select {
		case s.ch <- mote:
		default:
			// Backpressure: drop message
			newCount := m.droppedCount.Add(1)
			if newCount == 1 || newCount%1000 == 0 {
				log.Printf("Mux: dropped %d messages (stream: %s)", newCount, mote.StreamId)
			}
		}
	}
}
