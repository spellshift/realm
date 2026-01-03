package portals_test

import (
	"context"
	"crypto/rand"
	"fmt"
	"log/slog"
	"net"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	_ "github.com/mattn/go-sqlite3"
	"google.golang.org/grpc/test/bufconn"

	"realm.pub/tavern/internal/c2"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/portal"
	"realm.pub/tavern/internal/ent/task"
	"realm.pub/tavern/internal/ent/tome"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/portals"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
)

// bufListener implements a bufconn listener for testing gRPC over memory.
type bufListener struct {
	*bufconn.Listener
}

func newBufListener(sz int) *bufListener {
	return &bufListener{bufconn.Listen(sz)}
}

func BenchmarkPortalThroughput(b *testing.B) {
	// 1. Setup DB
	client := enttest.Open(b, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer client.Close()

	ctx := context.Background()

	// 2. Setup Entities
	u := client.User.Create().
		SetName("benchuser").
		SetOauthID("oauth").
		SetPhotoURL("photo").
		SaveX(ctx)

	h := client.Host.Create().
		SetName("benchhost").
		SetIdentifier("ident").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)

	beacon := client.Beacon.Create().
		SetName("benchbeacon").
		SetTransport(c2pb.ActiveTransport_TRANSPORT_HTTP1).
		SetHost(h).
		SaveX(ctx)

	tomeEnt := client.Tome.Create().
		SetName("benchtome").
		SetDescription("desc").
		SetAuthor(u.Name).
		SetUploader(u).
		SetTactic(tome.TacticRECON).
		SetEldritch("nop").
		SaveX(ctx)

	quest := client.Quest.Create().
		SetName("benchquest").
		SetParameters("").
		SetCreator(u).
		SetTome(tomeEnt).
		SaveX(ctx)

	taskEnt := client.Task.Create().
		SetQuest(quest).
		SetBeacon(beacon).
		SaveX(ctx)

	// 3. Setup Server Components
	portalMux := mux.New(mux.WithInMemoryDriver(), mux.WithSubscriberBufferSize(1000))

	// Create a placeholder shellMux since C2 requires it, but we won't use it.
	var shellMux *stream.Mux = nil

	c2Srv := c2.New(client, shellMux, portalMux)
	portalSrv := portals.New(client, portalMux)

	// 4. Start gRPC Server
	lis := newBufListener(1024 * 1024)
	s := grpc.NewServer()
	c2pb.RegisterC2Server(s, c2Srv)
	portalpb.RegisterPortalServer(s, portalSrv)

	go func() {
		if err := s.Serve(lis); err != nil {
			slog.Error("Server exited", "error", err)
		}
	}()
	defer s.Stop()

	// 5. Connect Clients
	conn, err := grpc.DialContext(ctx, "bufnet",
		grpc.WithContextDialer(func(context.Context, string) (net.Conn, error) {
			return lis.Dial()
		}),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	require.NoError(b, err)
	defer conn.Close()

	c2Client := c2pb.NewC2Client(conn)
	portalClient := portalpb.NewPortalClient(conn)

	// 6. Benchmark Logic
	payloadSize := 64 * 1024 // 64KB
	payload := make([]byte, payloadSize)
	_, err = rand.Read(payload)
	require.NoError(b, err)

	b.SetBytes(int64(payloadSize * 2)) // Bidirectional throughput per op

	// A. Agent Creates Portal
	agentStream, err := c2Client.CreatePortal(ctx)
	require.NoError(b, err)

	// Send initial request with TaskID
	err = agentStream.Send(&c2pb.CreatePortalRequest{
		TaskId: int64(taskEnt.ID),
	})
	require.NoError(b, err)

	// We need to wait for the server to process this and create the portal in DB.
	var portalID int
	require.Eventually(b, func() bool {
		p, err := client.Portal.Query().
			Where(portal.HasTaskWith(task.ID(taskEnt.ID))).
			Where(portal.ClosedAtIsNil()).
			Order(ent.Desc(portal.FieldID)).
			First(ctx)
		if err != nil {
			return false
		}
		portalID = p.ID
		return true
	}, 5*time.Second, 10*time.Millisecond, "Portal should be created in DB")

	// B. Client Opens Portal
	portalStream, err := portalClient.OpenPortal(ctx)
	require.NoError(b, err)

	// Send initial request with PortalID
	err = portalStream.Send(&portalpb.OpenPortalRequest{
		PortalId: int64(portalID),
	})
	require.NoError(b, err)

	// C. Message Exchange Loop
	var wg sync.WaitGroup
	errCh := make(chan error, 2)

	b.ResetTimer()

	wg.Add(2)

	// Agent Side: Echo Server
	go func() {
		defer wg.Done()
		for j := 0; j < b.N; j++ {
			// Receive from Client
			resp, err := agentStream.Recv()
			if err != nil {
				errCh <- fmt.Errorf("agent recv error: %w", err)
				return
			}
			mote := resp.GetMote()
			if mote == nil {
				errCh <- fmt.Errorf("agent received nil mote")
				return
			}

			// Echo back to Client
			req := &c2pb.CreatePortalRequest{
				Mote: mote,
			}
			if err := agentStream.Send(req); err != nil {
				errCh <- fmt.Errorf("agent send error: %w", err)
				return
			}
		}
	}()

	// Client Side: Sender
	go func() {
		defer wg.Done()
		mote := &portalpb.Mote{
			Payload: &portalpb.Mote_Bytes{
				Bytes: &portalpb.BytesPayload{
					Data: payload,
					Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA,
				},
			},
		}

		for j := 0; j < b.N; j++ {
			// Send to Agent
			req := &portalpb.OpenPortalRequest{
				Mote: mote,
			}
			if err := portalStream.Send(req); err != nil {
				errCh <- fmt.Errorf("client send error: %w", err)
				return
			}

			// Receive Echo from Agent
			resp, err := portalStream.Recv()
			if err != nil {
				errCh <- fmt.Errorf("client recv error: %w", err)
				return
			}
			if resp.GetMote() == nil {
				errCh <- fmt.Errorf("client received nil mote")
				return
			}
		}
	}()

	// Wait for completion
	done := make(chan struct{})
	go func() {
		wg.Wait()
		close(done)
	}()

	select {
	case <-done:
		// Success
	case err := <-errCh:
		b.Fatalf("Benchmark failed: %v", err)
	case <-time.After(60 * time.Second): // Timeout safety
		b.Fatal("Benchmark timed out")
	}

	b.StopTimer()

	// Cleanup
	agentStream.CloseSend()
	portalStream.CloseSend()
}
