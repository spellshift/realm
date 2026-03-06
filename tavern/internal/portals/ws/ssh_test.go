package ws

import (
	"context"
	"fmt"
	"net"
	"net/http/httptest"
	"net/url"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	_ "github.com/mattn/go-sqlite3"
	"golang.org/x/crypto/ssh"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
)

const dummyPrivateKey = `-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACBQ1piRVJ/8KkNWIrsJO57Gw2bMIUyqofSFFvj4oZdydwAAAJC/am57v2pu
ewAAAAtzc2gtZWQyNTUxOQAAACBQ1piRVJ/8KkNWIrsJO57Gw2bMIUyqofSFFvj4oZdydw
AAAECUWdYdkOpTHL3OmCs2jWOZ6YpF0CwTLpkoAD3YYcZXw1DWmJFUn/wqQ1Yiuwk7nsbD
ZswhTKqh9IUW+Pihl3J3AAAADGp1bGVzQGRldmJveAE=
-----END OPENSSH PRIVATE KEY-----`

func TestSSHPortalHandler(t *testing.T) {
	ctx := context.Background()

	// 1. Setup Mock DB & Portal Mux
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1&_busy_timeout=10000")
	defer client.Close()

	portalMux := mux.New()

	// Create a User and Portal
	user := client.User.Create().SetName("admin").SetOauthID("oauth").SetPhotoURL("photo").SaveX(ctx)
	h := client.Host.Create().SetName("host").SetIdentifier("ident").SetPlatform(c2pb.Host_PLATFORM_LINUX).SaveX(ctx)
	beacon := client.Beacon.Create().SetName("testbeacon").SetTransport(c2pb.Transport_TRANSPORT_HTTP1).SetHost(h).SaveX(ctx)
	tomeEnt := client.Tome.Create().SetName("testtome").SetDescription("desc").SetAuthor(user.Name).SetUploader(user).SetEldritch("nop").SaveX(ctx)
	quest := client.Quest.Create().SetName("testquest").SetParameters("").SetCreator(user).SetTome(tomeEnt).SaveX(ctx)
	tCreate := client.Task.Create().SetQuest(quest).SetBeacon(beacon).SaveX(ctx)
	portalID, _, err := portalMux.CreatePortal(ctx, client, tCreate.ID, 0)
	if err != nil {
		t.Fatalf("failed to create portal: %v", err)
	}
	portal, _ := client.Portal.Get(ctx, portalID)

	// 2. Setup Local SSH Server (Echo Server)
	sshServerListener, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("Failed to listen for mock ssh server: %v", err)
	}
	defer sshServerListener.Close()

	sshServerConfig := &ssh.ServerConfig{
		PasswordCallback: func(c ssh.ConnMetadata, pass []byte) (*ssh.Permissions, error) {
			if c.User() == "testuser" && string(pass) == "testpass" {
				return nil, nil
			}
			return nil, fmt.Errorf("password rejected for %q", c.User())
		},
	}
	signer, err := ssh.ParsePrivateKey([]byte(dummyPrivateKey))
	if err != nil {
		t.Fatalf("Failed to parse dummy private key: %v", err)
	}
	sshServerConfig.AddHostKey(signer)

	go func() {
		for {
			tcpConn, err := sshServerListener.Accept()
			if err != nil {
				return
			}

			sshConn, chans, reqs, err := ssh.NewServerConn(tcpConn, sshServerConfig)
			if err != nil {
				continue
			}
			go ssh.DiscardRequests(reqs)

			go func(newChannel <-chan ssh.NewChannel) {
				for newChannel := range newChannel {
					if newChannel.ChannelType() != "session" {
						newChannel.Reject(ssh.UnknownChannelType, "unknown channel type")
						continue
					}
					channel, requests, err := newChannel.Accept()
					if err != nil {
						continue
					}

					go func(in <-chan *ssh.Request) {
						for req := range in {
							if req.Type == "pty-req" {
								req.Reply(true, nil)
							} else if req.Type == "shell" {
								req.Reply(true, nil)
								// Echo shell
								go func() {
									defer channel.Close()
									buf := make([]byte, 1024)
									for {
										n, err := channel.Read(buf)
										if err != nil {
											break
										}
										// Write back to channel
										channel.Write(buf[:n])
									}
								}()
							} else {
								req.Reply(false, nil)
							}
						}
					}(requests)
				}
			}(chans)
			_ = sshConn
		}
	}()

	// 3. Setup Agent Mock (Translating Portal Motes to TCP)
	topicIn := portalMux.TopicIn(portal.ID)
	recvIn, cleanupIn := portalMux.Subscribe(topicIn, mux.WithHistoryReplay())
	defer cleanupIn()

	go func() {
		var activeConn net.Conn
		var activeStreamID string

		defer func() {
			if activeConn != nil {
				activeConn.Close()
			}
		}()

		var seqOut uint64 = 0

		for mote := range recvIn {

			if bytesPayload := mote.GetBytes(); bytesPayload != nil && bytesPayload.GetKind() == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
				if activeConn != nil && activeStreamID == mote.GetStreamId() {
					activeConn.Close()
					activeConn = nil
				}
				continue
			}

			if tcpPayload := mote.GetTcp(); tcpPayload != nil {
				if activeConn == nil || activeStreamID != mote.GetStreamId() {
					if activeConn != nil {
						activeConn.Close()
					}
					activeStreamID = mote.GetStreamId()
					target := fmt.Sprintf("%s:%d", tcpPayload.GetDstAddr(), tcpPayload.GetDstPort())

					var err error

					activeConn, err = net.Dial("tcp", sshServerListener.Addr().String())
					if err != nil {
						t.Errorf("Agent failed to dial TCP target %s: %v", target, err)
						return
					}

					// Start reading from TCP and publishing to portalMux
					go func(conn net.Conn, streamID string) {
						buf := make([]byte, 32*1024)
						topicOut := portalMux.TopicOut(portal.ID)
						for {
							n, err := conn.Read(buf)
							if n > 0 {
								// Need to copy buffer here!
								dataCopy := make([]byte, n)
								copy(dataCopy, buf[:n])
								outMote := &portalpb.Mote{
									StreamId: streamID,
									SeqId:    seqOut,
									Payload: &portalpb.Mote_Tcp{
										Tcp: &portalpb.TCPPayload{
											Data: dataCopy,
										},
									},
								}
								seqOut++
								portalMux.Publish(context.Background(), topicOut, outMote)
							}
							if err != nil {
								// Send close mote
								closeMote := &portalpb.Mote{
									StreamId: streamID,
									SeqId:    seqOut,
									Payload: &portalpb.Mote_Bytes{
										Bytes: &portalpb.BytesPayload{
											Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE,
										},
									},
								}
								seqOut++
								portalMux.Publish(context.Background(), topicOut, closeMote)
								break
							}
						}
					}(activeConn, activeStreamID)
				}

				// Write TCP data
				data := tcpPayload.GetData()
				if len(data) > 0 {
					activeConn.Write(data)
				}
			}
		}
	}()

	// 4. Setup HTTP Server for Handler
	handler := NewSSHHandler(client, portalMux)
	server := httptest.NewServer(handler)
	defer server.Close()

	// 5. Connect Websocket Client
	u, _ := url.Parse(server.URL)
	u.Scheme = "ws"

	sessionID := "test-session-1"
	targetHostPort := sshServerListener.Addr().String()

	q := u.Query()
	q.Set("portal_id", fmt.Sprintf("%d", portal.ID))
	q.Set("session_id", sessionID)
	q.Set("target", fmt.Sprintf("testuser:testpass@%s", targetHostPort))
	u.RawQuery = q.Encode()

	wsConn, _, err := websocket.DefaultDialer.Dial(u.String(), nil)
	if err != nil {
		t.Fatalf("Failed to dial websocket: %v", err)
	}
	defer wsConn.Close()

	// 6. Test SSH Session Echo
	testString := "echo 'Hello SSH from Tavern Portal!'\r\n"
	err = wsConn.WriteMessage(websocket.TextMessage, []byte(testString))
	if err != nil {
		t.Fatalf("Failed to write to websocket: %v", err)
	}

	wsConn.SetReadDeadline(time.Now().Add(5 * time.Second))
	var receivedOutput string
	for {
		_, msg, err := wsConn.ReadMessage()
		if err != nil {
			if netErr, ok := err.(net.Error); ok && netErr.Timeout() {
				break
			}
			t.Fatalf("Failed to read from websocket: %v", err)
		}
		receivedOutput += string(msg)
		if len(receivedOutput) >= len(testString) {
			break
		}
	}

	if receivedOutput != testString {
		t.Errorf("Expected output %q, got %q", testString, receivedOutput)
	}
}
