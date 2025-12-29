package benchmarks

import (
	"context"
	"fmt"
	"io"
	"log"
	"net"
	"testing"
	"time"

	"gocloud.dev/pubsub"
	"gocloud.dev/pubsub/mempubsub"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"

	"realm.pub/tavern/internal/c2"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	entportal "realm.pub/tavern/internal/ent/portal"
	"realm.pub/tavern/internal/ent/task"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/portal"
	"realm.pub/tavern/internal/portal/portalpb"
	"realm.pub/tavern/internal/test_data"

	// Register sqlite3 driver
	_ "github.com/mattn/go-sqlite3"
)

const bufSize = 1024 * 1024

func BenchmarkPortalThroughput(b *testing.B) {
	log.SetOutput(io.Discard) // Suppress logs

	// Local listeners for this benchmark run
	lisC2 := bufconn.Listen(bufSize)
	lisPortal := bufconn.Listen(bufSize)

	bufDialerC2 := func(context.Context, string) (net.Conn, error) {
		return lisC2.Dial()
	}
	bufDialerPortal := func(context.Context, string) (net.Conn, error) {
		return lisPortal.Dial()
	}

	// 1. Database
	client := enttest.Open(b, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer client.Close()

	// 3. Servers
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	c2Server, portalServer, topic, sub, err := startComponents(ctx, client)
	if err != nil {
		b.Fatalf("Failed to start components: %v", err)
	}
	defer topic.Shutdown(context.Background())
	defer sub.Shutdown(context.Background())

	// Start gRPC Servers
	grpcServerC2 := grpc.NewServer()
	c2pb.RegisterC2Server(grpcServerC2, c2Server)
	go grpcServerC2.Serve(lisC2)
	defer grpcServerC2.Stop()

	grpcServerPortal := grpc.NewServer()
	portalpb.RegisterPortalServer(grpcServerPortal, portalServer)
	go grpcServerPortal.Serve(lisPortal)
	defer grpcServerPortal.Stop()

	// 4. Seed Data
	test_data.CreateTestData(ctx, client)
	user := test_data.User(ctx, client)
	beacon := test_data.Beacon(ctx, client, user)
	tome := test_data.Tome(ctx, client, user)
	quest := test_data.Quest(ctx, client, user, tome)
	taskEnt := test_data.Task(ctx, client, quest, beacon)

	// 5. Connect Clients
	connC2, err := grpc.DialContext(ctx, "bufnet", grpc.WithContextDialer(bufDialerC2), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		b.Fatalf("Failed to dial C2: %v", err)
	}
	defer connC2.Close()
	c2Client := c2pb.NewC2Client(connC2)

	connPortal, err := grpc.DialContext(ctx, "bufnet", grpc.WithContextDialer(bufDialerPortal), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		b.Fatalf("Failed to dial Portal: %v", err)
	}
	defer connPortal.Close()
	portalClient := portalpb.NewPortalClient(connPortal)

	// 6. Establish Streams (The "Setup")

	// A. CreatePortal
	createStream, err := c2Client.CreatePortal(ctx)
	if err != nil {
		b.Fatalf("CreatePortal failed: %v", err)
	}
	// Send Register
	if err := createStream.Send(&c2pb.CreatePortalRequest{TaskId: int64(taskEnt.ID)}); err != nil {
		b.Fatalf("CreatePortal register failed: %v", err)
	}

	// Wait for Portal Entity
	var portalID int
	for k := 0; k < 50; k++ {
		// We look for the latest portal for this task
		p, err := client.Portal.Query().
			Where(entportal.HasTaskWith(task.ID(taskEnt.ID))).
			Order(ent.Desc(entportal.FieldID)).
			First(ctx)
		if err == nil && p != nil {
			portalID = p.ID
			break
		}
		time.Sleep(50 * time.Millisecond)
	}
	if portalID == 0 {
		b.Fatal("Timed out waiting for Portal entity creation")
	}

	// B. InvokePortal
	invokeStream, err := portalClient.InvokePortal(ctx)
	if err != nil {
		b.Fatalf("InvokePortal failed: %v", err)
	}
	// Send Register
	if err := invokeStream.Send(&portalpb.InvokePortalRequest{PortalId: int64(portalID)}); err != nil {
		b.Fatalf("InvokePortal register failed: %v", err)
	}

	// C. Drain CreatePortal Responses (Echo/Pings)
	go func() {
		for {
			_, err := createStream.Recv()
			if err != nil {
				return
			}
		}
	}()

	// 7. Benchmark Loop
	payloadSize := 1024 * 1024 // 1MB
	payloadData := make([]byte, payloadSize)
	// fill with some data
	payload := &portalpb.Payload{
		Payload: &portalpb.Payload_Bytes{
			Bytes: &portalpb.BytesMessage{
				Data: payloadData,
				Kind: portalpb.BytesMessageKind_BYTES_MESSAGE_KIND_DATA,
			},
		},
	}
	req := &c2pb.CreatePortalRequest{Payload: payload}

	b.SetBytes(int64(payloadSize))
	b.ResetTimer()

	// Run Send and Recv concurrently to pipeline messages
	errChan := make(chan error, 2)
	go func() {
		for i := 0; i < b.N; i++ {
			if err := createStream.Send(req); err != nil {
				errChan <- err
				return
			}
		}
	}()

	go func() {
		for i := 0; i < b.N; i++ {
			// Recv
			// We must loop until we get data, ignoring pings
			gotData := false
			for !gotData {
				resp, err := invokeStream.Recv()
				if err != nil {
					errChan <- err
					return
				}

				// Check if payload is data
				if resp.Payload != nil {
					if bm := resp.Payload.GetBytes(); bm != nil {
						if bm.Kind == portalpb.BytesMessageKind_BYTES_MESSAGE_KIND_DATA {
							gotData = true
							if len(bm.Data) != payloadSize {
								errChan <- fmt.Errorf("Size mismatch: got %d, want %d", len(bm.Data), payloadSize)
								return
							}
						}
					}
				}
			}
		}
		errChan <- nil
	}()

	// Wait for receiver to finish
	if err := <-errChan; err != nil {
		b.Fatalf("Benchmark failed: %v", err)
	}

	b.StopTimer()
	// Cleanup happens via defers
}

// Helpers
func startComponents(ctx context.Context, client *ent.Client) (*c2.Server, *portal.Server, *pubsub.Topic, *pubsub.Subscription, error) {
	topic := mempubsub.NewTopic()
	sub := mempubsub.NewSubscription(topic, 100*time.Millisecond)
	mux := stream.NewMux(topic, sub)

	go func() {
		if err := mux.Start(ctx); err != nil && err != context.Canceled {
			log.Printf("Mux exited: %v", err)
		}
	}()

	// Allow mux to start
	time.Sleep(100 * time.Millisecond)

	c2Server := c2.New(client, mux, mux)
	portalServer := portal.New(client, mux)

	return c2Server, portalServer, topic, sub, nil
}
