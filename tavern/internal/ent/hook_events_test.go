package ent_test

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	_ "github.com/mattn/go-sqlite3"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/event"
	"realm.pub/tavern/internal/ent/host"
	"realm.pub/tavern/internal/ent/notification"
)

func TestHookDeriveNotifications(t *testing.T) {
	client := enttest.OpenTempDB(t)
	defer client.Close()

	// Add the hooks since enttest.Open doesn't by default
	client.Host.Use(ent.HookDeriveHostEvents())
	client.Task.Use(ent.HookDeriveQuestEvents())
	client.Event.Use(ent.HookDeriveNotifications())

	ctx := context.Background()

	// Create user
	u, err := client.User.Create().
		SetName("test-user").
		SetOauthID("oauth-1").
		SetPhotoURL("http://photo.com").
		Save(ctx)
	require.NoError(t, err)

	// Test HOST_ACCESS_NEW
	t.Run("HOST_ACCESS_NEW creates notification", func(t *testing.T) {
		h, err := client.Host.Create().
			SetIdentifier("host-1").
			SetPlatform(c2pb.Host_PLATFORM_LINUX).
			Save(ctx)
		require.NoError(t, err)

		// Verify event
		evt, err := client.Event.Query().Where(event.HasHostWith(host.ID(h.ID))).Only(ctx)
		require.NoError(t, err)
		require.Equal(t, event.KindHOST_ACCESS_NEW, evt.Kind)

		// Verify notification
		notifs, err := client.Notification.Query().WithUser().WithEvent().All(ctx)
		require.NoError(t, err)
		require.Len(t, notifs, 1)
		require.Equal(t, u.ID, notifs[0].Edges.User.ID)
		require.Equal(t, evt.ID, notifs[0].Edges.Event.ID)
	})
}

