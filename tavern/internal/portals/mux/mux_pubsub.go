package mux

import (
	"context"
	"fmt"
	"log/slog"
	"time"

	"realm.pub/tavern/internal/portals/pubsub"
	"realm.pub/tavern/portals/portalpb"
)

// Publish sends a message to the specified topic.
func (m *Mux) Publish(ctx context.Context, topicID string, mote *portalpb.Mote) error {
	// 1. Fast Path (Local)
	m.dispatch(topicID, mote)

	// 2. Slow Path (Global)
	// Add to history
	m.addToHistory(topicID, mote)

	// Metrics
	msgsPublished.WithLabelValues(topicID, "attempt").Inc()

	t, err := m.getTopic(ctx, topicID)
	if err != nil {
		msgsPublished.WithLabelValues(topicID, "error_topic_open").Inc()
		return fmt.Errorf("failed to open topic %s: %w", topicID, err)
	}

	// Send
	err = t.Publish(ctx, mote)
	if err != nil {
		msgsPublished.WithLabelValues(topicID, "error_send").Inc()
		return fmt.Errorf("failed to publish to topic %s: %w", topicID, err)
	}

	msgsPublished.WithLabelValues(topicID, "success").Inc()
	return nil
}

// SubOption defines an option for subscription.
type SubOption func(*subOptions)

type subOptions struct {
	replayHistory bool
}

// WithHistoryReplay enables history replay for a subscription.
func WithHistoryReplay() SubOption {
	return func(o *subOptions) {
		o.replayHistory = true
	}
}

// Subscribe creates a local subscription to the topic.
func (m *Mux) Subscribe(topicID string, opts ...SubOption) (<-chan *portalpb.Mote, func()) {
	options := subOptions{
		replayHistory: false,
	}
	for _, opt := range opts {
		opt(&options)
	}

	ch := make(chan *portalpb.Mote, m.subs.bufferSize)

	m.subs.Lock()
	m.subs.subs[topicID] = append(m.subs.subs[topicID], ch)
	m.subs.Unlock()

	// Replay history
	if options.replayHistory {
		m.history.RLock()
		if buf, ok := m.history.buffers[topicID]; ok {
			msgs := buf.Get()
			for _, msg := range msgs {
				select {
				case ch <- msg:
				default:
					slog.Warn("Subscriber channel full during history replay", "topic", topicID)
				}
			}
		}
		m.history.RUnlock()
	}

	cancel := func() {
		m.subs.Lock()
		defer m.subs.Unlock()

		subs := m.subs.subs[topicID]
		for i, c := range subs {
			if c == ch {
				// Remove from slice
				m.subs.subs[topicID] = append(subs[:i], subs[i+1:]...)
				close(ch)
				break
			}
		}
	}

	return ch, cancel
}

// dispatch sends the message to all local subscribers.
func (m *Mux) dispatch(topicID string, mote *portalpb.Mote) {
	m.subs.RLock()
	defer m.subs.RUnlock()

	subs := m.subs.subs[topicID]
	for _, ch := range subs {
		select {
		case ch <- mote:
		case <-time.After(100 * time.Millisecond):
			// Drop message if subscriber is slow
			slog.Warn("Dropping message for slow subscriber", "topic", topicID)
			msgsDropped.WithLabelValues(topicID).Inc()
		}
	}
}

// addToHistory adds a message to the history buffer for the topic.
func (m *Mux) addToHistory(topicID string, mote *portalpb.Mote) {
	m.history.Lock()
	defer m.history.Unlock()

	buf, ok := m.history.buffers[topicID]
	if !ok {
		buf = NewHistoryBuffer(m.history.size)
		m.history.buffers[topicID] = buf
	}
	buf.Add(mote)
}

func (m *Mux) receiveLoop(ctx context.Context, topicID string, sub pubsub.Subscriber) {
	err := sub.Receive(ctx, func(ctx context.Context, mote *portalpb.Mote) {
		msgsReceived.WithLabelValues(topicID).Inc()

		// Dispatch locally
		m.dispatch(topicID, mote)

		// Add to history (messages from others also go into history)
		m.addToHistory(topicID, mote)
	})
	if err != nil {
		// Context canceled or subscription closed
		if ctx.Err() != nil {
			return
		}
		slog.Error("Receive loop exited with error", "topic", topicID, "error", err)
	}
}

// getTopic returns a cached topic handle or opens a new one.
func (m *Mux) getTopic(ctx context.Context, topicID string) (pubsub.Publisher, error) {
	m.topics.RLock()
	t, ok := m.topics.topics[topicID]
	m.topics.RUnlock()
	if ok {
		return t, nil
	}

	m.topics.Lock()
	defer m.topics.Unlock()
	// Double check
	if t, ok := m.topics.topics[topicID]; ok {
		return t, nil
	}

	t, err := m.pubsub.EnsurePublisher(ctx, topicID)
	if err != nil {
		return nil, err
	}

	m.topics.topics[topicID] = t
	return t, nil
}

// openSubscription opens a subscription.
func (m *Mux) openSubscription(ctx context.Context, topicID, subID string) (pubsub.Subscriber, error) {
	return m.pubsub.EnsureSubscriber(ctx, topicID, subID)
}
