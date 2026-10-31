package ssh_test

import (
	"context"
	"crypto/rand"
	"crypto/rsa"
	"encoding/json"
	"fmt"
	"net"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
	"time"

	gpubsub "cloud.google.com/go/pubsub/v2"
	"cloud.google.com/go/pubsub/v2/pstest"
	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/require"
	gossh "golang.org/x/crypto/ssh"
	"google.golang.org/api/option"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/http/shell"
	"realm.pub/tavern/internal/portals/mux"
	xpubsub "realm.pub/tavern/internal/portals/pubsub"
	portalssh "realm.pub/tavern/internal/portals/ssh"
	"realm.pub/tavern/portals/portalpb"
)

func TestSSHHandler_GCPPubSubMultiInstance(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// 1. Start pstest PubSub server
	psSrv := pstest.NewServer()
	defer psSrv.Close()

	conn, err := grpc.NewClient(psSrv.Addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	require.NoError(t, err)
	defer conn.Close()

	projectID := "test-project"

	// Client A: Server A (WebSocket)
	gcpClientA, err := gpubsub.NewClient(ctx, projectID, option.WithGRPCConn(conn))
	require.NoError(t, err)
	defer gcpClientA.Close()
	driverA := xpubsub.NewClient(xpubsub.WithGCPDriver("server-A", gcpClientA))
	defer driverA.Close()
	muxA := mux.New(mux.WithPubSubClient(driverA))

	// Client B: Server B (gRPC / ssh.Mux)
	gcpClientB, err := gpubsub.NewClient(ctx, projectID, option.WithGRPCConn(conn))
	require.NoError(t, err)
	defer gcpClientB.Close()
	driverB := xpubsub.NewClient(xpubsub.WithGCPDriver("server-B", gcpClientB))
	defer driverB.Close()
	muxB := mux.New(mux.WithPubSubClient(driverB))

	// DB shared between instances
	entClient := enttest.OpenTempDB(t)

	u, err := entClient.User.Create().
		SetOauthID("ssh-user-gcp").
		SetName("SSH GCP User").
		SetPhotoURL("http://example.com/photo.jpg").
		SetSessionToken("ssh-gcp-token").
		Save(ctx)
	require.NoError(t, err)

	h, err := entClient.Host.Create().
		SetName("ssh-host-gcp").
		SetIdentifier("ssh-host-gcp-1").
		SetPrimaryIP("127.0.0.1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		Save(ctx)
	require.NoError(t, err)

	b, err := entClient.Beacon.Create().
		SetIdentifier("ssh-beacon-gcp").
		SetName("ssh-beacon-gcp").
		SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
		Save(ctx)
	require.NoError(t, err)

	tom, err := entClient.Tome.Create().
		SetName("ssh-tome-gcp").
		SetDescription("ssh tome").
		SetAuthor("ssh-user").
		SetTactic("COMMAND_AND_CONTROL").
		SetEldritch("print('hello')").
		Save(ctx)
	require.NoError(t, err)

	q, err := entClient.Quest.Create().
		SetName("ssh-quest-gcp").
		SetCreator(u).
		SetTome(tom).
		Save(ctx)
	require.NoError(t, err)

	task, err := entClient.Task.Create().
		SetBeacon(b).
		SetQuest(q).
		Save(ctx)
	require.NoError(t, err)

	sh, err := entClient.Shell.Create().
		SetBeacon(b).
		SetOwner(u).
		SetData([]byte{}).
		Save(ctx)
	require.NoError(t, err)

	// Start Mock SSH Server
	sshAddr, cleanupSSH := startGCPTestSSHServer(t, "operator", "secret123")
	defer cleanupSSH()

	// Server B creates the portal (as would happen in api_create_portal.go)
	portalID, cleanupPortalB, err := muxB.CreatePortal(ctx, entClient, task.ID, 0)
	require.NoError(t, err)
	defer cleanupPortalB()

	topicIn := muxB.TopicIn(portalID)
	inCh, cleanupIn := muxB.Subscribe(topicIn)
	defer cleanupIn()

	// Server B sets up ssh.Mux
	var tcpConnsMu sync.Mutex
	tcpConns := make(map[string]net.Conn)
	var sshMux *portalssh.Mux

	sendToAgent := func(mote *portalpb.Mote) error {
		tcpPayload := mote.GetTcp()
		if tcpPayload == nil {
			return nil
		}
		streamID := mote.StreamId

		tcpConnsMu.Lock()
		conn, exists := tcpConns[streamID]
		if !exists {
			var err error
			conn, err = net.Dial("tcp", sshAddr)
			if err != nil {
				tcpConnsMu.Unlock()
				return err
			}
			tcpConns[streamID] = conn

			go func(c net.Conn, sid string) {
				defer func() {
					c.Close()
					tcpConnsMu.Lock()
					delete(tcpConns, sid)
					tcpConnsMu.Unlock()
					_ = sshMux.RouteAgentMote(&portalpb.Mote{
						StreamId: sid,
						Payload: &portalpb.Mote_Bytes{
							Bytes: &portalpb.BytesPayload{
								Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE,
							},
						},
					})
				}()
				buf := make([]byte, 4096)
				var seq uint64
				for {
					n, err := c.Read(buf)
					if n > 0 {
						chunk := make([]byte, n)
						copy(chunk, buf[:n])
						_ = sshMux.RouteAgentMote(&portalpb.Mote{
							StreamId: sid,
							SeqId:    seq,
							Payload: &portalpb.Mote_Tcp{
								Tcp: &portalpb.TCPPayload{
									Data: chunk,
								},
							},
						})
						seq++
					}
					if err != nil {
						return
					}
				}
			}(conn, streamID)
		}
		tcpConnsMu.Unlock()

		if len(tcpPayload.Data) > 0 {
			_, err := conn.Write(tcpPayload.Data)
			return err
		}
		return nil
	}

	sshMux = portalssh.New(portalID, muxB, sendToAgent)
	defer sshMux.Close()

	// Server B TopicIn loop (mirrors sendPortalInput in api_create_portal.go)
	go func() {
		for {
			select {
			case <-ctx.Done():
				return
			case mote, ok := <-inCh:
				if !ok || mote == nil {
					return
				}
				if sshPayload := mote.GetSsh(); sshPayload != nil {
					sshMux.HandleMote(ctx, mote)
				}
			}
		}
	}()

	// Server A runs HTTP WebSocket handler
	handlerA := portalssh.NewHandler(entClient, muxA)
	serverA := httptest.NewServer(http.HandlerFunc(handlerA.ServeHTTP))
	defer serverA.Close()
	wsURL := "ws" + strings.TrimPrefix(serverA.URL, "http")

	// Connect browser WebSocket to Server A
	dialURL := fmt.Sprintf("%s?portal_id=%d&shell_id=%d&target=%s",
		wsURL, portalID, sh.ID, fmt.Sprintf("operator:secret123@%s", sshAddr))
	ws, _, err := websocket.DefaultDialer.Dial(dialURL, nil)
	require.NoError(t, err)
	defer ws.Close()

	// Wait for welcome banner
	waitForWSOutput := func(conn *websocket.Conn, substr string) string {
		var combined strings.Builder
		deadline := time.Now().Add(10 * time.Second)
		conn.SetReadDeadline(deadline)
		for time.Now().Before(deadline) {
			_, data, err := conn.ReadMessage()
			if err != nil {
				break
			}
			var msg shell.WebsocketTaskOutputMessage
			if err := json.Unmarshal(data, &msg); err == nil && msg.Kind == shell.WebsocketMessageKindOutput {
				combined.WriteString(msg.Output)
				if strings.Contains(combined.String(), substr) {
					return combined.String()
				}
			}
		}
		t.Fatalf("did not receive expected output %q, got %q", substr, combined.String())
		return ""
	}

	waitForWSOutput(ws, "Welcome to Test SSH")

	// Send interactive input
	require.NoError(t, ws.WriteJSON(shell.WebsocketTaskInputMessage{
		Kind:  shell.WebsocketMessageKindInput,
		Input: "id\n",
	}))
	waitForWSOutput(ws, "ECHO:id\n")
}

func startGCPTestSSHServer(t *testing.T, expectedUser, expectedPass string) (string, func()) {
	t.Helper()

	privateKey, err := rsa.GenerateKey(rand.Reader, 2048)
	require.NoError(t, err)
	signer, err := gossh.NewSignerFromKey(privateKey)
	require.NoError(t, err)

	config := &gossh.ServerConfig{
		PasswordCallback: func(c gossh.ConnMetadata, pass []byte) (*gossh.Permissions, error) {
			if c.User() == expectedUser && string(pass) == expectedPass {
				return nil, nil
			}
			return nil, fmt.Errorf("password rejected for %q", c.User())
		},
	}
	config.AddHostKey(signer)

	listener, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(t, err)

	go func() {
		for {
			tcpConn, err := listener.Accept()
			if err != nil {
				return
			}
			go func(c net.Conn) {
				defer c.Close()
				sConn, chans, reqs, err := gossh.NewServerConn(c, config)
				if err != nil {
					return
				}
				defer sConn.Close()
				go gossh.DiscardRequests(reqs)

				for newChannel := range chans {
					if newChannel.ChannelType() != "session" {
						newChannel.Reject(gossh.UnknownChannelType, "unknown channel type")
						continue
					}
					ch, reqsCh, err := newChannel.Accept()
					if err != nil {
						return
					}
					go func(channel gossh.Channel, inReqs <-chan *gossh.Request) {
						defer channel.Close()
						for req := range inReqs {
							switch req.Type {
							case "pty-req":
								req.Reply(true, nil)
							case "shell":
								req.Reply(true, nil)
								channel.Write([]byte("Welcome to Test SSH\r\n"))
								buf := make([]byte, 1024)
								for {
									n, err := channel.Read(buf)
									if err != nil {
										return
									}
									channel.Write([]byte(fmt.Sprintf("ECHO:%s", string(buf[:n]))))
								}
							default:
								req.Reply(false, nil)
							}
						}
					}(ch, reqsCh)
				}
			}(tcpConn)
		}
	}()

	return listener.Addr().String(), func() {
		listener.Close()
	}
}