// TestHookDeriveNotifications_HostAccessRecovered verifies that HOST_ACCESS_RECOVERED
// events create Urgent notifications for subscribers and Medium notifications for
// non-subscribers.
func TestHookDeriveNotifications_HostAccessRecovered(t *testing.T) {
	t.Run("via UpdateOne", func(t *testing.T) {
		client := enttest.OpenTempDB(t)
		defer client.Close()

		client.Host.Use(ent.HookDeriveHostEvents())
		client.Task.Use(ent.HookDeriveQuestEvents())
		client.Event.Use(ent.HookDeriveNotifications())

		ctx := context.Background()

		// Create two users
		subscriber, err := client.User.Create().
			SetName("subscriber-user").
			SetOauthID("oauth-sub").
			SetPhotoURL("http://photo.com/sub").
			Save(ctx)
		require.NoError(t, err)

		nonSubscriber, err := client.User.Create().
			SetName("other-user").
			SetOauthID("oauth-other").
			SetPhotoURL("http://photo.com/other").
			Save(ctx)
		require.NoError(t, err)

		// Create a host with NextSeenAt in the past (simulating an overdue host)
		pastNext := time.Now().Add(-5 * time.Minute)
		h, err := client.Host.Create().
			SetIdentifier("host-recovered").
			SetPlatform(c2pb.Host_PLATFORM_LINUX).
			SetNextSeenAt(pastNext).
			Save(ctx)
		require.NoError(t, err)

		// Subscribe one user to the host
		_, err = client.Host.UpdateOne(h).AddSubscriberIDs(subscriber.ID).Save(ctx)
		require.NoError(t, err)

		// Simulate recovery: update LastSeenAt to now (which is > NextSeenAt + 1 minute)
		_, err = client.Host.UpdateOne(h).SetLastSeenAt(time.Now()).Save(ctx)
		require.NoError(t, err)

		// Verify HOST_ACCESS_RECOVERED event was created
		recoveredEvents, err := client.Event.Query().
			Where(
				event.HasHostWith(host.ID(h.ID)),
				event.KindEQ(event.KindHOST_ACCESS_RECOVERED),
			).All(ctx)
		require.NoError(t, err)
		require.Len(t, recoveredEvents, 1, "exactly one HOST_ACCESS_RECOVERED event should exist")

		// Verify notifications were created with correct priorities
		notifs, err := client.Notification.Query().
			Where(notification.HasEventWith(event.ID(recoveredEvents[0].ID))).
			WithUser().
			All(ctx)
		require.NoError(t, err)
		require.Len(t, notifs, 2, "notifications should be created for both users")

		for _, n := range notifs {
			if n.Edges.User.ID == subscriber.ID {
				assert.Equal(t, notification.PriorityUrgent, n.Priority,
					"subscriber should get Urgent notification")
			} else if n.Edges.User.ID == nonSubscriber.ID {
				assert.Equal(t, notification.PriorityMedium, n.Priority,
					"non-subscriber should get Medium notification")
			} else {
				t.Errorf("unexpected user ID %d in notification", n.Edges.User.ID)
			}
		}
	})

	// Test the upsert path used by ClaimTasks (Create().OnConflict().UpdateNewValues())
	t.Run("via Upsert", func(t *testing.T) {
		client := enttest.OpenTempDB(t)
		defer client.Close()

		client.Host.Use(ent.HookDeriveHostEvents())
		client.Task.Use(ent.HookDeriveQuestEvents())
		client.Event.Use(ent.HookDeriveNotifications())

		ctx := context.Background()

		// Create two users
		subscriber, err := client.User.Create().
			SetName("subscriber-user").
			SetOauthID("oauth-sub-2").
			SetPhotoURL("http://photo.com/sub").
			Save(ctx)
		require.NoError(t, err)

		nonSubscriber, err := client.User.Create().
			SetName("other-user").
			SetOauthID("oauth-other-2").
			SetPhotoURL("http://photo.com/other").
			Save(ctx)
		require.NoError(t, err)

		// Step 1: First upsert — creates the host (like first beacon check-in)
		hostID, err := client.Host.Create().
			SetIdentifier("host-upsert-recovered").
			SetName("test-host").
			SetPlatform(c2pb.Host_PLATFORM_LINUX).
			SetLastSeenAt(time.Now()).
			SetNextSeenAt(time.Now().Add(10 * time.Second)).
			OnConflict().
			UpdateNewValues().
			ID(ctx)
		require.NoError(t, err)

		// Verify HOST_ACCESS_NEW event
		newEvents, err := client.Event.Query().
			Where(
				event.HasHostWith(host.ID(hostID)),
				event.KindEQ(event.KindHOST_ACCESS_NEW),
			).All(ctx)
		require.NoError(t, err)
		require.Len(t, newEvents, 1, "HOST_ACCESS_NEW event should exist after first upsert")

		// Subscribe one user
		_, err = client.Host.UpdateOneID(hostID).AddSubscriberIDs(subscriber.ID).Save(ctx)
		require.NoError(t, err)

		// Step 2: Simulate the host being lost by setting NextSeenAt into the past
		pastTime := time.Now().Add(-5 * time.Minute)
		_, err = client.Host.UpdateOneID(hostID).SetNextSeenAt(pastTime).Save(ctx)
		require.NoError(t, err)

		// Step 3: Second upsert — beacon checks in again after being lost
		// newLastSeen (now) should be after oldNextSeen (5 min ago) + 1 minute
		_, err = client.Host.Create().
			SetIdentifier("host-upsert-recovered").
			SetName("test-host").
			SetPlatform(c2pb.Host_PLATFORM_LINUX).
			SetLastSeenAt(time.Now()).
			SetNextSeenAt(time.Now().Add(10 * time.Second)).
			OnConflict().
			UpdateNewValues().
			ID(ctx)
		require.NoError(t, err)

		// Verify HOST_ACCESS_RECOVERED event was created
		recoveredEvents, err := client.Event.Query().
			Where(
				event.HasHostWith(host.ID(hostID)),
				event.KindEQ(event.KindHOST_ACCESS_RECOVERED),
			).All(ctx)
		require.NoError(t, err)
		require.Len(t, recoveredEvents, 1, "exactly one HOST_ACCESS_RECOVERED event should exist after upsert recovery")

		// Verify notifications with correct priorities
		notifs, err := client.Notification.Query().
			Where(notification.HasEventWith(event.ID(recoveredEvents[0].ID))).
			WithUser().
			All(ctx)
		require.NoError(t, err)
		require.Len(t, notifs, 2, "notifications should be created for both users")

		for _, n := range notifs {
			if n.Edges.User.ID == subscriber.ID {
				assert.Equal(t, notification.PriorityUrgent, n.Priority,
					"subscriber should get Urgent notification")
			} else if n.Edges.User.ID == nonSubscriber.ID {
				assert.Equal(t, notification.PriorityMedium, n.Priority,
					"non-subscriber should get Medium notification")
			} else {
				t.Errorf("unexpected user ID %d in notification", n.Edges.User.ID)
			}
		}
	})
}

