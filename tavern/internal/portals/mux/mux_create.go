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
	// CreatePortal is generally unique per portalID/server, but technically multiple CreatePortal calls could race?
	// The prompt implies CreatePortal is Host/Agent side. Usually one per task.
	// But let's handle race anyway for consistency/robustness or just overwrite?
	// If we overwrite, we leak the previous one unless we check.
	if existingSub, ok := m.active[subName]; ok {
		// Existing found. Shutdown new one.
		// NOTE: This assumes we want to reuse existing or fail.
		// CreatePortal implies "Spawn".
		// If it exists, maybe we shouldn't have created it?
		// We'll replace it or error?
		// The requirement doesn't specify.
		// I will overwrite, but cleanup OLD one if it was there?
		// Or cleanup NEW one?
		// Since `receiveLoop` is tied to subscription, if we overwrite map entry, we lose track.
		// I'll assume CreatePortal should succeed.
		// For robustness, I'll close the NEW one and return error, OR close OLD one?
		// I'll close the OLD one properly if it exists.
		if cancel, ok := m.cancelFuncs[subName]; ok {
			cancel()
		}
		existingSub.Shutdown(context.Background())
	}
	m.active[subName] = sub

	ctxLoop, cancelLoop := context.WithCancel(context.Background())
	m.cancelFuncs[subName] = cancelLoop
	m.activeSubs.Unlock()

	// 4. Spawn
	go func() {
		m.receiveLoop(ctxLoop, topicIn, sub)
	}()

	teardown := func() {
		m.activeSubs.Lock()
		s, ok := m.active[subName]
		cancel := m.cancelFuncs[subName]
		if ok {
			delete(m.active, subName)
			delete(m.cancelFuncs, subName)
		}
		m.activeSubs.Unlock()

		if ok {
			if cancel != nil {
				cancel()
			}
			// Shutdown using Background context
			s.Shutdown(context.Background())
		}

		// Update DB to Closed
		client.Portal.UpdateOne(p).
			SetClosedAt(time.Now()).
			Save(context.Background())
	}

	return teardown, nil
}
