package mux

import (
	"context"
	"testing"
	"time"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/tome"
	"realm.pub/tavern/portals/portalpb"
)

func TestMux_InMemory(t *testing.T) {
	ctx := context.Background()
	// Setup Mux
	m, err := New(ctx, WithInMemoryDriver())
	require.NoError(t, err)
	defer m.Close()

	topicID := "test-topic"

	// Ensure Topic Exists
	err = m.ensureTopic(ctx, topicID)
	require.NoError(t, err)

	// Subscribe first
	ch, cancel := m.Subscribe(topicID, WithHistoryReplay())
	defer cancel()

	// Publish
	mote := &portalpb.Mote{
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Data: []byte("hello world"),
				Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA,
			},
		},
	}

	err = m.Publish(ctx, topicID, mote)
	require.NoError(t, err)

	// Verify Receive
	select {
	case received := <-ch:
		// proto messages might not compare equal due to internal fields, so we check content
		require.NotNil(t, received.GetBytes())
		assert.Equal(t, mote.GetBytes().Data, received.GetBytes().Data)
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for message")
	}

	// Verify History
	m.history.RLock()
	hist, ok := m.history.buffers[topicID]
	m.history.RUnlock()
	require.True(t, ok)
	assert.Equal(t, 1, len(hist.Get()))

	// New subscriber should get history
	ch2, cancel2 := m.Subscribe(topicID, WithHistoryReplay())
	defer cancel2()

	select {
	case received := <-ch2:
		require.NotNil(t, received.GetBytes())
		assert.Equal(t, mote.GetBytes().Data, received.GetBytes().Data)
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for history replay")
	}
}

func TestMux_CreatePortal(t *testing.T) {
	// Setup DB
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer client.Close()

	ctx := context.Background()
	// Setup Mux
	m, err := New(ctx, WithInMemoryDriver())
	require.NoError(t, err)
	defer m.Close()

	// Create User, Tome, Quest, Task required for Portal
	u := client.User.Create().SetName("testuser").SetOauthID("oauth").SetPhotoURL("photo").SaveX(ctx)
	h := client.Host.Create().SetName("host").SetIdentifier("ident").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	b := client.Beacon.Create().SetName("beacon").SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SetHost(h).SaveX(ctx)

	tomeEnt := client.Tome.Create().SetName("testtome").SetDescription("desc").SetAuthor(u.Name).SetUploader(u).SetTactic(tome.TacticRECON).SetEldritch("print(1)").SaveX(ctx)
	quest := client.Quest.Create().SetName("testquest").SetParameters("").SetCreator(u).SetTome(tomeEnt).SaveX(ctx)
	task := client.Task.Create().SetQuest(quest).SetBeacon(b).SaveX(ctx)

	// Updated call
	portalID, teardown, err := m.CreatePortal(ctx, client, task.ID)
	require.NoError(t, err)
	assert.NotZero(t, portalID)
	defer teardown()

	// Check DB
	portals := client.Portal.Query().AllX(ctx)
	require.Len(t, portals, 1)
	p := portals[0]
	if !p.ClosedAt.IsZero() {
		assert.True(t, p.ClosedAt.IsZero(), "ClosedAt should be zero/nil")
	}

	// Publish to IN topic (as if from Client)
	topicIn := m.TopicIn(portalID)
	mote := &portalpb.Mote{
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Data: []byte("input"),
			},
		},
	}
	err = m.Publish(ctx, topicIn, mote)
	require.NoError(t, err)

	// Check if dispatched locally (simulate Agent listening)
	ch, subCancel := m.Subscribe(topicIn, WithHistoryReplay())
	defer subCancel()

	// Wait for message (might be racey if dispatched before subscribe, but history should handle it)
	select {
	case received := <-ch:
		require.NotNil(t, received.GetBytes())
		assert.Equal(t, mote.GetBytes().Data, received.GetBytes().Data)
	case <-time.After(time.Second):
		t.Fatal("timeout")
	}

	// Teardown
	teardown()
	p = client.Portal.GetX(ctx, p.ID)
	assert.False(t, p.ClosedAt.IsZero())
}

func TestMux_OpenPortal(t *testing.T) {
	ctx := context.Background()
	m, err := New(ctx, WithInMemoryDriver())
	require.NoError(t, err)
	defer m.Close()

	portalID := 456

	// Simulate "Portal Output" topic existing (as if created by CreatePortal)
	topicOut := m.TopicOut(portalID)
	err = m.ensureTopic(ctx, topicOut)
	require.NoError(t, err)

	teardown, err := m.OpenPortal(ctx, portalID)
	require.NoError(t, err)

	// Publish to OUT topic (as if from Agent)
	mote := &portalpb.Mote{
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Data: []byte("output"),
			},
		},
	}
	err = m.Publish(ctx, topicOut, mote)
	require.NoError(t, err)

	// Subscribe to verify
	ch, subCancel := m.Subscribe(topicOut, WithHistoryReplay())
	defer subCancel()

	select {
	case received := <-ch:
		require.NotNil(t, received.GetBytes())
		assert.Equal(t, mote.GetBytes().Data, received.GetBytes().Data)
	case <-time.After(time.Second):
		t.Fatal("timeout")
	}

	// Test ref counting
	teardown2, err := m.OpenPortal(ctx, portalID)
	require.NoError(t, err)

	m.subMgr.Lock()
	subName := m.SubName(topicOut)
	assert.Equal(t, 2, m.subMgr.refs[subName])
	m.subMgr.Unlock()

	teardown()
	m.subMgr.Lock()
	assert.Equal(t, 1, m.subMgr.refs[subName])
	m.subMgr.Unlock()

	teardown2()
	m.subMgr.Lock()
	_, ok := m.subMgr.active[subName]
	assert.False(t, ok)
	m.subMgr.Unlock()
}
