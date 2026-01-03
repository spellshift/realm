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

	m.subMgr.Lock()
	// Check Cache
	if _, ok := m.subMgr.active[subName]; ok {
		m.subMgr.refs[subName]++
		m.subMgr.Unlock()
		return func() {
			m.subMgr.Lock()
			m.subMgr.refs[subName]--
			shouldShutdown := false
			var s *pubsub.Subscription
			var cancel context.CancelFunc
			if m.subMgr.refs[subName] <= 0 {
				if sub, ok := m.subMgr.active[subName]; ok {
					s = sub
					cancel = m.subMgr.cancelFuncs[subName]
					delete(m.subMgr.active, subName)
					delete(m.subMgr.refs, subName)
					delete(m.subMgr.cancelFuncs, subName)
					shouldShutdown = true
				}
			}
			m.subMgr.Unlock()

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
	m.subMgr.Unlock()

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

	m.subMgr.Lock()
	// RACE CONDITION CHECK:
	// Re-check cache in case another goroutine created it while we were provisioning/connecting
	if existingSub, ok := m.subMgr.active[subName]; ok {
		// Another routine won the race. Use theirs.
		m.subMgr.refs[subName]++
		m.subMgr.Unlock()

		// Close our unused subscription immediately
		sub.Shutdown(context.Background())

		// Return teardown for the EXISTING subscription
		return func() {
			m.subMgr.Lock()
			m.subMgr.refs[subName]--
			shouldShutdown := false
			var s *pubsub.Subscription
			var cancel context.CancelFunc
			if m.subMgr.refs[subName] <= 0 {
				if sub, ok := m.subMgr.active[subName]; ok {
					s = sub
					cancel = m.subMgr.cancelFuncs[subName]
					delete(m.subMgr.active, subName)
					delete(m.subMgr.refs, subName)
					delete(m.subMgr.cancelFuncs, subName)
					shouldShutdown = true
				}
			}
			m.subMgr.Unlock()

			if shouldShutdown {
				if cancel != nil {
					cancel()
				}
				if existingSub != nil {
					s.Shutdown(context.Background())
				}
			}
		}, nil
	}

	// We won the race (or are the first).
	m.subMgr.active[subName] = sub
	m.subMgr.refs[subName] = 1

	// Prepare Loop Context
	ctxLoop, cancelLoop := context.WithCancel(context.Background())
	m.subMgr.cancelFuncs[subName] = cancelLoop

	m.subMgr.Unlock()

	// Spawn
	go func() {
		// No defer cancelLoop() here, controlled by teardown/map
		m.receiveLoop(ctxLoop, topicOut, sub)
	}()

	teardown := func() {
		m.subMgr.Lock()
		m.subMgr.refs[subName]--
		shouldShutdown := false
		var s *pubsub.Subscription
		var cancel context.CancelFunc
		if m.subMgr.refs[subName] <= 0 {
			if sub, ok := m.subMgr.active[subName]; ok {
				s = sub
				cancel = m.subMgr.cancelFuncs[subName]
				delete(m.subMgr.active, subName)
				delete(m.subMgr.refs, subName)
				delete(m.subMgr.cancelFuncs, subName)
				shouldShutdown = true
			}
		}
		m.subMgr.Unlock()

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
