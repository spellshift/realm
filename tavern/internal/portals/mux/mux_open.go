package mux

import (
	"context"
	"fmt"
	"log/slog"

	gcppubsub "cloud.google.com/go/pubsub"
	"gocloud.dev/pubsub"
)

// OpenPortal opens an existing portal for viewing (Client side).
func (m *Mux) OpenPortal(ctx context.Context, portalID int) (func(), error) {
	topicOut := m.TopicOut(portalID)
	subName := m.SubName(topicOut)

	m.subMgr.Lock()
	// Check Cache
	if _, ok := m.subMgr.active[subName]; ok {
		m.subMgr.refs[subName]++
		m.subMgr.Unlock()
		return m.createTeardownFunc(subName), nil
	}
	m.subMgr.Unlock()

	// Provisioning
	// Ensure subscription exists for the OUT topic
	if err := m.ensureSub(ctx, topicOut, subName); err != nil {
		return nil, fmt.Errorf("failed to ensure subscription: %w", err)
	}

	// Connect
	if !m.useInMem && m.gcpClient != nil {
		return m.openPortalGCP(ctx, topicOut, subName)
	}
	return m.openPortalGeneric(ctx, topicOut, subName)
}

func (m *Mux) openPortalGCP(ctx context.Context, topicOut, subName string) (func(), error) {
	sub := m.gcpClient.Subscription(subName)

	// Configure ReceiveSettings for low latency
	// Note: Synchronous is false by default.
	sub.ReceiveSettings.MaxOutstandingMessages = -1

	ctxLoop, cancelLoop := context.WithCancel(context.Background())

	m.subMgr.Lock()
	// RACE CONDITION CHECK
	if _, ok := m.subMgr.active[subName]; ok {
		m.subMgr.refs[subName]++
		m.subMgr.Unlock()
		cancelLoop()
		return m.createTeardownFunc(subName), nil
	}

	m.subMgr.active[subName] = sub
	m.subMgr.refs[subName] = 1
	m.subMgr.cancelFuncs[subName] = cancelLoop
	m.subMgr.Unlock()

	go func() {
		err := sub.Receive(ctxLoop, func(ctx context.Context, msg *gcppubsub.Message) {
			msg.Ack()
			senderID := ""
			if val, ok := msg.Attributes["sender_id"]; ok {
				senderID = val
			}
			m.dispatchRaw(topicOut, msg.Data, senderID)
		})
		if err != nil {
			// Only log error if not canceled
			if ctxLoop.Err() == nil {
				slog.Error("GCP Streaming Pull failed", "subscription", subName, "error", err)
				// Close streams
				m.subs.Lock()
				for _, ch := range m.subs.subs[topicOut] {
					close(ch)
				}
				m.subs.subs[topicOut] = nil
				m.subs.Unlock()

				// Cleanup manager
				m.subMgr.Lock()
				delete(m.subMgr.active, subName)
				delete(m.subMgr.refs, subName)
				delete(m.subMgr.cancelFuncs, subName)
				m.subMgr.Unlock()
			}
		}
	}()

	return m.createTeardownFunc(subName), nil
}

func (m *Mux) openPortalGeneric(ctx context.Context, topicOut, subName string) (func(), error) {
	subURL := m.SubURL(topicOut, subName)
	sub, err := pubsub.OpenSubscription(ctx, subURL)
	if err != nil {
		return nil, fmt.Errorf("failed to open subscription %s: %w", subURL, err)
	}

	m.subMgr.Lock()
	if _, ok := m.subMgr.active[subName]; ok {
		m.subMgr.refs[subName]++
		m.subMgr.Unlock()
		sub.Shutdown(context.Background())
		return m.createTeardownFunc(subName), nil
	}

	m.subMgr.active[subName] = sub
	m.subMgr.refs[subName] = 1

	ctxLoop, cancelLoop := context.WithCancel(context.Background())
	m.subMgr.cancelFuncs[subName] = cancelLoop

	m.subMgr.Unlock()

	go func() {
		m.receiveLoop(ctxLoop, topicOut, sub)
	}()

	return m.createTeardownFunc(subName), nil
}

func (m *Mux) createTeardownFunc(subName string) func() {
	return func() {
		m.subMgr.Lock()
		m.subMgr.refs[subName]--
		shouldShutdown := false
		var sGeneric *pubsub.Subscription
		var cancel context.CancelFunc

		if m.subMgr.refs[subName] <= 0 {
			if val, ok := m.subMgr.active[subName]; ok {
				cancel = m.subMgr.cancelFuncs[subName]
				delete(m.subMgr.active, subName)
				delete(m.subMgr.refs, subName)
				delete(m.subMgr.cancelFuncs, subName)

				if sub, ok := val.(*pubsub.Subscription); ok {
					sGeneric = sub
				}
				shouldShutdown = true
			}
		}
		m.subMgr.Unlock()

		if shouldShutdown {
			if cancel != nil {
				cancel()
			}
			if sGeneric != nil {
				sGeneric.Shutdown(context.Background())
			}
		}
	}
}
