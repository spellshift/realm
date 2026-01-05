package portals_test

import (
	"context"
	"io"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/portals/portalpb"
)

func TestPortalClose(t *testing.T) {
	env := SetupTestEnv(t)
	defer env.Close()

	ctx := context.Background()

	// 1. Setup Data (Task)
	taskID, err := CreateTask(ctx, env.EntClient)
	require.NoError(t, err)

	// 2. Start C2.CreatePortal (Agent Side)
	c2Stream, err := env.C2Client.CreatePortal(ctx)
	require.NoError(t, err)

	// Send initial registration message
	err = c2Stream.Send(&c2pb.CreatePortalRequest{
		TaskId: int64(taskID),
	})
	require.NoError(t, err)

	// Wait for portal creation with retry backoff
	var portalsAll []*ent.Portal
	for i := 0; i < 10; i++ {
		portalsAll, err = env.EntClient.Portal.Query().All(ctx)
		require.NoError(t, err)
		if len(portalsAll) > 0 {
			break
		}
		time.Sleep(time.Duration(100*(i+1)) * time.Millisecond)
	}
	require.Len(t, portalsAll, 1)
	portalID := portalsAll[0].ID

	// 3. Start Portal.OpenPortal (User Side)
	portalStream, err := env.PortalClient.OpenPortal(ctx)
	require.NoError(t, err)

	// Send initial registration message
	err = portalStream.Send(&portalpb.OpenPortalRequest{
		PortalId: int64(portalID),
	})
	require.NoError(t, err)

	// 4. Verify connection by sending a ping from User to Agent
	pingData := []byte("ping")
	err = portalStream.Send(&portalpb.OpenPortalRequest{
		Mote: &portalpb.Mote{
			Payload: &portalpb.Mote_Bytes{
				Bytes: &portalpb.BytesPayload{
					Data: pingData,
					Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA,
				},
			},
		},
	})
	require.NoError(t, err)

	// Receive ping on Agent side
	resp, err := c2Stream.Recv()
	require.NoError(t, err)
	require.Equal(t, pingData, resp.Mote.GetBytes().Data)

	// 5. Close Agent Stream (Simulate Agent Disconnect/End of Session)
	err = c2Stream.CloseSend()
	require.NoError(t, err)

	// 6. Verify User Stream Closes
	// The OpenPortal handler will receive a CLOSE mote via the pubsub topic
	// and should forward it to the user client before closing the stream.

	// Read from portalStream - expect CLOSE mote
	msg, err := portalStream.Recv()

	// Handle case where we might get keepalive or other messages, but in this controlled test
	// we expect the next message to be the close signal or immediately closed.
	// However, OpenPortal sends the mote BEFORE checking if it is a CLOSE mote.
	// So we must receive it.

	if err == io.EOF {
		// If we get immediate EOF, it means we missed the CLOSE mote or it wasn't sent.
		// But based on code reading, it should be sent.
		t.Fatal("Expected CLOSE mote, got EOF immediately")
	}
	require.NoError(t, err)
	require.NotNil(t, msg.Mote)
	require.NotNil(t, msg.Mote.GetBytes())
	require.Equal(t, portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE, msg.Mote.GetBytes().Kind)

	// Attempt to receive again - expect error (portal closed) or EOF
	_, err = portalStream.Recv()
	require.Error(t, err)
	// Optionally check error message content if needed, but error presence is sufficient
	// require.Contains(t, err.Error(), "portal closed")
}
