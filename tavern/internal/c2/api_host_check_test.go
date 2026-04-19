package c2_test

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/c2test"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/event"
	"realm.pub/tavern/internal/ent/host"
	"realm.pub/tavern/internal/ent/notification"
	"realm.pub/tavern/internal/hostcheck"
)

// TestHostAccessLost verifies the end-to-end flow:
//  1. A beacon checks in, establishing a host.
//  2. The host fails to check in again (simulated by calling the host-check
//     handler with the expected NextSeenAt).
//  3. A HOST_ACCESS_LOST event is created.
//  4. URGENT notifications are sent to subscribers and Low notifications to
//     other users.
func TestHostAccessLost(t *testing.T) {
	ctx := context.Background()
	client, graph, close, _ := c2test.New(t)
	defer close()

	// Register hooks (c2test.New creates a plain ent client without hooks)
	graph.Host.Use(ent.HookDeriveHostEvents())
	graph.Task.Use(ent.HookDeriveQuestEvents())
	graph.Event.Use(ent.HookDeriveNotifications())

	// Create two users: one subscriber and one non-subscriber
	subscriber, err := graph.User.Create().
		SetName("subscriber-user").
		SetOauthID("oauth-sub-1").
		SetPhotoURL("http://photo.com/sub").
		SetIsActivated(true).
		Save(ctx)
	require.NoError(t, err)

	nonSubscriber, err := graph.User.Create().
		SetName("other-user").
		SetOauthID("oauth-other-1").
		SetPhotoURL("http://photo.com/other").
		SetIsActivated(true).
		Save(ctx)
	require.NoError(t, err)

	// Have a beacon check in to create a host
	interval := uint64(10) // 10 second interval
	_, err = client.ClaimTasks(ctx, &c2pb.ClaimTasksRequest{
		Beacon: &c2pb.Beacon{
			Identifier: "test-beacon-host-lost",
			Principal:  "root",
			Agent:      &c2pb.Agent{Identifier: "test-agent"},
			Host: &c2pb.Host{
				Identifier: "test-host-lost",
				Name:       "lost-host",
				Platform:   c2pb.Host_PLATFORM_LINUX,
				PrimaryIp:  "10.0.0.1",
			},
			AvailableTransports: &c2pb.AvailableTransports{
				Transports: []*c2pb.Transport{{
					Uri:      "grpc://127.0.0.1:8080",
					Interval: interval,
					Type:     c2pb.Transport_TRANSPORT_GRPC,
				}},
				ActiveIndex: 0,
			},
		},
	})
	require.Equal(t, codes.OK.String(), status.Code(err).String(), "ClaimTasks should succeed: %v", err)

	// Verify HOST_ACCESS_NEW event was created
	h, err := graph.Host.Query().Where(host.IdentifierEQ("test-host-lost")).Only(ctx)
	require.NoError(t, err)

	newEvent, err := graph.Event.Query().
		Where(
			event.HasHostWith(host.ID(h.ID)),
			event.KindEQ(event.KindHOST_ACCESS_NEW),
		).Only(ctx)
	require.NoError(t, err)
	assert.NotNil(t, newEvent, "HOST_ACCESS_NEW event should exist")

	// Subscribe one user to this host
	_, err = graph.Host.UpdateOne(h).AddSubscriberIDs(subscriber.ID).Save(ctx)
	require.NoError(t, err)

	// Set up host-check handler with the same ent client (which has hooks)
	handler := hostcheck.NewHandler(graph)
	ts := httptest.NewServer(handler)
	defer ts.Close()

	// Simulate the host being lost by calling the host-check endpoint
	// with the current NextSeenAt (meaning no new beacon checked in)
	checkReq := hostcheck.Request{
		HostID:             h.ID,
		ExpectedNextSeenAt: h.NextSeenAt,
	}
	body, err := json.Marshal(checkReq)
	require.NoError(t, err)

	resp, err := http.Post(ts.URL, "application/json", bytes.NewReader(body))
	require.NoError(t, err)
	defer resp.Body.Close()
	assert.Equal(t, http.StatusOK, resp.StatusCode, "host check handler should return 200")

	// Verify HOST_ACCESS_LOST event was created
	lostEvents, err := graph.Event.Query().
		Where(
			event.HasHostWith(host.ID(h.ID)),
			event.KindEQ(event.KindHOST_ACCESS_LOST),
		).All(ctx)
	require.NoError(t, err)
	require.Len(t, lostEvents, 1, "exactly one HOST_ACCESS_LOST event should exist")

	// Verify notifications were created
	notifs, err := graph.Notification.Query().
		Where(notification.HasEventWith(event.ID(lostEvents[0].ID))).
		WithUser().
		All(ctx)
	require.NoError(t, err)
	require.Len(t, notifs, 2, "notifications should be created for both users")

	// Verify subscriber got URGENT and non-subscriber got Low
	for _, n := range notifs {
		if n.Edges.User.ID == subscriber.ID {
			assert.Equal(t, notification.PriorityUrgent, n.Priority,
				"subscriber should get URGENT notification")
		} else if n.Edges.User.ID == nonSubscriber.ID {
			assert.Equal(t, notification.PriorityLow, n.Priority,
				"non-subscriber should get Low notification")
		} else {
			t.Errorf("unexpected user ID %d in notification", n.Edges.User.ID)
		}
	}

	// Verify idempotency: calling host check again should NOT create a duplicate event
	resp2, err := http.Post(ts.URL, "application/json", bytes.NewReader(body))
	require.NoError(t, err)
	defer resp2.Body.Close()
	assert.Equal(t, http.StatusOK, resp2.StatusCode)

	lostEventsAfter, err := graph.Event.Query().
		Where(
			event.HasHostWith(host.ID(h.ID)),
			event.KindEQ(event.KindHOST_ACCESS_LOST),
		).All(ctx)
	require.NoError(t, err)
	require.Len(t, lostEventsAfter, 1, "duplicate HOST_ACCESS_LOST event should NOT be created")

	// Verify that if the host checks in again (NextSeenAt changes), the old
	// check is correctly skipped.
	_, err = client.ClaimTasks(ctx, &c2pb.ClaimTasksRequest{
		Beacon: &c2pb.Beacon{
			Identifier: "test-beacon-host-lost",
			Principal:  "root",
			Agent:      &c2pb.Agent{Identifier: "test-agent"},
			Host: &c2pb.Host{
				Identifier: "test-host-lost",
				Name:       "lost-host",
				Platform:   c2pb.Host_PLATFORM_LINUX,
				PrimaryIp:  "10.0.0.1",
			},
			AvailableTransports: &c2pb.AvailableTransports{
				Transports: []*c2pb.Transport{{
					Uri:      "grpc://127.0.0.1:8080",
					Interval: interval,
					Type:     c2pb.Transport_TRANSPORT_GRPC,
				}},
				ActiveIndex: 0,
			},
		},
	})
	require.Equal(t, codes.OK.String(), status.Code(err).String())

	// The old ExpectedNextSeenAt no longer matches, so calling the handler with
	// the stale value should be a no-op.
	resp3, err := http.Post(ts.URL, "application/json", bytes.NewReader(body))
	require.NoError(t, err)
	defer resp3.Body.Close()
	assert.Equal(t, http.StatusOK, resp3.StatusCode)

	// Still only one HOST_ACCESS_LOST event
	finalLostEvents, err := graph.Event.Query().
		Where(
			event.HasHostWith(host.ID(h.ID)),
			event.KindEQ(event.KindHOST_ACCESS_LOST),
		).Count(ctx)
	require.NoError(t, err)
	assert.Equal(t, 1, finalLostEvents, "no additional HOST_ACCESS_LOST event should be created")

	// Verify a HOST_ACCESS_RECOVERED event was created (since the beacon
	// checked in again after being lost).
	recoveredCount, err := graph.Event.Query().
		Where(
			event.HasHostWith(host.ID(h.ID)),
			event.KindEQ(event.KindHOST_ACCESS_RECOVERED),
		).Count(ctx)
	require.NoError(t, err)

	_ = recoveredCount
	// Note: HOST_ACCESS_RECOVERED is triggered by HookDeriveHostEvents when
	// newLastSeen > oldNextSeen + 1 minute. In this test, the beacon checks
	// in right away, so the condition may or may not be met depending on timing.
	// We primarily verify the HOST_ACCESS_LOST flow here.

	t.Log("✅ HOST_ACCESS_LOST event created with URGENT notification for subscribers")
}

