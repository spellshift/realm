package mux

import (
	"context"
	"fmt"
	"time"

	"gocloud.dev/pubsub"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/task"
)

// CreatePortal sets up a new portal for a task.
func (m *Mux) CreatePortal(ctx context.Context, client *ent.Client, taskID int) (int, func(), error) {
	// 1. DB: Create ent.Portal record (State: Open)
	// We need to fetch Task dependencies (Beacon, Owner/Creator) to satisfy Portal constraints.
	t, err := client.Task.Query().
		Where(task.ID(taskID)).
		WithBeacon().
		WithQuest(func(q *ent.QuestQuery) {
			q.WithCreator()
		}).
		Only(ctx)
	if err != nil {
		return 0, nil, fmt.Errorf("failed to query task %d: %w", taskID, err)
	}

	creator := t.Edges.Quest.Edges.Creator
	beacon := t.Edges.Beacon

	// Create Portal
	pCreate := client.Portal.Create().
		SetTaskID(taskID)

	if beacon != nil {
		pCreate.SetBeacon(beacon)
	}
	if creator != nil {
		pCreate.SetOwner(creator)
	}

	p, err := pCreate.Save(ctx)
	if err != nil {
		return 0, nil, fmt.Errorf("failed to create portal record: %w", err)
	}

	portalID := p.ID
	topicIn := m.TopicIn(portalID)
	topicOut := m.TopicOut(portalID)
	subName := m.SubName(topicIn)

	provision := func() error {
		// Ensure topics exist
		if err := m.ensureTopic(ctx, topicIn); err != nil {
			return fmt.Errorf("failed to ensure topic in: %w", err)
		}
		if err := m.ensureTopic(ctx, topicOut); err != nil {
			return fmt.Errorf("failed to ensure topic out: %w", err)
		}

		// Ensure subscription exists
		if err := m.ensureSub(ctx, topicIn, subName); err != nil {
			return fmt.Errorf("failed to ensure subscription: %w", err)
		}
		return nil
	}

	startLoop := func(ctxLoop context.Context, sub *pubsub.Subscription) {
		m.receiveLoop(ctxLoop, topicIn, sub)
	}

	teardownSub, err := m.acquireSubscription(ctx, subName, topicIn, provision, startLoop)
	if err != nil {
		return portalID, nil, err
	}

	teardown := func() {
		teardownSub()

		// Update DB to Closed using ID
		client.Portal.UpdateOneID(p.ID).
			SetClosedAt(time.Now()).
			Save(context.Background())
	}

	return portalID, teardown, nil
}
