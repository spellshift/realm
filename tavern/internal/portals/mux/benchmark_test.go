package mux

import (
	"context"
	"crypto/rand"
	"testing"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/tome"
	"realm.pub/tavern/portals/portalpb"
)

func BenchmarkMuxThroughput(b *testing.B) {
	// Setup DB
	client := enttest.Open(b, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer client.Close()

	// Setup Mux
	m := New(WithInMemoryDriver(), WithSubscriberBufferSize(1000))
	ctx := context.Background()

	// Setup Entities
	u := client.User.Create().SetName("benchuser").SetOauthID("oauth").SetPhotoURL("photo").SaveX(ctx)
	h := client.Host.Create().SetName("benchhost").SetIdentifier("ident").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	beacon := client.Beacon.Create().SetName("benchbeacon").SetTransport(c2pb.Beacon_TRANSPORT_HTTP1).SetHost(h).SaveX(ctx)

	tomeEnt := client.Tome.Create().SetName("benchtome").SetDescription("desc").SetAuthor(u.Name).SetUploader(u).SetTactic(tome.TacticRECON).SetEldritch("nop").SaveX(ctx)
	quest := client.Quest.Create().SetName("benchquest").SetParameters("").SetCreator(u).SetTome(tomeEnt).SaveX(ctx)
	task := client.Task.Create().SetQuest(quest).SetBeacon(beacon).SaveX(ctx)

	// Setup Portals
	// Host Side
	portalID, teardownCreate, err := m.CreatePortal(ctx, client, task.ID)
	require.NoError(b, err)
	defer teardownCreate()

	hostCh, cancelHostSub := m.Subscribe(m.TopicIn(portalID))
	defer cancelHostSub()

	// Client Side
	teardownOpen, err := m.OpenPortal(ctx, portalID)
	require.NoError(b, err)
	defer teardownOpen()

	clientCh, cancelClientSub := m.Subscribe(m.TopicOut(portalID))
	defer cancelClientSub()

	// Payload (64KB)
	payloadSize := 64 * 1024
	payloadData := make([]byte, payloadSize)
	_, err = rand.Read(payloadData)
	require.NoError(b, err)

	mote := &portalpb.Mote{
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Data: payloadData,
				Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA,
			},
		},
	}

	b.SetBytes(int64(payloadSize * 2)) // Bidirectional throughput
	b.ResetTimer()

	// TODO: This fails if it runs long enough, we need to fix that. Seems like an OOM.
	for i := 0; i < b.N; i++ {
		// 1. Client sends to Host (TopicIn)
		if err := m.Publish(ctx, m.TopicIn(portalID), mote); err != nil {
			b.Fatalf("Failed to publish to TopicIn: %v", err)
		}

		// Host receives
		select {
		case <-hostCh:
			// Success
		case <-ctx.Done():
			b.Fatal("Context cancelled")
		}

		// 2. Host sends to Client (TopicOut)
		if err := m.Publish(ctx, m.TopicOut(portalID), mote); err != nil {
			b.Fatalf("Failed to publish to TopicOut: %v", err)
		}

		// Client receives
		select {
		case <-clientCh:
			// Success
		case <-ctx.Done():
			b.Fatal("Context cancelled")
		}
	}
}
