package mux

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/portals/portalpb"
)

func TestMux_DispatchTimeout_Drop(t *testing.T) {
	// Setup Mux with small buffer size
	m := New(WithInMemoryDriver(), WithSubscriberBufferSize(1))
	ctx := context.Background()
	topicID := "test-timeout-drop"

	err := m.ensureTopic(ctx, topicID)
	require.NoError(t, err)

	// Subscribe
	ch, cancel := m.Subscribe(topicID)
	defer cancel()

	// Fill buffer (size 1)
	mote1 := &portalpb.Mote{Payload: &portalpb.Mote_Bytes{Bytes: &portalpb.BytesPayload{Data: []byte("1")}}}
	err = m.Publish(ctx, topicID, mote1)
	require.NoError(t, err)

	// Confirm message is in channel (don't read it yet)
	assert.Equal(t, 1, len(ch))

	// Publish second message, should block for ~100ms then drop
	mote2 := &portalpb.Mote{Payload: &portalpb.Mote_Bytes{Bytes: &portalpb.BytesPayload{Data: []byte("2")}}}

	start := time.Now()
	err = m.Publish(ctx, topicID, mote2)
	duration := time.Since(start)

	require.NoError(t, err) // Dispatch doesn't return error on drop

	// Assert it took at least 100ms (allow some small margin if scheduling is fast, but practically > 90ms)
	assert.GreaterOrEqual(t, duration.Milliseconds(), int64(90), "Expected publish to block for ~100ms, took %v", duration)

	// Assert channel is still full with 1 message (the first one)
	assert.Equal(t, 1, len(ch))

	// Verify we can read the first message
	msg := <-ch
	assert.Equal(t, mote1.GetBytes().Data, msg.GetBytes().Data)
}

func TestMux_DispatchTimeout_Success(t *testing.T) {
	// Setup Mux with small buffer size
	m := New(WithInMemoryDriver(), WithSubscriberBufferSize(1))
	ctx := context.Background()
	topicID := "test-timeout-success"

	err := m.ensureTopic(ctx, topicID)
	require.NoError(t, err)

	// Subscribe
	ch, cancel := m.Subscribe(topicID)
	defer cancel()

	// Fill buffer (size 1)
	mote1 := &portalpb.Mote{Payload: &portalpb.Mote_Bytes{Bytes: &portalpb.BytesPayload{Data: []byte("1")}}}
	err = m.Publish(ctx, topicID, mote1)
	require.NoError(t, err)

	// Start consumer to read after 50ms
	go func() {
		time.Sleep(50 * time.Millisecond)
		<-ch // Read mote1
	}()

	// Publish second message
	mote2 := &portalpb.Mote{Payload: &portalpb.Mote_Bytes{Bytes: &portalpb.BytesPayload{Data: []byte("2")}}}

	start := time.Now()
	err = m.Publish(ctx, topicID, mote2)
	duration := time.Since(start)

	require.NoError(t, err)

	// Assert it took roughly 50ms
	assert.GreaterOrEqual(t, duration.Milliseconds(), int64(40), "Expected publish to block for ~50ms, took %v", duration)
	assert.Less(t, duration.Milliseconds(), int64(200), "Expected publish to complete before timeout, took %v", duration)

	// Verify we can read the second message
	msg := <-ch
	assert.Equal(t, mote2.GetBytes().Data, msg.GetBytes().Data)
}
