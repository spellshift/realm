package ssh_test

import (
	"bytes"
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/rsa"
	"encoding/binary"
	"encoding/json"
	"encoding/pem"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/http/httptest"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	gossh "golang.org/x/crypto/ssh"

	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/epb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/network"
	"realm.pub/tavern/internal/ent/networkhost"
	"realm.pub/tavern/internal/http/shell"
	"realm.pub/tavern/internal/portals/mux"
	portalssh "realm.pub/tavern/internal/portals/ssh"
	"realm.pub/tavern/portals/portalpb"

	_ "github.com/mattn/go-sqlite3"
	_ "realm.pub/tavern/internal/ent/runtime"
)

type sshTestEnv struct {
	EntClient *ent.Client
	Mux       *mux.Mux
	Server    *httptest.Server
	WSURL     string
	TaskEnt   *ent.Task
	ShellEnt  *ent.Shell
}

func (env *sshTestEnv) Close() {
	env.Server.Close()
	env.EntClient.Close()
}

func setupSSHTestEnv(t *testing.T) *sshTestEnv {
	t.Helper()
	ctx := context.Background()

	entClient := enttest.OpenTempDB(t)
	portalMux := mux.New()

	u, err := entClient.User.Create().
		SetOauthID("ssh-test-user").
		SetName("SSH Test User").
		SetPhotoURL("http://example.com/photo.jpg").
		SetSessionToken("ssh-test-token").
		Save(ctx)
	require.NoError(t, err)

	h, err := entClient.Host.Create().
		SetName("ssh-host").
		SetIdentifier("ssh-host-1").
		SetPrimaryIP("127.0.0.1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		Save(ctx)
	require.NoError(t, err)

	b, err := entClient.Beacon.Create().
		SetIdentifier("ssh-beacon").
		SetName("ssh-beacon").
		SetHost(h).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
		Save(ctx)
	require.NoError(t, err)

	tom, err := entClient.Tome.Create().
		SetName("ssh-tome").
		SetDescription("ssh tome").
		SetAuthor("ssh-user").
		SetTactic("COMMAND_AND_CONTROL").
		SetEldritch("print('hello')").
		Save(ctx)
	require.NoError(t, err)

	q, err := entClient.Quest.Create().
		SetName("ssh-quest").
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

	handler := portalssh.NewHandler(entClient, portalMux)
	server := httptest.NewServer(http.HandlerFunc(handler.ServeHTTP))
	wsURL := "ws" + strings.TrimPrefix(server.URL, "http")

	return &sshTestEnv{
		EntClient: entClient,
		Mux:       portalMux,
		Server:    server,
		WSURL:     wsURL,
		TaskEnt:   task,
		ShellEnt:  sh,
	}
}

// startTestSSHServer starts an in-process SSH server that echoes PTY input with a greeting.
func startTestSSHServer(t *testing.T, expectedUser, expectedPass string) (string, func()) {
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

	var wg sync.WaitGroup
	ctx, cancel := context.WithCancel(context.Background())

	wg.Add(1)
	go func() {
		defer wg.Done()
		for {
			conn, err := listener.Accept()
			if err != nil {
				return
			}
			go func(nConn net.Conn) {
				defer nConn.Close()
				_, chans, reqs, err := gossh.NewServerConn(nConn, config)
				if err != nil {
					return
				}
				go gossh.DiscardRequests(reqs)

				for newChannel := range chans {
					if newChannel.ChannelType() != "session" {
						_ = newChannel.Reject(gossh.UnknownChannelType, "unknown channel type")
						continue
					}
					channel, requests, err := newChannel.Accept()
					if err != nil {
						continue
					}

					go func(in <-chan *gossh.Request) {
						for req := range in {
							switch req.Type {
							case "pty-req", "shell":
								if req.WantReply {
									_ = req.Reply(true, nil)
								}
								if req.Type == "shell" {
									_, _ = channel.Write([]byte("Welcome to Test SSH\r\n"))
								}
							case "window-change":
								if len(req.Payload) >= 8 {
									cols := binary.BigEndian.Uint32(req.Payload[0:4])
									rows := binary.BigEndian.Uint32(req.Payload[4:8])
									_, _ = channel.Write([]byte(fmt.Sprintf("RESIZED:%dx%d\r\n", cols, rows)))
								}
							default:
								if req.WantReply {
									_ = req.Reply(false, nil)
								}
							}
						}
					}(requests)

					go func() {
						defer channel.Close()
						buf := make([]byte, 1024)
						for {
							select {
							case <-ctx.Done():
								return
							default:
							}
							n, err := channel.Read(buf)
							if n > 0 {
								chunk := string(buf[:n])
								if strings.Contains(chunk, "exit") {
									_, _ = channel.Write([]byte("bye\r\n"))
									return
								}
								_, _ = channel.Write([]byte("ECHO:" + chunk))
							}
							if err != nil {
								return
							}
						}
					}()
				}
			}(conn)
		}
	}()

	cleanup := func() {
		cancel()
		_ = listener.Close()
		wg.Wait()
	}

	return listener.Addr().String(), cleanup
}

// TestSSHPortalNegotiationOnGRPCInstance verifies that:
// 1. Connecting to the SSH WebSocket publishes an SSH_PAYLOAD_KIND_CONNECT mote on TopicIn
//    (instead of negotiating SSH on the WebSocket handler side).
// 2. The gRPC-side Mux intercepts SSHPayload motes, negotiates the SSH handshake
//    over TCP motes with the agent, and publishes decrypted SSH_PAYLOAD_KIND_DATA motes to TopicOut.
// 3. Input, resize, and close messages work end-to-end and closed pivot history is preserved on reconnect.
func TestSSHPortalNegotiationOnGRPCInstance(t *testing.T) {
	env := setupSSHTestEnv(t)
	defer env.Close()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	sshAddr, cleanupSSH := startTestSSHServer(t, "operator", "secret123")
	defer cleanupSSH()

	// 1. Create Portal on the gRPC-receiving side
	portalID, agentCleanup, err := env.Mux.CreatePortal(ctx, env.EntClient, env.TaskEnt.ID, 0)
	require.NoError(t, err)
	defer agentCleanup()

	topicIn := env.Mux.TopicIn(portalID)
	topicOut := env.Mux.TopicOut(portalID)
	inCh, cleanupIn := env.Mux.Subscribe(topicIn)
	defer cleanupIn()

	// Track whether SSH_PAYLOAD_KIND_CONNECT was received on TopicIn
	connectMoteSeen := make(chan *portalpb.SSHPayload, 1)

	// Simulate the gRPC-side Tavern instance (CreatePortal + ssh.Mux)
	// and the connected agent TCP proxy.
	var (
		tcpConnsMu sync.Mutex
		tcpConns   = make(map[string]net.Conn)
		sshMux     *portalssh.Mux
	)

	// sendToAgent simulates sending a TCP mote over gRPC to the agent, which dials the target TCP address
	sendToAgent := func(mote *portalpb.Mote) error {
		tcpPayload := mote.GetTcp()
		if tcpPayload == nil {
			return nil
		}
		streamID := mote.StreamId

		tcpConnsMu.Lock()
		conn, exists := tcpConns[streamID]
		if !exists {
			targetAddr := net.JoinHostPort(tcpPayload.DstAddr, strconv.Itoa(int(tcpPayload.DstPort)))
			var err error
			conn, err = net.DialTimeout("tcp", targetAddr, 5*time.Second)
			if err != nil {
				tcpConnsMu.Unlock()
				_ = sshMux.RouteAgentMote(&portalpb.Mote{
					StreamId: streamID,
					Payload: &portalpb.Mote_Bytes{
						Bytes: &portalpb.BytesPayload{
							Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE,
						},
					},
				})
				return nil
			}
			tcpConns[streamID] = conn

			// Agent TCP reader -> RouteAgentMote on gRPC Tavern instance
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

	sshMux = portalssh.New(portalID, env.Mux, sendToAgent)
	defer sshMux.Close()

	// gRPC-side TopicIn loop (mirrors sendPortalInput in api_create_portal.go)
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
					if sshPayload.Kind == portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CONNECT {
						select {
						case connectMoteSeen <- sshPayload:
						default:
						}
					}
					sshMux.HandleMote(ctx, mote)
				}
			}
		}
	}()
	_ = topicOut

	// 2. Connect browser WebSocket to SSH Handler
	wsURL := fmt.Sprintf("%s?portal_id=%d&shell_id=%d&target=%s",
		env.WSURL, portalID, env.ShellEnt.ID, fmt.Sprintf("operator:secret123@%s", sshAddr))
	ws, _, err := websocket.DefaultDialer.Dial(wsURL, nil)
	require.NoError(t, err)
	defer ws.Close()

	// 3. Verify SSH_PAYLOAD_KIND_CONNECT mote was sent over pubsub TopicIn
	select {
	case connectPayload := <-connectMoteSeen:
		assert.Equal(t, "operator", connectPayload.Username)
		assert.Equal(t, "secret123", connectPayload.Password)
		assert.Equal(t, "127.0.0.1", connectPayload.DstAddr)
	case <-time.After(5 * time.Second):
		t.Fatal("timed out waiting for SSH_PAYLOAD_KIND_CONNECT mote on TopicIn")
	}

	// 4. Read initial SSH welcome banner over WebSocket
	waitForWSOutput := func(conn *websocket.Conn, substr string) string {
		var combined strings.Builder
		deadline := time.Now().Add(5 * time.Second)
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

	// 5. Send interactive SSH input from WebSocket -> verify echoed output
	require.NoError(t, ws.WriteJSON(shell.WebsocketTaskInputMessage{
		Kind:  shell.WebsocketMessageKindInput,
		Input: "id\n",
	}))
	waitForWSOutput(ws, "ECHO:id\n")

	// 6. Send window resize control message -> verify SSH window-change request
	require.NoError(t, ws.WriteJSON(portalssh.WebsocketControlMessage{
		Kind: "resize",
		Rows: 50,
		Cols: 132,
	}))
	waitForWSOutput(ws, "RESIZED:132x50")

	// 7. Trigger remote SSH session exit -> verify websocket receives "closed" status and DB records ClosedAt & Data
	require.NoError(t, ws.WriteJSON(shell.WebsocketTaskInputMessage{
		Kind:  shell.WebsocketMessageKindInput,
		Input: "exit\n",
	}))

	var sawClosedStatus bool
	deadline := time.Now().Add(5 * time.Second)
	ws.SetReadDeadline(deadline)
	for time.Now().Before(deadline) {
		_, data, err := ws.ReadMessage()
		if err != nil {
			if err == io.EOF || websocket.IsCloseError(err, websocket.CloseNormalClosure, websocket.CloseAbnormalClosure) {
				break
			}
			break
		}
		var statusMsg shell.WebsocketStatusMessage
		if err := json.Unmarshal(data, &statusMsg); err == nil && statusMsg.Kind == shell.WebsocketMessageKindStatus && statusMsg.Status == "closed" {
			sawClosedStatus = true
			break
		}
	}
	assert.True(t, sawClosedStatus, "expected closed status message when SSH session terminates")

	// 8. Reconnect to the closed pivot by pivot_id and verify saved history + closed status are returned
	pivots, err := env.EntClient.ShellPivot.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, pivots, 1)
	pivot := pivots[0]
	assert.False(t, pivot.ClosedAt.IsZero(), "ShellPivot ClosedAt should be set")
	assert.Contains(t, pivot.Data, "Welcome to Test SSH")

	reconnectURL := fmt.Sprintf("%s?pivot_id=%d", env.WSURL, pivot.ID)
	wsReconnect, _, err := websocket.DefaultDialer.Dial(reconnectURL, nil)
	require.NoError(t, err)
	defer wsReconnect.Close()

	waitForWSOutput(wsReconnect, "Welcome to Test SSH")

	// 9. Verify that a successful SSH pivot recorded NetworkHost and Network entities
	createdNetwork, err := env.EntClient.Network.Query().
		Where(network.Subnet("127.0.0.0/24")).
		Only(ctx)
	require.NoError(t, err)
	assert.Equal(t, "127.0.0.0/24", createdNetwork.Subnet)

	createdNetworkHost, err := env.EntClient.NetworkHost.Query().
		Where(networkhost.IPAddress("127.0.0.1")).
		WithNetworks().
		Only(ctx)
	require.NoError(t, err)
	assert.Equal(t, "127.0.0.1", createdNetworkHost.IPAddress)
	assert.Equal(t, "127.0.0.0/24", createdNetworkHost.Subnet)
	assert.Contains(t, createdNetworkHost.OpenPorts, pivot.Port)
	require.NotEmpty(t, createdNetworkHost.Edges.Networks)
	assert.Equal(t, createdNetwork.ID, createdNetworkHost.Edges.Networks[0].ID)
}

