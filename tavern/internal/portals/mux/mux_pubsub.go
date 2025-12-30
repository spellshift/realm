package mux

import (
	"context"
	"fmt"
	"log/slog"

	"gocloud.dev/pubsub"
	"google.golang.org/protobuf/proto"
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

	// Serialize
	data, err := proto.Marshal(mote)
	if err != nil {
		msgsPublished.WithLabelValues(topicID, "error_marshal").Inc()
		return fmt.Errorf("failed to marshal mote: %w", err)
	}

	t, err := m.getTopic(ctx, topicID)
	if err != nil {
		msgsPublished.WithLabelValues(topicID, "error_topic_open").Inc()
		return fmt.Errorf("failed to open topic %s: %w", topicID, err)
	}
	// Do not shutdown topic here as we are caching it.

	// Send
	err = t.Send(ctx, &pubsub.Message{
		Body: data,
		Metadata: map[string]string{
			"sender_id": m.serverID,
		},
	})
	if err != nil {
		msgsPublished.WithLabelValues(topicID, "error_send").Inc()
		return fmt.Errorf("failed to publish to topic %s: %w", topicID, err)
	}

	msgsPublished.WithLabelValues(topicID, "success").Inc()
	return nil
}

// Subscribe creates a local subscription to the topic.
func (m *Mux) Subscribe(topicID string) (<-chan *portalpb.Mote, func()) {
	ch := make(chan *portalpb.Mote, 100) // Buffer size 100

	m.subscribers.Lock()
	m.subs[topicID] = append(m.subs[topicID], ch)
	m.subscribers.Unlock()

	// Replay history
	m.history.RLock()
	if buf, ok := m.hist[topicID]; ok {
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

	cancel := func() {
		m.subscribers.Lock()
		defer m.subscribers.Unlock()

		subs := m.subs[topicID]
		for i, c := range subs {
			if c == ch {
				// Remove from slice
				m.subs[topicID] = append(subs[:i], subs[i+1:]...)
				close(ch)
				break
			}
		}
	}

	return ch, cancel
}

// dispatch sends the message to all local subscribers.
func (m *Mux) dispatch(topicID string, mote *portalpb.Mote) {
	m.subscribers.RLock()
	defer m.subscribers.RUnlock()

	subs := m.subs[topicID]
	for _, ch := range subs {
		select {
		case ch <- mote:
		default:
			// Drop message if subscriber is slow
			slog.Warn("Dropping message for slow subscriber", "topic", topicID)
		}
	}
}

// addToHistory adds a message to the history buffer for the topic.
func (m *Mux) addToHistory(topicID string, mote *portalpb.Mote) {
	m.history.Lock()
	defer m.history.Unlock()

	buf, ok := m.hist[topicID]
	if !ok {
		buf = NewHistoryBuffer(m.historySize)
		m.hist[topicID] = buf
	}
	buf.Add(mote)
}

// dispatchMsg handles a raw pubsub message, unmarshals it, and dispatches it locally.
func (m *Mux) dispatchMsg(topicID string, msg *pubsub.Message) {
	defer msg.Ack()

	// Check for loopback
	if senderID, ok := msg.Metadata["sender_id"]; ok && senderID == m.serverID {
		return
	}

	msgsReceived.WithLabelValues(topicID).Inc()

	var mote portalpb.Mote
	if err := proto.Unmarshal(msg.Body, &mote); err != nil {
		slog.Error("Failed to unmarshal mote", "topic", topicID, "error", err)
		return
	}

	// Dispatch locally
	m.dispatch(topicID, &mote)

	// Add to history (messages from others also go into history)
	m.addToHistory(topicID, &mote)
}

func (m *Mux) receiveLoop(ctx context.Context, topicID string, sub *pubsub.Subscription) {
	for {
		msg, err := sub.Receive(ctx)
		if err != nil {
			// Context canceled or subscription closed
			return
		}
		m.dispatchMsg(topicID, msg)
	}
}
