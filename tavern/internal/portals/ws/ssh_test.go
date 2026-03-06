package ws_test

import (
	"context"
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"encoding/pem"
	"fmt"
	"log"
	"net"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"

	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/require"
	"golang.org/x/crypto/ssh"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/portals/pubsub"
	"realm.pub/tavern/internal/portals/ws"
	"realm.pub/tavern/portals/portalpb"

	// Needed for ent to work
	entsql "entgo.io/ent/dialect/sql"
	_ "github.com/mattn/go-sqlite3"
	_ "realm.pub/tavern/internal/ent/runtime"
)

func generatePrivateKey(t *testing.T) ([]byte, error) {
	privateKey, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		return nil, err
	}

	privateKeyPEM := pem.EncodeToMemory(&pem.Block{
		Type:  "RSA PRIVATE KEY",
		Bytes: x509.MarshalPKCS1PrivateKey(privateKey),
	})

	return privateKeyPEM, nil
}

func startMockSSHServer(t *testing.T) (net.Listener, error) {
	config := &ssh.ServerConfig{
		PasswordCallback: func(c ssh.ConnMetadata, pass []byte) (*ssh.Permissions, error) {
			if c.User() == "testuser" && string(pass) == "testpass" {
				return nil, nil
			}
			return nil, fmt.Errorf("password rejected for %q", c.User())
		},
	}

	privateBytes, err := generatePrivateKey(t)
	require.NoError(t, err)
	private, err := ssh.ParsePrivateKey(privateBytes)
	require.NoError(t, err)
	config.AddHostKey(private)

	listener, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		return nil, err
	}

	go func() {
		for {
			nConn, err := listener.Accept()
			if err != nil {
				return // probably closed
			}

			// Before use, a handshake must be performed on the incoming net.Conn.
			conn, chans, reqs, err := ssh.NewServerConn(nConn, config)
			if err != nil {
				log.Printf("failed to handshake: %v", err)
				continue
			}

			// The incoming Request channel must be serviced.
			go ssh.DiscardRequests(reqs)

			// Service the incoming Channel channel.
			go func() {
				for newChannel := range chans {
					if newChannel.ChannelType() != "session" {
						newChannel.Reject(ssh.UnknownChannelType, "unknown channel type")
						continue
					}
					channel, requests, err := newChannel.Accept()
					if err != nil {
						log.Fatalf("Could not accept channel: %v", err)
					}

					// Sessions have out-of-band requests such as "shell", "pty-req" and "env".
					go func(in <-chan *ssh.Request) {
						for req := range in {
							ok := false
							switch req.Type {
							case "pty-req":
								ok = true
							case "shell":
								ok = true
								// start shell
								go func() {
									defer channel.Close()
									// Mock shell output
									channel.Write([]byte("Welcome to the mock shell\n$ "))
									buf := make([]byte, 256)
									for {
										n, err := channel.Read(buf)
										if err != nil {
											return
										}
										cmd := string(buf[:n])
										if strings.Contains(cmd, "exit") {
											return
										}
										channel.Write([]byte("Echo: " + cmd + "\n$ "))
									}
								}()
							}
							req.Reply(ok, nil)
						}
					}(requests)
				}
			}()

			_ = conn
		}
	}()

	return listener, nil
}

