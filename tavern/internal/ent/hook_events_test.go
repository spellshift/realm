package ent_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"

	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/event"
	"realm.pub/tavern/internal/ent/host"
    _ "github.com/mattn/go-sqlite3"
)

func TestHookDeriveNotifications(t *testing.T) {
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
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
