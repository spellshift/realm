package mux

import (
	"context"
	"fmt"
	"time"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/shelltask"
	"realm.pub/tavern/internal/ent/task"
)

// CreatePortal sets up a new portal for a task.
func (m *Mux) CreatePortal(ctx context.Context, client *ent.Client, taskID int, shellTaskID int) (int, func(), error) {
	// 1. DB: Create ent.Portal record (State: Open)
	var creator *ent.User
	var beacon *ent.Beacon

	pCreate := client.Portal.Create()

	if taskID > 0 {
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

		creator = t.Edges.Quest.Edges.Creator
		beacon = t.Edges.Beacon
		pCreate.SetTaskID(taskID)
	} else if shellTaskID > 0 {
		st, err := client.ShellTask.Query().
			Where(shelltask.ID(shellTaskID)).
			WithCreator().
			WithShell(func(s *ent.ShellQuery) {
				s.WithBeacon()
			}).
			Only(ctx)
		if err != nil {
			return 0, nil, fmt.Errorf("failed to query shell task %d: %w", shellTaskID, err)
		}
		creator = st.Edges.Creator
		beacon = st.Edges.Shell.Edges.Beacon
		pCreate.SetShellTaskID(shellTaskID)
	} else {
		return 0, nil, fmt.Errorf("either taskID or shellTaskID must be provided")
	}

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
	sub, err := m.openSubscription(ctx, topicIn, subName)
	if err != nil {
		return portalID, nil, fmt.Errorf("failed to open subscription for topic %s: %w", topicIn, err)
	}

	// Store in subMgr
	m.subMgr.Lock()

	// Check if existing sub
	if _, ok := m.subMgr.active[subName]; ok {
		// Existing found. Shutdown existing one's loop.
		if cancel, ok := m.subMgr.cancelFuncs[subName]; ok {
			cancel()
		}
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
		_, ok := m.subMgr.active[subName]
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
		}

		// Update DB to Closed using ID
		client.Portal.UpdateOneID(p.ID).
			SetClosedAt(time.Now()).
			Save(context.Background())
	}

	return portalID, teardown, nil
}
