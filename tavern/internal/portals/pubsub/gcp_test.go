package pubsub_test

import (
	"context"
	"fmt"
	"testing"
	"time"

	gpubsub "cloud.google.com/go/pubsub/v2"
	"cloud.google.com/go/pubsub/v2/apiv1/pubsubpb"
	"cloud.google.com/go/pubsub/v2/pstest"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/api/option"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	xpubsub "realm.pub/tavern/internal/portals/pubsub"
	"realm.pub/tavern/portals/portalpb"
)

func TestGCPDriver(t *testing.T) {
	// 1. Start pstest server
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	srv := pstest.NewServer()
	defer srv.Close()

	// 2. Create pubsub client connected to pstest
	conn, err := grpc.NewClient(srv.Addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	require.NoError(t, err)
	defer conn.Close()

	projectID := "test-project"
	client, err := gpubsub.NewClient(ctx, projectID, option.WithGRPCConn(conn))
	require.NoError(t, err)
	defer client.Close()

	// 3. Initialize tavern pubsub client with GCP driver
	c := xpubsub.NewClient(xpubsub.WithGCPDriver("test-server", client))
	defer c.Close()

	// 4. Test EnsurePublisher
	// Note: gcp.go implementation uses the provided topic string directly in CreateTopic.
	// So we should provide fully qualified names.
	topicName := fmt.Sprintf("projects/%s/topics/test-topic", projectID)
	publisher, err := c.EnsurePublisher(ctx, topicName)
	require.NoError(t, err)
	require.NotNil(t, publisher)

	// Verify topic exists in pstest
	_, err = client.TopicAdminClient.GetTopic(ctx, &pubsubpb.GetTopicRequest{Topic: topicName})
	require.NoError(t, err)

	// 5. Test EnsureSubscriber
	subName := fmt.Sprintf("projects/%s/subscriptions/test-sub", projectID)
	subscriber, err := c.EnsureSubscriber(ctx, topicName, subName)
	require.NoError(t, err)
	require.NotNil(t, subscriber)

	_, err = client.SubscriptionAdminClient.GetSubscription(ctx, &pubsubpb.GetSubscriptionRequest{Subscription: subName})
	require.NoError(t, err)

	// 6. Test Publish and Receive
	received := make(chan *portalpb.Mote, 1)

	// Start receiving in a goroutine
	go func() {
		err := subscriber.Receive(ctx, func(ctx context.Context, mote *portalpb.Mote) {
			// Ignore keepalive messages which might be present from EnsurePublisher
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
				Data: []byte("hello world"),
			},
		},
	}

	err = publisher.Publish(ctx, mote)
	require.NoError(t, err)

	select {
	case r := <-received:
		assert.Equal(t, portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA, r.GetBytes().Kind)
		assert.Equal(t, []byte("hello world"), r.GetBytes().Data)
	case <-time.After(5 * time.Second):
		t.Fatal("timed out waiting for message")
	}
}

func TestGCPDriver_Failure(t *testing.T) {
	ctx := context.Background()

	// Start and immediately close server to simulate failure
	srv := pstest.NewServer()
	addr := srv.Addr
	srv.Close()

	conn, err := grpc.NewClient(addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	require.NoError(t, err)
	defer conn.Close()

	client, err := gpubsub.NewClient(ctx, "test-project", option.WithGRPCConn(conn))
	require.NoError(t, err)
	defer client.Close()

	c := xpubsub.NewClient(xpubsub.WithGCPDriver("test-server", client))
	defer c.Close()

	// Attempt to ensure publisher should fail
	ctxShort, cancel := context.WithTimeout(ctx, 1*time.Second)
	defer cancel()

	_, err = c.EnsurePublisher(ctxShort, "projects/test-project/topics/fail-topic")
	assert.Error(t, err)
}
