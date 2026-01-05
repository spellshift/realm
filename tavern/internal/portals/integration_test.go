package portals_test

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"fmt"
	"net"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"gocloud.dev/pubsub"
	_ "gocloud.dev/pubsub/mempubsub"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"

	"realm.pub/tavern/internal/c2"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/jwt"
	"realm.pub/tavern/internal/portals"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
	portalstream "realm.pub/tavern/portals/stream"

	// Needed for ent to work
	_ "github.com/mattn/go-sqlite3"
)

const bufSize = 1024 * 1024

type TestEnv struct {
	C2Client     c2pb.C2Client
	PortalClient portalpb.PortalClient
	Close        func()
	EntClient    *ent.Client
}

func SetupTestEnv(t *testing.T) *TestEnv {
	ctx := context.Background()

	// 1. Setup DB
	entClient := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")

	// 2. Setup Portal Mux (In-Memory)
	portalMux := mux.New(mux.WithInMemoryDriver())

	// 3. Setup C2 Stream Mux (Dummy In-Memory)
	topic, err := pubsub.OpenTopic(ctx, "mem://c2topic")
	require.NoError(t, err)
	sub, err := pubsub.OpenSubscription(ctx, "mem://c2topic")
	require.NoError(t, err)

	c2StreamMux := stream.NewMux(topic, sub)

	// Start c2StreamMux in background
	ctxMux, cancelMux := context.WithCancel(ctx)
	go c2StreamMux.Start(ctxMux)

	// Setup JWT service for testing
	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	jwtService, err := jwt.NewService(privKey, pubKey)
	require.NoError(t, err)

	c2Server := c2.New(entClient, c2StreamMux, portalMux, jwtService)
	portalServer := portals.New(entClient, portalMux)

	// 4. Setup gRPC Listener with bufconn
	lis := bufconn.Listen(bufSize)
	s := grpc.NewServer()

	c2pb.RegisterC2Server(s, c2Server)
	portalpb.RegisterPortalServer(s, portalServer)

	go func() {
		if err := s.Serve(lis); err != nil {
			// t.Logf("Server exited with error: %v", err)
		}
	}()

	// 5. Setup Clients
	dialer := func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}

	conn, err := grpc.DialContext(ctx, "bufnet", grpc.WithContextDialer(dialer), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to dial bufnet: %v", err)
	}

	c2Client := c2pb.NewC2Client(conn)
	portalClient := portalpb.NewPortalClient(conn)

	return &TestEnv{
		C2Client:     c2Client,
		PortalClient: portalClient,
		EntClient:    entClient,
		Close: func() {
			err := conn.Close()
			if err != nil {
				t.Logf("Failed to close connection: %v", err)
			}
			s.Stop()
			cancelMux()
			entClient.Close()
			topic.Shutdown(ctx)
			sub.Shutdown(ctx)
		},
	}
}