func startTestSSHKeyServer(t *testing.T, expectedUser string, expectedPubKey gossh.PublicKey) (string, func()) {
	t.Helper()

	hostKey, err := rsa.GenerateKey(rand.Reader, 2048)
	require.NoError(t, err)
	signer, err := gossh.NewSignerFromKey(hostKey)
	require.NoError(t, err)

	config := &gossh.ServerConfig{
		PublicKeyCallback: func(c gossh.ConnMetadata, pubKey gossh.PublicKey) (*gossh.Permissions, error) {
			if c.User() == expectedUser && bytes.Equal(pubKey.Marshal(), expectedPubKey.Marshal()) {
				return nil, nil
			}
			return nil, fmt.Errorf("public key rejected for %q", c.User())
		},
	}
	config.AddHostKey(signer)

	listener, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(t, err)

	var wg sync.WaitGroup
	ctx, cancel := context.WithCancel(context.Background())

	wg.Add(1)
	go func() {
		defer wg.Done()
		for {
			conn, err := listener.Accept()
			if err != nil {
				return
			}
			go func(nConn net.Conn) {
				defer nConn.Close()
				_, chans, reqs, err := gossh.NewServerConn(nConn, config)
				if err != nil {
					return
				}
				go gossh.DiscardRequests(reqs)

				for newChannel := range chans {
					if newChannel.ChannelType() != "session" {
						_ = newChannel.Reject(gossh.UnknownChannelType, "unknown channel type")
						continue
					}
					channel, requests, err := newChannel.Accept()
					if err != nil {
						continue
					}

					go func(in <-chan *gossh.Request) {
						for req := range in {
							switch req.Type {
							case "pty-req", "shell":
								if req.WantReply {
									_ = req.Reply(true, nil)
								}
								if req.Type == "shell" {
									_, _ = channel.Write([]byte("Key Auth SSH Ready\r\n"))
								}
							default:
								if req.WantReply {
									_ = req.Reply(false, nil)
								}
							}
						}
					}(requests)

					go func() {
						defer channel.Close()
						buf := make([]byte, 1024)
						for {
							select {
							case <-ctx.Done():
								return
							default:
							}
							n, err := channel.Read(buf)
							if n > 0 {
								_, _ = channel.Write([]byte("KEY_ECHO:" + string(buf[:n])))
							}
							if err != nil {
								return
							}
						}
					}()
				}
			}(conn)
		}
	}()

	cleanup := func() {
		cancel()
		_ = listener.Close()
		wg.Wait()
	}

	return listener.Addr().String(), cleanup
}

