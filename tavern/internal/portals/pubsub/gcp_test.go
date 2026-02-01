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

	// 2. Create pubsub clients connected to pstest
	// We need two clients to simulate two different servers (sender and receiver)
	// so that loopback detection doesn't drop the message.
	conn, err := grpc.NewClient(srv.Addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	require.NoError(t, err)
	defer conn.Close()

	projectID := "test-project"

	// Client 1: Sender
	gcpClient1, err := gpubsub.NewClient(ctx, projectID, option.WithGRPCConn(conn))
	require.NoError(t, err)
	defer gcpClient1.Close()
	sender := xpubsub.NewClient(xpubsub.WithGCPDriver("sender-server", gcpClient1))
	defer sender.Close()

	// Client 2: Receiver
	gcpClient2, err := gpubsub.NewClient(ctx, projectID, option.WithGRPCConn(conn))
	require.NoError(t, err)
	defer gcpClient2.Close()
	receiver := xpubsub.NewClient(xpubsub.WithGCPDriver("receiver-server", gcpClient2))
	defer receiver.Close()

	// 4. Test EnsurePublisher (Sender)
	topicName := fmt.Sprintf("projects/%s/topics/test-topic", projectID)
	publisher, err := sender.EnsurePublisher(ctx, topicName)
	require.NoError(t, err)
	require.NotNil(t, publisher)

	// Verify topic exists in pstest
	// We can check via gcpClient1
	_, err = gcpClient1.TopicAdminClient.GetTopic(ctx, &pubsubpb.GetTopicRequest{Topic: topicName})
	require.NoError(t, err)

	// 5. Test EnsureSubscriber (Receiver)
	subName := fmt.Sprintf("projects/%s/subscriptions/test-sub", projectID)
	// Ensure subscriber on receiver side
	subObj, err := receiver.EnsureSubscriber(ctx, topicName, subName)
	require.NoError(t, err)
	require.NotNil(t, subObj)

	_, err = gcpClient1.SubscriptionAdminClient.GetSubscription(ctx, &pubsubpb.GetSubscriptionRequest{Subscription: subName})
	require.NoError(t, err)

	// 6. Test Publish and Receive
	received := make(chan *portalpb.Mote, 1)

	// Start receiving in a goroutine
	go func() {
		err := subObj.Receive(ctx, func(ctx context.Context, mote *portalpb.Mote) {
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
