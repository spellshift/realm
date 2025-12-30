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
func (m *Mux) CreatePortal(ctx context.Context, client *ent.Client, portalID int, taskID int) (func(), error) {
	topicIn := m.TopicIn(portalID)
	topicOut := m.TopicOut(portalID)
	subName := m.SubName(topicIn)

	// 1. Provisioning
	// Ensure topics exist
	if err := m.ensureTopic(ctx, topicIn); err != nil {
		return nil, fmt.Errorf("failed to ensure topic in: %w", err)
	}
	if err := m.ensureTopic(ctx, topicOut); err != nil {
		return nil, fmt.Errorf("failed to ensure topic out: %w", err)
	}

	// Ensure subscription exists
	if err := m.ensureSub(ctx, topicIn, subName); err != nil {
		return nil, fmt.Errorf("failed to ensure subscription: %w", err)
	}

	// 2. DB: Create ent.Portal record (State: Open)
	// We need to fetch Task dependencies (Beacon, Owner/Creator) to satisfy Portal constraints.
	t, err := client.Task.Query().
		Where(task.ID(taskID)).
		WithBeacon().
		WithQuest(func(q *ent.QuestQuery) {
			q.WithCreator()
		}).
		Only(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to query task %d: %w", taskID, err)
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
		return nil, fmt.Errorf("failed to create portal record: %w", err)
	}

	// 3. Connect
	// Updated SubURL usage
	subURL := m.SubURL(topicIn, subName)
	sub, err := pubsub.OpenSubscription(ctx, subURL)
	if err != nil {
		return nil, fmt.Errorf("failed to open subscription %s: %w", subURL, err)
	}

	// Store in activeSubs
	m.activeSubs.Lock()
	m.active[subName] = sub
	m.activeSubs.Unlock()

	// 4. Spawn
	ctxLoop, cancelLoop := context.WithCancel(context.Background())
	go func() {
		defer cancelLoop()
		m.receiveLoop(ctxLoop, topicIn, sub)
	}()

	teardown := func() {
		cancelLoop()

		m.activeSubs.Lock()
		if s, ok := m.active[subName]; ok {
			s.Shutdown(ctx) // Best effort shutdown
			delete(m.active, subName)
		}
		m.activeSubs.Unlock()

		// Update DB to Closed
		client.Portal.UpdateOne(p).
			SetClosedAt(time.Now()).
			Save(context.Background())
	}

	return teardown, nil
}
