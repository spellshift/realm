package ent_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"

	_ "github.com/mattn/go-sqlite3"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/event"
	"realm.pub/tavern/internal/ent/host"
	"realm.pub/tavern/internal/ent/shell"
)

func TestHookDeriveNotifications(t *testing.T) {
	client := enttest.OpenTempDB(t)
	defer client.Close()

	// Add the hooks since enttest.Open doesn't by default
	client.Host.Use(ent.HookDeriveHostEvents())
	client.Task.Use(ent.HookDeriveQuestEvents())
	client.Shell.Use(ent.HookDeriveShellEvents())
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

func TestHookDeriveShellEvents(t *testing.T) {
	client := enttest.OpenTempDB(t)
	defer client.Close()

	// Add the hooks
	client.Shell.Use(ent.HookDeriveShellEvents())
	client.Event.Use(ent.HookDeriveNotifications())

	ctx := context.Background()

	// Create user
	u, err := client.User.Create().
		SetName("test-user").
		SetOauthID("oauth-1").
		SetPhotoURL("http://photo.com").
		Save(ctx)
	require.NoError(t, err)

	// Create a host and beacon for the shell
	h, err := client.Host.Create().
		SetIdentifier("host-1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		Save(ctx)
	require.NoError(t, err)

	b, err := client.Beacon.Create().
		SetHost(h).
		SetIdentifier("beacon-1").
		SetAgentIdentifier("agent-1").
		SetPrincipal("root").
		SetTransport(c2pb.Transport_TRANSPORT_GRPC).
		Save(ctx)
	require.NoError(t, err)

	t.Run("SHELL_CREATED creates event with user and shell edges", func(t *testing.T) {
		s, err := client.Shell.Create().
			SetBeacon(b).
			SetOwner(u).
			SetData([]byte{}).
			Save(ctx)
		require.NoError(t, err)

		// Verify event was created
		evt, err := client.Event.Query().
			Where(event.HasShellWith(shell.ID(s.ID))).
			WithShell().
			WithUser().
			Only(ctx)
		require.NoError(t, err)
		require.Equal(t, event.KindSHELL_CREATED, evt.Kind)
		require.NotNil(t, evt.Edges.Shell)
		require.Equal(t, s.ID, evt.Edges.Shell.ID)
		require.NotNil(t, evt.Edges.User)
		require.Equal(t, u.ID, evt.Edges.User.ID)

		// Verify notification was created for the user
		notifs, err := client.Notification.Query().WithUser().WithEvent().All(ctx)
		require.NoError(t, err)
		require.Len(t, notifs, 1)
		require.Equal(t, u.ID, notifs[0].Edges.User.ID)
		require.Equal(t, evt.ID, notifs[0].Edges.Event.ID)
	})
}
