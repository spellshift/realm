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
	"realm.pub/tavern/internal/ent/user"
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

// TestHookDeriveUserRequestEvents verifies that creating a non-activated user
// triggers a NEW_USER_REQUEST event and notifies only admin users.
func TestHookDeriveUserRequestEvents(t *testing.T) {
	t.Run("non-activated user creates event and notifies admins", func(t *testing.T) {
		client := enttest.OpenTempDB(t)
		defer client.Close()

		client.Host.Use(ent.HookDeriveHostEvents())
		client.Task.Use(ent.HookDeriveQuestEvents())
		client.Event.Use(ent.HookDeriveNotifications())
		client.User.Use(ent.HookDeriveUserRequestEvents())

		ctx := context.Background()

		// Create an admin user (activated)
		admin, err := client.User.Create().
			SetName("admin-user").
			SetOauthID("oauth-admin").
			SetPhotoURL("http://photo.com/admin").
			SetIsAdmin(true).
			SetIsActivated(true).
			Save(ctx)
		require.NoError(t, err)

		// Create a regular activated user (non-admin)
		_, err = client.User.Create().
			SetName("regular-user").
			SetOauthID("oauth-regular").
			SetPhotoURL("http://photo.com/regular").
			SetIsActivated(true).
			Save(ctx)
		require.NoError(t, err)

		// Create a new non-activated user (should trigger NEW_USER_REQUEST)
		newUser, err := client.User.Create().
			SetName("new-user").
			SetOauthID("oauth-new").
			SetPhotoURL("http://photo.com/new").
			Save(ctx)
		require.NoError(t, err)

		// Verify NEW_USER_REQUEST event was created
		evt, err := client.Event.Query().
			Where(
				event.KindEQ(event.KindNEW_USER_REQUEST),
				event.HasUserWith(user.IDEQ(newUser.ID)),
			).
			Only(ctx)
		require.NoError(t, err)
		require.Equal(t, event.KindNEW_USER_REQUEST, evt.Kind)

		// Verify notification was created only for admin user
		notifs, err := client.Notification.Query().
			Where(notification.HasEventWith(event.ID(evt.ID))).
			WithUser().
			All(ctx)
		require.NoError(t, err)
		require.Len(t, notifs, 1, "only admin should be notified")
		assert.Equal(t, admin.ID, notifs[0].Edges.User.ID)
		assert.Equal(t, notification.PriorityHigh, notifs[0].Priority)
	})

	t.Run("activated user does not create event", func(t *testing.T) {
		client := enttest.OpenTempDB(t)
		defer client.Close()

		client.Host.Use(ent.HookDeriveHostEvents())
		client.Task.Use(ent.HookDeriveQuestEvents())
		client.Event.Use(ent.HookDeriveNotifications())
		client.User.Use(ent.HookDeriveUserRequestEvents())

		ctx := context.Background()

		// Create an activated user directly
		_, err := client.User.Create().
			SetName("active-user").
			SetOauthID("oauth-active").
			SetPhotoURL("http://photo.com/active").
			SetIsActivated(true).
			Save(ctx)
		require.NoError(t, err)

		// Verify no NEW_USER_REQUEST event was created
		count, err := client.Event.Query().
			Where(event.KindEQ(event.KindNEW_USER_REQUEST)).
			Count(ctx)
		require.NoError(t, err)
		assert.Equal(t, 0, count, "no event should be created for activated users")
	})

	t.Run("multiple admins all receive notifications", func(t *testing.T) {
		client := enttest.OpenTempDB(t)
		defer client.Close()

		client.Host.Use(ent.HookDeriveHostEvents())
		client.Task.Use(ent.HookDeriveQuestEvents())
		client.Event.Use(ent.HookDeriveNotifications())
		client.User.Use(ent.HookDeriveUserRequestEvents())

		ctx := context.Background()

		// Create two admin users
		admin1, err := client.User.Create().
			SetName("admin-one").
			SetOauthID("oauth-admin-1").
			SetPhotoURL("http://photo.com/admin1").
			SetIsAdmin(true).
			SetIsActivated(true).
			Save(ctx)
		require.NoError(t, err)

		admin2, err := client.User.Create().
			SetName("admin-two").
			SetOauthID("oauth-admin-2").
			SetPhotoURL("http://photo.com/admin2").
			SetIsAdmin(true).
			SetIsActivated(true).
			Save(ctx)
		require.NoError(t, err)

		// Create a non-admin, non-activated user
		_, err = client.User.Create().
			SetName("pending-user").
			SetOauthID("oauth-pending").
			SetPhotoURL("http://photo.com/pending").
			Save(ctx)
		require.NoError(t, err)

		// Verify NEW_USER_REQUEST event was created
		evts, err := client.Event.Query().
			Where(event.KindEQ(event.KindNEW_USER_REQUEST)).
			All(ctx)
		require.NoError(t, err)
		require.Len(t, evts, 1)

		// Verify both admins got notifications
		notifs, err := client.Notification.Query().
			Where(notification.HasEventWith(event.ID(evts[0].ID))).
			WithUser().
			All(ctx)
		require.NoError(t, err)
		require.Len(t, notifs, 2, "both admins should be notified")

		notifUserIDs := map[int]bool{}
		for _, n := range notifs {
			notifUserIDs[n.Edges.User.ID] = true
			assert.Equal(t, notification.PriorityHigh, n.Priority)
		}
		assert.True(t, notifUserIDs[admin1.ID], "admin1 should be notified")
		assert.True(t, notifUserIDs[admin2.ID], "admin2 should be notified")
	})
}