func CreateTask(ctx context.Context, client *ent.Client) (int, error) {
	// Create user
	u, err := client.User.Create().
		SetOauthID("testuser").
		SetName("Test User").
		SetPhotoURL("http://example.com/photo.jpg"). // Required field
		Save(ctx)
	if err != nil {
		return 0, fmt.Errorf("failed to create user: %w", err)
	}

	// Create Host (Required by Beacon)
	h, err := client.Host.Create().
		SetName("testhost").
		SetIdentifier("host1").
		SetPrimaryIP("127.0.0.1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		Save(ctx)
	if err != nil {
		return 0, fmt.Errorf("failed to create host: %w", err)
	}

	// Create beacon
	b, err := client.Beacon.Create().
		SetIdentifier("testbeacon").
		SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
		Save(ctx)
	if err != nil {
		return 0, fmt.Errorf("failed to create beacon: %w", err)
	}

	// Create tome
	tom, err := client.Tome.Create().
		SetName("testtome").
		SetDescription("test tome").
		SetAuthor("testuser").
		SetTactic("COMMAND_AND_CONTROL").
		SetEldritch("print('hello')"). // Required field
		Save(ctx)
	if err != nil {
		return 0, fmt.Errorf("failed to create tome: %w", err)
	}

	// Create quest
	q, err := client.Quest.Create().
		SetName("testquest").
		SetCreator(u).
		SetTome(tom).
		Save(ctx)
	if err != nil {
		return 0, fmt.Errorf("failed to create quest: %w", err)
	}

	// Create Task
	t, err := client.Task.Create().
		SetBeacon(b).
		SetQuest(q).
		Save(ctx)

	if err != nil {
		return 0, fmt.Errorf("failed to create task: %w", err)
	}

	return t.ID, nil
}

func TestPortalIntegration(t *testing.T) {
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

	// Wait for portal creation
	time.Sleep(500 * time.Millisecond)

	portalsAll, err := env.EntClient.Portal.Query().All(ctx)
	require.NoError(t, err)
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

	// 4. Setup Ordered Writers and Readers
	agentWriter := portalstream.NewOrderedWriter("agent", func(m *portalpb.Mote) error {
		return c2Stream.Send(&c2pb.CreatePortalRequest{Mote: m})
	})

	agentReader := portalstream.NewOrderedReader(func() (*portalpb.Mote, error) {
		resp, err := c2Stream.Recv()
		if err != nil {
			return nil, err
		}
		return resp.Mote, nil
	})

	userWriter := portalstream.NewOrderedWriter("user", func(m *portalpb.Mote) error {
		return portalStream.Send(&portalpb.OpenPortalRequest{Mote: m})
	})

	userReader := portalstream.NewOrderedReader(func() (*portalpb.Mote, error) {
		resp, err := portalStream.Recv()
		if err != nil {
			return nil, err
		}
		return resp.Mote, nil
	})

	// Synchronize: Ensure User is fully connected
	// We send a ping from User to Agent and verify receipt.
	// This ensures that the server's OpenPortal handler has reached sendPortalInput,
	// which occurs *after* the subscription to the output topic (PORTAL_OUT) is established.
	// This synchronization avoids a race condition where the Agent publishes to PORTAL_OUT
	// before the User has subscribed, causing the message to be dropped (as mempubsub warns).
	pingData := []byte("ping")
	err = userWriter.WriteBytes(pingData, portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA)
	require.NoError(t, err)

	pingMote, err := agentReader.Read()
	require.NoError(t, err)
	require.Equal(t, pingData, pingMote.GetBytes().Data)
	// Note: this consumes User->Agent Seq 0.

	// 5. Test Agent -> User (Ordered)
	t.Run("AgentToUser_Ordered", func(t *testing.T) {
		data := []byte("hello user")
		err := agentWriter.WriteBytes(data, portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA)
		require.NoError(t, err)

		mote, err := userReader.Read()
		require.NoError(t, err)
		require.Equal(t, data, mote.GetBytes().Data)
	})

	// 6. Test User -> Agent (Ordered)
	t.Run("UserToAgent_Ordered", func(t *testing.T) {
		data := []byte("hello agent")
		err := userWriter.WriteBytes(data, portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA)
		require.NoError(t, err)

		mote, err := agentReader.Read()
		require.NoError(t, err)
		require.Equal(t, data, mote.GetBytes().Data)
	})

	// 7. Test Agent -> User (Out of Order)
	t.Run("AgentToUser_OutOfOrder", func(t *testing.T) {
		mote2 := &portalpb.Mote{
			SeqId: 2,
			Payload: &portalpb.Mote_Bytes{
				Bytes: &portalpb.BytesPayload{Data: []byte("msg 2")},
			},
		}
		mote1 := &portalpb.Mote{
			SeqId: 1,
			Payload: &portalpb.Mote_Bytes{
				Bytes: &portalpb.BytesPayload{Data: []byte("msg 1")},
			},
		}
		mote3 := &portalpb.Mote{
			SeqId: 3,
			Payload: &portalpb.Mote_Bytes{
				Bytes: &portalpb.BytesPayload{Data: []byte("msg 3")},
			},
		}

		err := c2Stream.Send(&c2pb.CreatePortalRequest{Mote: mote3})
		require.NoError(t, err)
		err = c2Stream.Send(&c2pb.CreatePortalRequest{Mote: mote2})
		require.NoError(t, err)
		err = c2Stream.Send(&c2pb.CreatePortalRequest{Mote: mote1})
		require.NoError(t, err)

		readMote1, err := userReader.Read()
		require.NoError(t, err)
		require.Equal(t, uint64(1), readMote1.SeqId)
		require.Equal(t, []byte("msg 1"), readMote1.GetBytes().Data)

		readMote2, err := userReader.Read()
		require.NoError(t, err)
		require.Equal(t, uint64(2), readMote2.SeqId)
		require.Equal(t, []byte("msg 2"), readMote2.GetBytes().Data)

		readMote3, err := userReader.Read()
		require.NoError(t, err)
		require.Equal(t, uint64(3), readMote3.SeqId)
		require.Equal(t, []byte("msg 3"), readMote3.GetBytes().Data)
	})

	// 8. Test Integrity & Types
	t.Run("Integrity_TCP", func(t *testing.T) {
		data := []byte{0x00, 0x01, 0x02, 0xFF}
		dstAddr := "1.2.3.4"
		dstPort := uint32(80)

		err := userWriter.WriteTCP(data, dstAddr, dstPort)
		require.NoError(t, err)

		mote, err := agentReader.Read()
		require.NoError(t, err)
		require.NotNil(t, mote.GetTcp())
		require.Equal(t, data, mote.GetTcp().Data)
		require.Equal(t, dstAddr, mote.GetTcp().DstAddr)
		require.Equal(t, dstPort, mote.GetTcp().DstPort)
	})
}
