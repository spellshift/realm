package mux

import (
	"context"
	"fmt"
	"log/slog"

	"realm.pub/tavern/internal/logging"
)

// OpenPortal opens an existing portal for viewing (Client side).
func (m *Mux) OpenPortal(ctx context.Context, portalID int) (func(), error) {
	ctx = logging.WithStreamAttrs(ctx, slog.Int("portal_id", portalID))
	topicIn := m.TopicIn(portalID)
	topicOut := m.TopicOut(portalID)
	subName := m.SubName(topicOut)

	releaseRef := func() {
		m.subMgr.Lock()
		m.subMgr.refs[subName]--
		shouldShutdown := false
		hasLocalPortal := false
		var cancel context.CancelFunc
		if m.subMgr.refs[subName] <= 0 {
			if _, ok := m.subMgr.active[subName]; ok {
				cancel = m.subMgr.cancelFuncs[subName]
				delete(m.subMgr.active, subName)
				delete(m.subMgr.refs, subName)
				delete(m.subMgr.cancelFuncs, subName)
				shouldShutdown = true
			}
			_, hasLocalPortal = m.subMgr.active[m.SubName(topicIn)]
		}
		m.subMgr.Unlock()

		if shouldShutdown {
			if cancel != nil {
				log.DebugContext(logging.DetachContext(ctx), "closing mux portal output subscription", "topic", topicOut, "subscription", subName)
				cancel()
			}
			if !hasLocalPortal {
				m.cleanupTopic(topicOut)
				m.cleanupTopic(topicIn)
			}
		}
	}

	m.subMgr.Lock()
	// Check Cache
	if _, ok := m.subMgr.active[subName]; ok {
		m.subMgr.refs[subName]++
		refs := m.subMgr.refs[subName]
		m.subMgr.Unlock()
		log.DebugContext(ctx, "reusing active mux portal output subscription", "topic", topicOut, "subscription", subName, "refs", refs)
		return releaseRef, nil
	}
	m.subMgr.Unlock()

	// Provisioning
	// Ensure portal topics exist in case OpenPortal races ahead of CreatePortal topic setup right after DB commit
	if err := m.ensureTopic(ctx, m.TopicIn(portalID)); err != nil {
		return nil, fmt.Errorf("failed to ensure topic in: %w", err)
	}
	if err := m.ensureTopic(ctx, topicOut); err != nil {
		return nil, fmt.Errorf("failed to ensure topic out: %w", err)
	}
	// Ensure subscription exists for the OUT topic
	if err := m.ensureSub(ctx, topicOut, subName); err != nil {
		return nil, fmt.Errorf("failed to ensure subscription: %w", err)
	}

	// Connect
	sub, err := m.openSubscription(ctx, topicOut, subName)
	if err != nil {
		return nil, fmt.Errorf("failed to open subscription for topic %s: %w", topicOut, err)
	}

	m.subMgr.Lock()
	// RACE CONDITION CHECK:
	// Re-check cache in case another goroutine created it while we were provisioning/connecting
	if _, ok := m.subMgr.active[subName]; ok {
		// Another routine won the race. Use theirs.
		m.subMgr.refs[subName]++
		refs := m.subMgr.refs[subName]
		m.subMgr.Unlock()
		log.DebugContext(ctx, "joined concurrently created mux portal output subscription", "topic", topicOut, "subscription", subName, "refs", refs)
		return releaseRef, nil
	}

	// We won the race (or are the first).
	m.subMgr.active[subName] = sub
	m.subMgr.refs[subName] = 1

	// Prepare Loop Context
	ctxLoop, cancelLoop := context.WithCancel(logging.DetachContext(ctx))
	m.subMgr.cancelFuncs[subName] = cancelLoop

	m.subMgr.Unlock()

	log.DebugContext(ctx, "opened mux portal output subscription", "topic", topicOut, "subscription", subName)

	// Spawn
	go func() {
		// No defer cancelLoop() here, controlled by teardown/map
		m.receiveLoop(ctxLoop, topicOut, sub)
	}()

	return releaseRef, nil
}
