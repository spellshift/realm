package mux

import (
	"context"
	"strings"
	"sync"
	"testing"
	"time"

	_ "github.com/mattn/go-sqlite3"
	"github.com/prometheus/client_golang/prometheus"
	dto "github.com/prometheus/client_model/go"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/tome"
	"realm.pub/tavern/portals/portalpb"
)

func TestMux_InMemory(t *testing.T) {
	// Setup Mux (Defaults to In-Memory)
	m := New()

	ctx := context.Background()
	topicID := "test-topic"

	// Ensure Topic Exists
	err := m.ensureTopic(ctx, topicID)
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
	client := enttest.OpenTempDB(t)
	defer client.Close()

	// Setup Mux
	m := New()
	ctx := context.Background()

	// Create User, Tome, Quest, Task required for Portal
	u := client.User.Create().SetName("testuser").SetOauthID("oauth").SetPhotoURL("photo").SaveX(ctx)
	h := client.Host.Create().SetName("host").SetIdentifier("ident").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	b := client.Beacon.Create().SetName("beacon").SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SetHost(h).SaveX(ctx)

	tomeEnt := client.Tome.Create().SetName("testtome").SetDescription("desc").SetAuthor(u.Name).SetUploader(u).SetTactic(tome.TacticRECON).SetEldritch("print(1)").SaveX(ctx)
	quest := client.Quest.Create().SetName("testquest").SetParameters("").SetCreator(u).SetTome(tomeEnt).SaveX(ctx)
	task := client.Task.Create().SetQuest(quest).SetBeacon(b).SaveX(ctx)
	foreign := client.Beacon.Create().SetName("foreign").SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SetHost(h).SaveX(ctx)
	_, _, err := m.CreatePortal(ctx, client, task.ID, 0, foreign.ID)
	require.True(t, ent.IsNotFound(err))
	require.Zero(t, client.Portal.Query().CountX(ctx))

	// Updated call
	portalID, teardown, err := m.CreatePortal(ctx, client, task.ID, 0, b.ID)
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
	m := New()
	ctx := context.Background()
	portalID := 456

	// Simulate "Portal Output" topic existing (as if created by CreatePortal)
	topicOut := m.TopicOut(portalID)
	err := m.ensureTopic(ctx, topicOut)
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

func TestWithSubscriberBufferSize(t *testing.T) {
	expected := 12345
	m := New(WithSubscriberBufferSize(expected))
	assert.Equal(t, expected, m.subs.bufferSize)
}

func TestWithResourcePrefix(t *testing.T) {
	mUnset := New()
	assert.Equal(t, "PORTAL_IN_42", mUnset.TopicIn(42))
	assert.Equal(t, "PORTAL_OUT_42", mUnset.TopicOut(42))
	assert.True(t, strings.HasPrefix(mUnset.SubName(mUnset.TopicIn(42)), "PORTAL_IN_42_SUB_"))

	mCustom := New(WithResourcePrefix("abcxyz"))
	assert.Equal(t, "abcxyz-PORTAL_IN_42", mCustom.TopicIn(42))
	assert.Equal(t, "abcxyz-PORTAL_OUT_42", mCustom.TopicOut(42))
	assert.True(t, strings.HasPrefix(mCustom.SubName(mCustom.TopicIn(42)), "abcxyz-PORTAL_IN_42_SUB_"))
}

func TestMux_CreatePortal_ShellTask(t *testing.T) {
	// Setup DB
	client := enttest.OpenTempDB(t)
	defer client.Close()

	// Setup Mux
	m := New()
	ctx := context.Background()

	// Create User, Host, Beacon
	u := client.User.Create().SetName("testuser").SetOauthID("oauth").SetPhotoURL("photo").SaveX(ctx)
	h := client.Host.Create().SetName("host").SetIdentifier("ident").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	b := client.Beacon.Create().SetName("beacon").SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SetHost(h).SaveX(ctx)

	// Create Shell and ShellTask
	shell := client.Shell.Create().SetBeacon(b).SetOwner(u).SetData([]byte("")).SaveX(ctx)
	st := client.ShellTask.Create().SetShell(shell).SetCreator(u).SetInput("test").SetStreamID("stream").SetSequenceID(1).SaveX(ctx)
	foreign := client.Beacon.Create().SetName("foreign").SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SetHost(h).SaveX(ctx)
	_, _, err := m.CreatePortal(ctx, client, 0, st.ID, foreign.ID)
	require.True(t, ent.IsNotFound(err))
	require.Zero(t, client.Portal.Query().CountX(ctx))

	// Updated call
	portalID, teardown, err := m.CreatePortal(ctx, client, 0, st.ID, b.ID)
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

	// Verify Relations
	require.Equal(t, st.ID, p.QueryShellTask().OnlyIDX(ctx))
	require.Equal(t, b.ID, p.QueryBeacon().OnlyIDX(ctx))
	require.Equal(t, u.ID, p.QueryOwner().OnlyIDX(ctx))
}

func countTopicSeries(t *testing.T, c prometheus.Collector, topic string) int {
	t.Helper()
	ch := make(chan prometheus.Metric, 256)
	go func() {
		c.Collect(ch)
		close(ch)
	}()
	count := 0
	for metric := range ch {
		var pb dto.Metric
		require.NoError(t, metric.Write(&pb))
		for _, lp := range pb.GetLabel() {
			if lp.GetName() == "topic" && lp.GetValue() == topic {
				count++
			}
		}
	}
	return count
}

func TestMux_TeardownEvictsStateAndMetrics(t *testing.T) {
	client := enttest.OpenTempDB(t)
	defer client.Close()

	m := New(WithResourcePrefix("evicttest"))
	ctx := context.Background()

	u := client.User.Create().SetName("evictuser").SetOauthID("oauth-evict").SetPhotoURL("photo").SaveX(ctx)
	h := client.Host.Create().SetName("host-evict").SetIdentifier("ident-evict").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	b := client.Beacon.Create().SetName("beacon-evict").SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SetHost(h).SaveX(ctx)
	tomeEnt := client.Tome.Create().SetName("tome-evict").SetDescription("desc").SetAuthor(u.Name).SetUploader(u).SetTactic(tome.TacticRECON).SetEldritch("print(1)").SaveX(ctx)
	quest := client.Quest.Create().SetName("quest-evict").SetParameters("").SetCreator(u).SetTome(tomeEnt).SaveX(ctx)
	taskEnt := client.Task.Create().SetQuest(quest).SetBeacon(b).SaveX(ctx)

	portalID, teardown, err := m.CreatePortal(ctx, client, taskEnt.ID, 0, b.ID)
	require.NoError(t, err)

	topicIn := m.TopicIn(portalID)
	topicOut := m.TopicOut(portalID)

	chIn, cancelIn := m.Subscribe(topicIn)
	require.NoError(t, m.Publish(ctx, topicIn, &portalpb.Mote{
		Payload: &portalpb.Mote_Bytes{Bytes: &portalpb.BytesPayload{Data: []byte("in")}},
	}))
	require.NoError(t, m.Publish(ctx, topicOut, &portalpb.Mote{
		Payload: &portalpb.Mote_Bytes{Bytes: &portalpb.BytesPayload{Data: []byte("out")}},
	}))
	<-chIn
	cancelIn()

	// Seed received/dropped series for both topics so we verify eviction across all three counters
	msgsReceived.WithLabelValues(topicIn).Inc()
	msgsDropped.WithLabelValues(topicIn).Inc()
	msgsReceived.WithLabelValues(topicOut).Inc()
	msgsDropped.WithLabelValues(topicOut).Inc()

	assert.Greater(t, countTopicSeries(t, msgsPublished, topicIn), 0)
	assert.Greater(t, countTopicSeries(t, msgsPublished, topicOut), 0)
	assert.Equal(t, 1, countTopicSeries(t, msgsReceived, topicIn))
	assert.Equal(t, 1, countTopicSeries(t, msgsDropped, topicOut))

	teardown()

	m.history.RLock()
	_, hasHistIn := m.history.buffers[topicIn]
	_, hasHistOut := m.history.buffers[topicOut]
	m.history.RUnlock()
	assert.False(t, hasHistIn, "topicIn history buffer should be evicted")
	assert.False(t, hasHistOut, "topicOut history buffer should be evicted")

	m.topics.RLock()
	_, hasTopicIn := m.topics.topics[topicIn]
	_, hasTopicOut := m.topics.topics[topicOut]
	m.topics.RUnlock()
	assert.False(t, hasTopicIn, "topicIn publisher cache should be evicted")
	assert.False(t, hasTopicOut, "topicOut publisher cache should be evicted")

	m.subs.RLock()
	_, hasSubIn := m.subs.subs[topicIn]
	_, hasSubOut := m.subs.subs[topicOut]
	m.subs.RUnlock()
	assert.False(t, hasSubIn, "empty topicIn subscriber slice should be removed")
	assert.False(t, hasSubOut, "empty topicOut subscriber slice should be removed")

	assert.Zero(t, countTopicSeries(t, msgsPublished, topicIn))
	assert.Zero(t, countTopicSeries(t, msgsPublished, topicOut))
	assert.Zero(t, countTopicSeries(t, msgsReceived, topicIn))
	assert.Zero(t, countTopicSeries(t, msgsReceived, topicOut))
	assert.Zero(t, countTopicSeries(t, msgsDropped, topicIn))
	assert.Zero(t, countTopicSeries(t, msgsDropped, topicOut))
}

func TestMux_OpenPortalTeardownEvictsWhenNoLocalCreate(t *testing.T) {
	m := New(WithResourcePrefix("openevict"))
	ctx := context.Background()
	portalID := 789
	topicOut := m.TopicOut(portalID)

	teardown, err := m.OpenPortal(ctx, portalID)
	require.NoError(t, err)

	require.NoError(t, m.Publish(ctx, topicOut, &portalpb.Mote{
		Payload: &portalpb.Mote_Bytes{Bytes: &portalpb.BytesPayload{Data: []byte("hello")}},
	}))

	teardown()

	m.history.RLock()
	_, hasHistOut := m.history.buffers[topicOut]
	m.history.RUnlock()
	assert.False(t, hasHistOut)

	m.topics.RLock()
	_, hasTopicOut := m.topics.topics[topicOut]
	m.topics.RUnlock()
	assert.False(t, hasTopicOut)
	assert.Zero(t, countTopicSeries(t, msgsPublished, topicOut))
}

func TestMux_PublishClonesForConcurrentSubscribers(t *testing.T) {
	m := New()
	ctx := context.Background()
	topicID := "clone-race-topic"

	ch1, cancel1 := m.Subscribe(topicID)
	defer cancel1()
	ch2, cancel2 := m.Subscribe(topicID)
	defer cancel2()

	const count = 40
	var wg sync.WaitGroup
	mutateSubscriber := func(ch <-chan *portalpb.Mote, tag byte) {
		defer wg.Done()
		for i := 0; i < count; i++ {
			mote := <-ch
			mote.GetBytes().Data = append(mote.GetBytes().Data, tag)
		}
	}

	wg.Add(2)
	go mutateSubscriber(ch1, 'A')
	go mutateSubscriber(ch2, 'B')

	for i := 0; i < count; i++ {
		mote := &portalpb.Mote{
			Payload: &portalpb.Mote_Bytes{
				Bytes: &portalpb.BytesPayload{
					Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA,
					Data: []byte("payload"),
				},
			},
		}
		require.NoError(t, m.Publish(ctx, topicID, mote))
	}

	wg.Wait()
}
