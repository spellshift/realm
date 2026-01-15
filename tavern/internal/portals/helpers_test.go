package portals_test

import (
	"context"
	"fmt"
	"net"
	"testing"

	_ "gocloud.dev/pubsub/mempubsub"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"

	"realm.pub/tavern/internal/c2"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/portals"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"

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

	// Start c2StreamMux (Not used, but kept for structure similarity if needed, removed here for simplicity as we use PortalMux now)
	// c2.New now takes portalMux.

	c2Server := c2.New(entClient, portalMux)
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
			entClient.Close()
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
