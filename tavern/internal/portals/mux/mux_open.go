package mux

import (
	"context"

	"gocloud.dev/pubsub"
)

// OpenPortal opens an existing portal for viewing (Client side).
func (m *Mux) OpenPortal(ctx context.Context, portalID int) (func(), error) {
	topicOut := m.TopicOut(portalID)
	subName := m.SubName(topicOut)

	provision := func() error {
		// Ensure subscription exists for the OUT topic
		if err := m.ensureSub(ctx, topicOut, subName); err != nil {
			return err // m.ensureSub already formats error if needed, or we can wrap here. Original wrapped it.
		}
		return nil
	}

	startLoop := func(ctxLoop context.Context, sub *pubsub.Subscription) {
		m.receiveLoop(ctxLoop, topicOut, sub)
	}

	teardown, err := m.acquireSubscription(ctx, subName, topicOut, provision, startLoop)
	if err != nil {
		return nil, err // acquireSubscription already wraps error
	}

	return teardown, nil
}
