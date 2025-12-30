package mux

import (
	"context"
	"fmt"

	"gocloud.dev/pubsub"
)

// OpenPortal opens an existing portal for viewing (Client side).
func (m *Mux) OpenPortal(ctx context.Context, portalID int) (func(), error) {
	topicOut := m.TopicOut(portalID)
	subName := m.SubName(topicOut)

	m.activeSubs.Lock()
	// Check Cache
	if _, ok := m.active[subName]; ok {
		m.subRefs[subName]++
		m.activeSubs.Unlock()
		return func() {
			m.activeSubs.Lock()
			defer m.activeSubs.Unlock()
			m.subRefs[subName]--
			if m.subRefs[subName] <= 0 {
				if s, ok := m.active[subName]; ok {
					s.Shutdown(context.Background())
					delete(m.active, subName)
					delete(m.subRefs, subName)
				}
			}
		}, nil
	}
	m.activeSubs.Unlock()

	// Provisioning
	// Ensure subscription exists for the OUT topic
	if err := m.ensureSub(ctx, topicOut, subName); err != nil {
		return nil, fmt.Errorf("failed to ensure subscription: %w", err)
	}

	// Connect
	// Updated SubURL usage
	subURL := m.SubURL(topicOut, subName)
	sub, err := pubsub.OpenSubscription(ctx, subURL)
	if err != nil {
		return nil, fmt.Errorf("failed to open subscription %s: %w", subURL, err)
	}

	m.activeSubs.Lock()
	m.active[subName] = sub
	m.subRefs[subName] = 1
	m.activeSubs.Unlock()

	// Spawn
	ctxLoop, cancelLoop := context.WithCancel(context.Background())
	go func() {
		defer cancelLoop()
		m.receiveLoop(ctxLoop, topicOut, sub)
	}()

	teardown := func() {
		cancelLoop()

		m.activeSubs.Lock()
		defer m.activeSubs.Unlock()
		m.subRefs[subName]--
		if m.subRefs[subName] <= 0 {
			if s, ok := m.active[subName]; ok {
				s.Shutdown(context.Background())
				delete(m.active, subName)
				delete(m.subRefs, subName)
			}
		}
	}

	return teardown, nil
}
