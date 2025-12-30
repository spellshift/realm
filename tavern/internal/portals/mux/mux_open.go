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
			m.subRefs[subName]--
			shouldShutdown := false
			var s *pubsub.Subscription
			var cancel context.CancelFunc
			if m.subRefs[subName] <= 0 {
				if sub, ok := m.active[subName]; ok {
					s = sub
					cancel = m.cancelFuncs[subName]
					delete(m.active, subName)
					delete(m.subRefs, subName)
					delete(m.cancelFuncs, subName)
					shouldShutdown = true
				}
			}
			m.activeSubs.Unlock()

			if shouldShutdown {
				if cancel != nil {
					cancel()
				}
				if s != nil {
					s.Shutdown(context.Background())
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
	// RACE CONDITION CHECK:
	// Re-check cache in case another goroutine created it while we were provisioning/connecting
	if existingSub, ok := m.active[subName]; ok {
		// Another routine won the race. Use theirs.
		m.subRefs[subName]++
		m.activeSubs.Unlock()

		// Close our unused subscription immediately
		sub.Shutdown(context.Background())

		// Return teardown for the EXISTING subscription
		return func() {
			m.activeSubs.Lock()
			m.subRefs[subName]--
			shouldShutdown := false
			var s *pubsub.Subscription
			var cancel context.CancelFunc
			if m.subRefs[subName] <= 0 {
				if sub, ok := m.active[subName]; ok {
					s = sub
					cancel = m.cancelFuncs[subName]
					delete(m.active, subName)
					delete(m.subRefs, subName)
					delete(m.cancelFuncs, subName)
					shouldShutdown = true
				}
			}
			m.activeSubs.Unlock()

			if shouldShutdown {
				if cancel != nil {
					cancel()
				}
				if existingSub != nil {
					// We use existingSub here because we are in the closure of the raced call,
					// but wait, `s` extracted from map IS `existingSub` (or whatever is current).
					// Using `s` is safer.
					s.Shutdown(context.Background())
				}
			}
		}, nil
	}

	// We won the race (or are the first).
	m.active[subName] = sub
	m.subRefs[subName] = 1

	// Prepare Loop Context
	ctxLoop, cancelLoop := context.WithCancel(context.Background())
	m.cancelFuncs[subName] = cancelLoop

	m.activeSubs.Unlock()

	// Spawn
	go func() {
		// No defer cancelLoop() here, controlled by teardown/map
		m.receiveLoop(ctxLoop, topicOut, sub)
	}()

	teardown := func() {
		m.activeSubs.Lock()
		m.subRefs[subName]--
		shouldShutdown := false
		var s *pubsub.Subscription
		var cancel context.CancelFunc
		if m.subRefs[subName] <= 0 {
			if sub, ok := m.active[subName]; ok {
				s = sub
				cancel = m.cancelFuncs[subName]
				delete(m.active, subName)
				delete(m.subRefs, subName)
				delete(m.cancelFuncs, subName)
				shouldShutdown = true
			}
		}
		m.activeSubs.Unlock()

		if shouldShutdown {
			if cancel != nil {
				cancel()
			}
			if s != nil {
				s.Shutdown(context.Background())
			}
		}
	}

	return teardown, nil
}