func TestSSHPortalWithSSHKeyCredentialAndPortalUpgrade(t *testing.T) {
	env := setupSSHTestEnv(t)
	defer env.Close()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Generate Ed25519 client key pair and PEM encode private key
	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	require.NoError(t, err)
	sshPubKey, err := gossh.NewPublicKey(pubKey)
	require.NoError(t, err)
	pemBlock, err := gossh.MarshalPrivateKey(privKey, "")
	require.NoError(t, err)
	pemBytes := pem.EncodeToMemory(pemBlock)

	sshAddr, cleanupSSH := startTestSSHKeyServer(t, "deploy", sshPubKey)
	defer cleanupSSH()

	// Store SSH key as a HostCredential
	hostEnt, err := env.TaskEnt.QueryBeacon().QueryHost().Only(ctx)
	require.NoError(t, err)
	credEnt, err := env.EntClient.HostCredential.Create().
		SetHost(hostEnt).
		SetTask(env.TaskEnt).
		SetPrincipal("deploy").
		SetSecret(string(pemBytes)).
		SetKind(epb.Credential_KIND_SSH_KEY).
		Save(ctx)
	require.NoError(t, err)

	// Create an initial portal (stale) and then a second active portal on the same beacon
	stalePortalID, staleCleanup, err := env.Mux.CreatePortal(ctx, env.EntClient, env.TaskEnt.ID, 0)
	require.NoError(t, err)
	defer staleCleanup()

	activePortalID, activeCleanup, err := env.Mux.CreatePortal(ctx, env.EntClient, env.TaskEnt.ID, 0)
	require.NoError(t, err)
	defer activeCleanup()
	require.NotEqual(t, stalePortalID, activePortalID)

	// Verify the stale portal was automatically closed when the new portal was created for the beacon
	stalePortalEnt, err := env.EntClient.Portal.Get(ctx, stalePortalID)
	require.NoError(t, err)
	assert.False(t, stalePortalEnt.ClosedAt.IsZero(), "previous portal on beacon should be marked closed")

	topicIn := env.Mux.TopicIn(activePortalID)
	inCh, cleanupIn := env.Mux.Subscribe(topicIn)
	defer cleanupIn()

	var (
		tcpConnsMu sync.Mutex
		tcpConns   = make(map[string]net.Conn)
		sshMux     *portalssh.Mux
	)

	sendToAgent := func(mote *portalpb.Mote) error {
		tcpPayload := mote.GetTcp()
		if tcpPayload == nil {
			return nil
		}
		streamID := mote.StreamId

		tcpConnsMu.Lock()
		conn, exists := tcpConns[streamID]
		if !exists {
			targetAddr := net.JoinHostPort(tcpPayload.DstAddr, strconv.Itoa(int(tcpPayload.DstPort)))
			var err error
			conn, err = net.DialTimeout("tcp", targetAddr, 5*time.Second)
			if err != nil {
				tcpConnsMu.Unlock()
				return nil
			}
			tcpConns[streamID] = conn

			go func(c net.Conn, sid string) {
				defer c.Close()
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

	sshMux = portalssh.New(activePortalID, env.Mux, sendToAgent)
	defer sshMux.Close()

	go func() {
		for {
			select {
			case <-ctx.Done():
				return
			case mote, ok := <-inCh:
				if !ok || mote == nil {
					return
				}
				if mote.GetSsh() != nil {
					sshMux.HandleMote(ctx, mote)
				}
			}
		}
	}()

	// Dial WebSocket using stalePortalID and credential_id (without embedding secret in target)
	wsURL := fmt.Sprintf("%s?portal_id=%d&shell_id=%d&target=%s&credential_id=%d",
		env.WSURL, stalePortalID, env.ShellEnt.ID, fmt.Sprintf("deploy@%s", sshAddr), credEnt.ID)
	ws, _, err := websocket.DefaultDialer.Dial(wsURL, nil)
	require.NoError(t, err)
	defer ws.Close()

	var combined strings.Builder
	deadline := time.Now().Add(5 * time.Second)
	ws.SetReadDeadline(deadline)
	for time.Now().Before(deadline) {
		_, data, err := ws.ReadMessage()
		if err != nil {
			break
		}
		var msg shell.WebsocketTaskOutputMessage
		if err := json.Unmarshal(data, &msg); err == nil && msg.Kind == shell.WebsocketMessageKindOutput {
			combined.WriteString(msg.Output)
			if strings.Contains(combined.String(), "Key Auth SSH Ready") {
				break
			}
		}
	}
	assert.Contains(t, combined.String(), "Key Auth SSH Ready")
}

func TestSSHMuxDecoupledStreamIDsAndPreBuffering(t *testing.T) {
	portalMux := mux.New()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	sshAddr, cleanupSSH := startTestSSHServer(t, "user1", "pass1")
	defer cleanupSSH()

	host, portStr, err := net.SplitHostPort(sshAddr)
	require.NoError(t, err)
	port, err := strconv.Atoi(portStr)
	require.NoError(t, err)

	portalID := 99
	topicIn := portalMux.TopicIn(portalID)
	topicOut := portalMux.TopicOut(portalID)

	outCh, cleanupOut := portalMux.Subscribe(topicOut)
	defer cleanupOut()

	var (
		seenTCPStreamID string
		tcpConnsMu      sync.Mutex
		tcpConns        = make(map[string]net.Conn)
		sshMux          *portalssh.Mux
	)

	sendToAgent := func(mote *portalpb.Mote) error {
		tcpPayload := mote.GetTcp()
		if tcpPayload == nil {
			return nil
		}
		sid := mote.StreamId
		tcpConnsMu.Lock()
		if seenTCPStreamID == "" {
			seenTCPStreamID = sid
		}
		conn, exists := tcpConns[sid]
		if !exists {
			targetAddr := net.JoinHostPort(tcpPayload.DstAddr, strconv.Itoa(int(tcpPayload.DstPort)))
			var dialErr error
			conn, dialErr = net.DialTimeout("tcp", targetAddr, 5*time.Second)
			if dialErr != nil {
				tcpConnsMu.Unlock()
				return dialErr
			}
			tcpConns[sid] = conn

			go func(c net.Conn, streamID string) {
				defer func() {
					c.Close()
					tcpConnsMu.Lock()
					delete(tcpConns, streamID)
					tcpConnsMu.Unlock()
					_ = sshMux.RouteAgentMote(&portalpb.Mote{
						StreamId: streamID,
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
					n, rErr := c.Read(buf)
					if n > 0 {
						chunk := make([]byte, n)
						copy(chunk, buf[:n])
						_ = sshMux.RouteAgentMote(&portalpb.Mote{
							StreamId: streamID,
							SeqId:    seq,
							Payload: &portalpb.Mote_Tcp{
								Tcp: &portalpb.TCPPayload{
									Data: chunk,
								},
							},
						})
						seq++
					}
					if rErr != nil {
						return
					}
				}
			}(conn, sid)
		}
		tcpConnsMu.Unlock()

		if len(tcpPayload.Data) > 0 {
			_, wErr := conn.Write(tcpPayload.Data)
			return wErr
		}
		return nil
	}

	sshMux = portalssh.New(portalID, portalMux, sendToAgent)
	defer sshMux.Close()

	sshStreamID := "ssh-ws-test-12345"

	// 1. Send out-of-order DATA payload BEFORE CONNECT payload arrives
	sshMux.HandleMote(ctx, &portalpb.Mote{
		StreamId: sshStreamID,
		SeqId:    1,
		Payload: &portalpb.Mote_Ssh{
			Ssh: &portalpb.SSHPayload{
				Kind: portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_DATA,
				Data: []byte("whoami\n"),
			},
		},
	})

	// 2. Now send the CONNECT payload
	sshMux.HandleMote(ctx, &portalpb.Mote{
		StreamId: sshStreamID,
		SeqId:    0,
		Payload: &portalpb.Mote_Ssh{
			Ssh: &portalpb.SSHPayload{
				Kind:     portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CONNECT,
				DstAddr:  host,
				DstPort:  uint32(port),
				Username: "user1",
				Password: "pass1",
				Rows:     40,
				Cols:     80,
			},
		},
	})

	// 3. Verify output arriving on TopicOut has StreamId = sshStreamID and contains the welcome + echo of whoami
	var output strings.Builder
	deadline := time.Now().Add(5 * time.Second)
	for time.Now().Before(deadline) {
		select {
		case outMote := <-outCh:
			if outMote.StreamId == sshStreamID && outMote.GetSsh() != nil {
				output.Write(outMote.GetSsh().Data)
				if strings.Contains(output.String(), "ECHO:whoami\n") {
					goto Done
				}
			}
		case <-time.After(100 * time.Millisecond):
		}
	}
Done:
	assert.Contains(t, output.String(), "Welcome to Test SSH")
	assert.Contains(t, output.String(), "ECHO:whoami\n")

	// 4. Verify seenTCPStreamID is a valid fresh UUID and is NOT the sshStreamID
	assert.NotEmpty(t, seenTCPStreamID)
	assert.NotEqual(t, sshStreamID, seenTCPStreamID)
	_, parseErr := uuid.Parse(seenTCPStreamID)
	assert.NoError(t, parseErr, "TCP stream ID must be a valid UUID")
	_ = topicIn
}

func TestSSHMuxSurfacesAgentConnectTimeout(t *testing.T) {
	portalMux := mux.New()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	portalID := 100
	topicOut := portalMux.TopicOut(portalID)
	outCh, cleanupOut := portalMux.Subscribe(topicOut)
	defer cleanupOut()

	var (
		seenTCPStreamID string
		sshMux          *portalssh.Mux
	)

	// Simulate agent failing to connect and sending back a close mote with a timeout error
	sendToAgent := func(mote *portalpb.Mote) error {
		if mote.GetTcp() == nil {
			return nil
		}
		seenTCPStreamID = mote.StreamId
		go func(sid string) {
			time.Sleep(50 * time.Millisecond)
			_ = sshMux.RouteAgentMote(&portalpb.Mote{
				StreamId: sid,
				SeqId:    0,
				Payload: &portalpb.Mote_Bytes{
					Bytes: &portalpb.BytesPayload{
						Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE,
						Data: []byte("tcp connect timeout: 192.0.2.1:22"),
					},
				},
			})
		}(mote.StreamId)
		return nil
	}

	sshMux = portalssh.New(portalID, portalMux, sendToAgent)
	defer sshMux.Close()

	sshStreamID := "ssh-ws-timeout-test"

	sshMux.HandleMote(ctx, &portalpb.Mote{
		StreamId: sshStreamID,
		SeqId:    0,
		Payload: &portalpb.Mote_Ssh{
			Ssh: &portalpb.SSHPayload{
				Kind:     portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CONNECT,
				DstAddr:  "192.0.2.1",
				DstPort:  22,
				Username: "user",
				Password: "password",
			},
		},
	})

	var closeReason string
	deadline := time.Now().Add(5 * time.Second)
	for time.Now().Before(deadline) {
		select {
		case outMote := <-outCh:
			if outMote.StreamId == sshStreamID && outMote.GetSsh() != nil && outMote.GetSsh().Kind == portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CLOSE {
				closeReason = string(outMote.GetSsh().Data)
				goto TimeoutDone
			}
		case <-time.After(50 * time.Millisecond):
		}
	}
TimeoutDone:
	assert.Contains(t, closeReason, "tcp connect timeout: 192.0.2.1:22")
	assert.NotEmpty(t, seenTCPStreamID)
	assert.NotEqual(t, sshStreamID, seenTCPStreamID)
}
