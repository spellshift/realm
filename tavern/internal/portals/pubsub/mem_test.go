package pubsub_test

import (
	"context"
	"fmt"
	"testing"
	"time"

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

	// 4. Test Publish
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

	// Note: We cannot verify Receive here effectively because loopback detection drops our own message,
	// and WithInMemoryDriver creates an isolated environment where we are the only participant.
	// To test receive logic, we rely on TestGCPDriver which sets up two clients.

	// Ensure Receive doesn't crash
	go func() {
		err := subscriber.Receive(ctx, func(ctx context.Context, mote *portalpb.Mote) {
			// Should not happen for loopback
		})
		if err != nil && ctx.Err() == nil {
			t.Logf("Receive stopped: %v", err)
		}
	}()

	time.Sleep(100 * time.Millisecond)
}