func TestSSHHandler(t *testing.T) {
	ctx := context.Background()

	// 1. Setup in-memory sqlite for Ent client
	drv, err := entsql.Open("sqlite3", "file:ent?mode=memory&cache=shared&_fk=1&_busy_timeout=10000")
	require.NoError(t, err)
	client := ent.NewClient(ent.Driver(drv))
	err = client.Schema.Create(ctx)
	require.NoError(t, err)
	defer client.Close()

	// 2. Setup Portal Mux
	pubSubClient := pubsub.NewClient(pubsub.WithInMemoryDriver())
	m := mux.New(mux.WithPubSubClient(pubSubClient))

	// Create required entities for portal
	owner := client.User.Create().SetName("admin").SetIsAdmin(true).SetOauthID("oauth-id").SetPhotoURL("url").SaveX(ctx)
	h := client.Host.Create().SetName("localhost").SetIdentifier("localhost").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	beacon := client.Beacon.Create().SetName("test-beacon").SetTransport(c2pb.Transport_TRANSPORT_GRPC).SetHost(h).SaveX(ctx)
	portal := client.Portal.Create().SetOwner(owner).SetBeacon(beacon).SaveX(ctx)

	// Pre-create the publish topic so EnsureSubscription succeeds in m.OpenPortal
	pubSubClient.EnsurePublisher(ctx, m.TopicOut(portal.ID))

	// 3. Start mock SSH server
	sshListener, err := startMockSSHServer(t)
	require.NoError(t, err)
	defer sshListener.Close()

	// 4. Setup HTTP Server for Handler
	handler := ws.NewSSHHandler(client, m)
	httpServer := httptest.NewServer(handler)
	defer httpServer.Close()

	// 5. Connect to WS
	u, _ := url.Parse(httpServer.URL)
	u.Scheme = "ws"

	// Target should point to the mock SSH server
	addr := sshListener.Addr().(*net.TCPAddr)
	target := fmt.Sprintf("testuser:testpass@127.0.0.1:%d", addr.Port)

	// Connect to WS endpoint
	wsURL := fmt.Sprintf("%s/portals/ssh/ws?portal_id=%d&target=%s", u.String(), portal.ID, target)
	wsConn, _, err := websocket.DefaultDialer.Dial(wsURL, nil)
	require.NoError(t, err)
	defer wsConn.Close()

	// Simulate a portal routing to handle TCP bytes sent from SSH handler -> Mux -> Our Mock -> SSH Server
	// We need a background goroutine to act as the "agent" forwarding the portal connection to the SSH Server.
	portalIn := m.TopicIn(portal.ID)
	recvIn, cleanupSubIn := m.Subscribe(portalIn)
	defer cleanupSubIn()

	go func() {
		// Mock agent routing TCP payload -> SSH Listener connection
		// Note: The SSH Handler creates its own direct TCP connection internally in our case because it uses SSH client
		// Wait, the SSH handler dials newPortalConn, which talks via Mux! So we must mock the Agent end!

		// Map of stream_id to net.Conn (connection to local ssh server)
		conns := make(map[string]net.Conn)

		for {
			select {
			case <-ctx.Done():
				return
			case mote, ok := <-recvIn:
				if !ok {
					return
				}

				streamID := mote.StreamId

				// Handle TCP bytes to write to SSH server
				if tcp := mote.GetTcp(); tcp != nil {
					conn, ok := conns[streamID]
					if !ok {
						// Dial ssh listener
						c, err := net.Dial("tcp", sshListener.Addr().String())
						if err != nil {
							continue
						}
						conns[streamID] = c
						conn = c

						// Start reading from the SSH server and sending back to Mux
						go func(c net.Conn, sid string) {
							defer c.Close()
							buf := make([]byte, 1024)
							seq := uint64(0)
							for {
								n, err := c.Read(buf)
								if n > 0 {
									bCopy := make([]byte, n)
									copy(bCopy, buf[:n])
									seq++
									outMote := &portalpb.Mote{
										StreamId: sid,
										SeqId:    seq,
										Payload: &portalpb.Mote_Tcp{
											Tcp: &portalpb.TCPPayload{
												Data: bCopy,
											},
										},
									}
									m.Publish(ctx, m.TopicOut(portal.ID), outMote)
								}
								if err != nil {
									// Send close mote
									seq++
									m.Publish(ctx, m.TopicOut(portal.ID), &portalpb.Mote{
										StreamId: sid,
										SeqId:    seq,
										Payload: &portalpb.Mote_Bytes{
											Bytes: &portalpb.BytesPayload{
												Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE,
											},
										},
									})
									return
								}
							}
						}(c, streamID)
					}

					// Write to ssh connection
					conn.Write(tcp.Data)
				}
			}
		}
	}()

	// Wait for connection/shell setup message
	msgType, msg, err := wsConn.ReadMessage()
	require.NoError(t, err)
	require.Equal(t, websocket.TextMessage, msgType)
	require.Contains(t, string(msg), "Welcome to the mock shell")

	// Send a command over WS
	err = wsConn.WriteMessage(websocket.TextMessage, []byte("hello server\n"))
	require.NoError(t, err)

	// Wait for echo
	msgType, msg, err = wsConn.ReadMessage()
	require.NoError(t, err)
	require.Equal(t, websocket.TextMessage, msgType)
	require.Contains(t, string(msg), "Echo: hello server")
}
