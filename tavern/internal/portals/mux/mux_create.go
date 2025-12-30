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

	// 2. Provisioning
	// Ensure topics exist
	if err := m.ensureTopic(ctx, topicIn); err != nil {
		return portalID, nil, fmt.Errorf("failed to ensure topic in: %w", err)
	}
	if err := m.ensureTopic(ctx, topicOut); err != nil {
		return portalID, nil, fmt.Errorf("failed to ensure topic out: %w", err)
	}

	// Ensure subscription exists
	if err := m.ensureSub(ctx, topicIn, subName); err != nil {
		return portalID, nil, fmt.Errorf("failed to ensure subscription: %w", err)
	}

	// 3. Connect
	// Updated SubURL usage
	subURL := m.SubURL(topicIn, subName)
	sub, err := pubsub.OpenSubscription(ctx, subURL)
	if err != nil {
		return portalID, nil, fmt.Errorf("failed to open subscription %s: %w", subURL, err)
	}

	// Store in subMgr
	m.subMgr.Lock()

	// Check if existing sub
	if existingSub, ok := m.subMgr.active[subName]; ok {
		// Existing found. Shutdown new one and cleanup old one if needed.
		if cancel, ok := m.subMgr.cancelFuncs[subName]; ok {
			cancel()
		}
		// We are overwriting, so we must assume the old one is invalid or we are restarting.
		existingSub.Shutdown(context.Background())
	}

	m.subMgr.active[subName] = sub

	ctxLoop, cancelLoop := context.WithCancel(context.Background())
	m.subMgr.cancelFuncs[subName] = cancelLoop
	m.subMgr.Unlock()

	// 4. Spawn
	go func() {
		m.receiveLoop(ctxLoop, topicIn, sub)
	}()

	teardown := func() {
		m.subMgr.Lock()
		s, ok := m.subMgr.active[subName]
		cancel := m.subMgr.cancelFuncs[subName]
		if ok {
			delete(m.subMgr.active, subName)
			delete(m.subMgr.cancelFuncs, subName)
		}
		m.subMgr.Unlock()

		if ok {
			if cancel != nil {
				cancel()
			}
			// Shutdown using Background context
			s.Shutdown(context.Background())
		}

		// Update DB to Closed using ID
		client.Portal.UpdateOneID(p.ID).
			SetClosedAt(time.Now()).
			Save(context.Background())
	}

	return portalID, teardown, nil
}
