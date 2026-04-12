package ent

import (
	"context"
	"fmt"
	"time"

	"entgo.io/ent"
	"realm.pub/tavern/internal/ent/event"
	"realm.pub/tavern/internal/ent/quest"
	"realm.pub/tavern/internal/ent/task"
)

// HookDeriveHostEvents will create host events based on mutations.
func HookDeriveHostEvents() ent.Hook {
	return func(next ent.Mutator) ent.Mutator {
		return ent.MutateFunc(func(ctx context.Context, m ent.Mutation) (ent.Value, error) {
			mut, ok := m.(*HostMutation)
			if !ok {
				return next.Mutate(ctx, m)
			}

			var shouldCreateNew, shouldCreateRecovered bool
			if mut.Op().Is(ent.OpCreate) {
				shouldCreateNew = true
			} else if mut.Op().Is(ent.OpUpdateOne) {
				newLastSeen, ok := mut.LastSeenAt()
				if ok {
					oldNextSeen, err := mut.OldNextSeenAt(ctx)
					if err == nil {
						// newLastSeen > oldNextSeen + 1 minute
						if newLastSeen.After(oldNextSeen.Add(1 * time.Minute)) {
							shouldCreateRecovered = true
						}
					}
				}
			}

			// We need to run the next mutator first to ensure the host is created/updated
			// and we have its ID for the event edge.
			val, err := next.Mutate(ctx, m)
			if err != nil {
				return val, err
			}

			if shouldCreateNew || shouldCreateRecovered {
				id, ok := mut.ID()
				if !ok {
					// For OpCreate, the ID might not be in the mutation, but in the returned value.
					if h, ok := val.(*Host); ok {
						id = h.ID
					} else {
						return val, fmt.Errorf("could not determine host ID for event creation")
					}
				}

				client := mut.Client()
				evtCreate := client.Event.Create().
					SetTimestamp(time.Now().Unix()).
					SetHostID(id)

				if shouldCreateNew {
					evtCreate.SetKind(event.KindHOST_ACCESS_NEW)
				} else if shouldCreateRecovered {
					evtCreate.SetKind(event.KindHOST_ACCESS_RECOVERED)
				}

				err := evtCreate.Exec(ctx)
				if err != nil {
					return nil, fmt.Errorf("creating host event: %w", err)
				}
			}

			return val, nil
		})
	}
}

// HookDeriveQuestEvents will create quest events based on task mutations.
func HookDeriveQuestEvents() ent.Hook {
	return func(next ent.Mutator) ent.Mutator {
		return ent.MutateFunc(func(ctx context.Context, m ent.Mutation) (ent.Value, error) {
			mut, ok := m.(*TaskMutation)
			if !ok {
				return next.Mutate(ctx, m)
			}

			var shouldCheckQuest bool
			if mut.Op().Is(ent.OpUpdateOne) {
				_, ok := mut.ExecFinishedAt()
				if ok {
					shouldCheckQuest = true
				}
			}

			val, err := next.Mutate(ctx, m)
			if err != nil {
				return val, err
			}

			if shouldCheckQuest {
				client := mut.Client()

				// Get quest ID. Try mutation first.
				questID, ok := mut.QuestID()
				if !ok {
					// If not in mutation, fetch the task to get its quest ID
					taskID, ok := mut.ID()
					if !ok {
						if t, ok := val.(*Task); ok {
							taskID = t.ID
						} else {
							return val, fmt.Errorf("could not determine task ID")
						}
					}
					t, err := client.Task.Query().Where(task.ID(taskID)).WithQuest().Only(ctx)
					if err != nil {
						return nil, fmt.Errorf("fetching task's quest: %w", err)
					}
					questID = t.Edges.Quest.ID
				}

				// Check if ALL tasks for this quest have ExecFinishedAt != nil
				allFinished, err := client.Task.Query().
					Where(
						task.HasQuestWith(quest.ID(questID)),
						task.ExecFinishedAtIsNil(),
					).
					Count(ctx)
				if err != nil {
					return nil, fmt.Errorf("counting unfinished tasks: %w", err)
				}

				if allFinished == 0 {
					// All tasks have ExecFinishedAt
					// Check if QUEST_COMPLETED event already exists
					exists, err := client.Event.Query().
						Where(
							event.HasQuestWith(quest.ID(questID)),
							event.KindEQ(event.KindQUEST_COMPLETED),
						).
						Exist(ctx)
					if err != nil {
						return nil, fmt.Errorf("checking for existing quest event: %w", err)
					}

					if !exists {
						err := client.Event.Create().
							SetKind(event.KindQUEST_COMPLETED).
							SetTimestamp(time.Now().Unix()).
							SetQuestID(questID).
							Exec(ctx)
						if err != nil {
							return nil, fmt.Errorf("creating quest completed event: %w", err)
						}
					}
				}
			}

			return val, nil
		})
	}
}

// Intercept the client directly in the constructor or use `ent.Client` features.
// Since ent generated `Open` functions return the client, we can't easily hook there without modifying `client.go` or `Open`.
// However, there is a better way!