// TestHookDeriveNotifications_HostAccessLost verifies that HOST_ACCESS_LOST
// events create Urgent notifications for subscribers and Medium notifications
// for non-subscribers, matching the behaviour of HOST_ACCESS_RECOVERED.
func TestHookDeriveNotifications_HostAccessLost(t *testing.T) {
	client := enttest.OpenTempDB(t)
	defer client.Close()

	client.Host.Use(ent.HookDeriveHostEvents())
	client.Task.Use(ent.HookDeriveQuestEvents())
	client.Event.Use(ent.HookDeriveNotifications())

	ctx := context.Background()

	// Create two users: one subscriber and one non-subscriber
	subscriber, err := client.User.Create().
		SetName("subscriber-user").
		SetOauthID("oauth-sub-lost").
		SetPhotoURL("http://photo.com/sub").
		Save(ctx)
	require.NoError(t, err)

	nonSubscriber, err := client.User.Create().
		SetName("other-user").
		SetOauthID("oauth-other-lost").
		SetPhotoURL("http://photo.com/other").
		Save(ctx)
	require.NoError(t, err)

	// Create a host with NextSeenAt in the past (simulating an overdue host)
	pastNext := time.Now().Add(-5 * time.Minute)
	h, err := client.Host.Create().
		SetIdentifier("host-lost").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SetNextSeenAt(pastNext).
		Save(ctx)
	require.NoError(t, err)

	// Subscribe one user to the host
	_, err = client.Host.UpdateOne(h).AddSubscriberIDs(subscriber.ID).Save(ctx)
	require.NoError(t, err)

	// Create a HOST_ACCESS_LOST event directly (as the hostcheck handler does)
	err = client.Event.Create().
		SetKind(event.KindHOST_ACCESS_LOST).
		SetTimestamp(time.Now().Unix()).
		SetHostID(h.ID).
		Exec(ctx)
	require.NoError(t, err)

	// Verify exactly one HOST_ACCESS_LOST event was created
	lostEvents, err := client.Event.Query().
		Where(
			event.HasHostWith(host.ID(h.ID)),
			event.KindEQ(event.KindHOST_ACCESS_LOST),
		).All(ctx)
	require.NoError(t, err)
	require.Len(t, lostEvents, 1, "exactly one HOST_ACCESS_LOST event should exist")

	// Verify notifications: subscriber gets Urgent, non-subscriber gets Medium
	notifs, err := client.Notification.Query().
		Where(notification.HasEventWith(event.ID(lostEvents[0].ID))).
		WithUser().
		All(ctx)
	require.NoError(t, err)
	require.Len(t, notifs, 2, "notifications should be created for both users")

	for _, n := range notifs {
		if n.Edges.User.ID == subscriber.ID {
			assert.Equal(t, notification.PriorityUrgent, n.Priority,
				"subscriber should get Urgent notification")
		} else if n.Edges.User.ID == nonSubscriber.ID {
			assert.Equal(t, notification.PriorityMedium, n.Priority,
				"non-subscriber should get Medium notification for lost access")
		} else {
			t.Errorf("unexpected user ID %d in notification", n.Edges.User.ID)
		}
	}
}
