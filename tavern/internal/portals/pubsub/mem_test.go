package pubsub_test

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/portals/pubsub"
	"realm.pub/tavern/portals/portalpb"
)

func TestInMemoryDriver(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// 1. Initialize tavern pubsub client with InMemory driver
	c := pubsub.NewClient(pubsub.WithInMemoryDriver())
	defer c.Close()

	projectID := "in-memory-project"

	// 2. Test EnsurePublisher
	topicName := fmt.Sprintf("projects/%s/topics/mem-topic", projectID)
	publisher, err := c.EnsurePublisher(ctx, topicName)
	require.NoError(t, err)
	require.NotNil(t, publisher)

	// 3. Test EnsureSubscriber
	subName := fmt.Sprintf("projects/%s/subscriptions/mem-sub", projectID)
	subscriber, err := c.EnsureSubscriber(ctx, topicName, subName)
	require.NoError(t, err)
	require.NotNil(t, subscriber)

	// 4. Test Publish and Receive
	received := make(chan *portalpb.Mote, 1)

	go func() {
		err := subscriber.Receive(ctx, func(ctx context.Context, mote *portalpb.Mote) {
			// Ignore keepalive messages
			if mote.GetBytes().GetKind() == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_KEEPALIVE {
				return
			}
			received <- mote
		})
		if err != nil && ctx.Err() == nil {
			t.Logf("Receive stopped: %v", err)
		}
	}()

	mote := &portalpb.Mote{
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA,
				Data: []byte("memory payload"),
			},
		},
	}

	err = publisher.Publish(ctx, mote)
	require.NoError(t, err)

	select {
	case r := <-received:
		assert.Equal(t, portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA, r.GetBytes().Kind)
		assert.Equal(t, []byte("memory payload"), r.GetBytes().Data)
	case <-time.After(5 * time.Second):
		t.Fatal("timed out waiting for message")
	}
}