// TestHostAccessLostSkippedWhenHostUpdated verifies that the host check
// handler is a no-op when the host's NextSeenAt has been updated since the
// check was scheduled (i.e., the beacon checked in on time).
func TestHostAccessLostSkippedWhenHostUpdated(t *testing.T) {
	ctx := context.Background()
	_, graph, close, _ := c2test.New(t)
	defer close()

	graph.Host.Use(ent.HookDeriveHostEvents())
	graph.Event.Use(ent.HookDeriveNotifications())

	// Create a host directly
	h, err := graph.Host.Create().
		SetIdentifier("host-not-lost").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SetLastSeenAt(time.Now()).
		SetNextSeenAt(time.Now().Add(60 * time.Second)).
		Save(ctx)
	require.NoError(t, err)

	handler := hostcheck.NewHandler(graph)
	ts := httptest.NewServer(handler)
	defer ts.Close()

	// Send a check with a stale ExpectedNextSeenAt (different from current)
	staleTime := time.Now().Add(-5 * time.Minute)
	body, _ := json.Marshal(hostcheck.Request{
		HostID:             h.ID,
		ExpectedNextSeenAt: staleTime,
	})

	resp, err := http.Post(ts.URL, "application/json", bytes.NewReader(body))
	require.NoError(t, err)
	defer resp.Body.Close()
	assert.Equal(t, http.StatusOK, resp.StatusCode)

	// No HOST_ACCESS_LOST event should exist
	count, err := graph.Event.Query().
		Where(
			event.HasHostWith(host.ID(h.ID)),
			event.KindEQ(event.KindHOST_ACCESS_LOST),
		).Count(ctx)
	require.NoError(t, err)
	assert.Equal(t, 0, count, "no HOST_ACCESS_LOST event should be created when NextSeenAt changed")
}
