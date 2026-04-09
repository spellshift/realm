package ws

import (
	"math/rand"

	"context"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/http/httptest"
	"os/exec"
	"strings"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/require"

	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
)

func TestSSHPortalHandler(t *testing.T) {
	t.Skip("Skipping flaky e2e SSH test")

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	sshPort := 2223 + rand.Intn(10000)

	// 1. Setup mock SSH server (e2e/sshecho) via child process
	cmd := exec.Command("go", "run", "realm.pub/bin/e2e/sshecho", "-p", fmt.Sprintf("%d", sshPort), "-u", "testuser", "--pass", "testpass")
	cmd.Dir = "../../../../"
	err := cmd.Start()
	require.NoError(t, err)
	defer func() {
		if cmd.Process != nil {
			cmd.Process.Kill()
		}
	}()

	// Wait for SSH server to start
	time.Sleep(4 * time.Second)

	// 2. Setup Ent DB & Portal Mux
	client := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer client.Close()

	// Create portal entity in DB to emulate proper OpenPortal startup
	host := client.Host.Create().SetPlatform(c2pb.Host_PLATFORM_LINUX).SetIdentifier("test-id").SetName("test").SaveX(ctx)
	beacon := client.Beacon.Create().SetName("test").SetTransport(c2pb.Transport_TRANSPORT_TCP_BIND).SetHost(host).SaveX(ctx)
	owner := client.User.Create().SetName("test").SetPhotoURL("http://test.com").SetOauthID("test").SaveX(ctx)
	p := client.Portal.Create().SetBeacon(beacon).SetOwner(owner).SaveX(ctx)

	sessionID := p.ID

	m := mux.New()

	// 3. Setup SSHPortalHandler endpoint
	handler := NewSSHPortalHandler(client, m)
	server := httptest.NewServer(handler)
	defer server.Close()

	seqID := uint64(10)
	target := fmt.Sprintf("testuser:testpass@127.0.0.1:%d", sshPort)

	// 4. Setup mock implant side logic
	inTopic := m.TopicIn(sessionID)
	outTopic := m.TopicOut(sessionID)

	// Pre-create publisher topics for mock implant
	err = m.EnsureTopic(ctx, inTopic)
	require.NoError(t, err)
	err = m.EnsureTopic(ctx, outTopic)
	require.NoError(t, err)

	recvIn, cancelIn := m.Subscribe(inTopic)
	defer cancelIn()

	implantConn, err := net.Dial("tcp", fmt.Sprintf("127.0.0.1:%d", sshPort))
	require.NoError(t, err)
	defer implantConn.Close()

	streamIDCh := make(chan string, 1)

	go func() {
		var capturedStreamID string
		for {
			select {
			case <-ctx.Done():
				return
			case mote, ok := <-recvIn:
				if !ok {
					return
				}
				if capturedStreamID == "" && mote.StreamId != "" {
					capturedStreamID = mote.StreamId
					streamIDCh <- capturedStreamID
				}
				if tcpPayload := mote.GetTcp(); tcpPayload != nil {
					implantConn.Write(tcpPayload.Data)
				}
			}
		}
	}()

	go func() {
		var streamID string
		select {
		case <-ctx.Done():
			return
		case id := <-streamIDCh:
			streamID = id
		}

		buf := make([]byte, 4096)
		var outSeq uint64 = 0
		for {
			n, err := implantConn.Read(buf)
			if err != nil {
				if err != io.EOF {
					t.Logf("implant read error: %v", err)
				}
				return
			}
			outSeq++
			outData := make([]byte, n)
			copy(outData, buf[:n])
			outMote := &portalpb.Mote{
				StreamId: streamID,
				SeqId:    outSeq,
				Payload: &portalpb.Mote_Tcp{
					Tcp: &portalpb.TCPPayload{
						Data: outData,
					},
				},
			}
			m.Publish(ctx, outTopic, outMote)
		}
	}()

	wsURL := "ws" + strings.TrimPrefix(server.URL, "http") + fmt.Sprintf("?session_id=%d&seq_id=%d&target=%s", sessionID, seqID, target)

	ws, resp, err := websocket.DefaultDialer.Dial(wsURL, nil)
	require.NoError(t, err)
	require.Equal(t, http.StatusSwitchingProtocols, resp.StatusCode)
	defer ws.Close()

	// The ssh client connects and will immediately start sending data (e.g. prompt from sshecho)
	// We might read prompt first.
	// We'll write "whoami\n" and expect "root"
	time.Sleep(1 * time.Second)
	err = ws.WriteMessage(websocket.TextMessage, []byte("whoami\r"))
	require.NoError(t, err)

	// Read until we get "root"
	gotRoot := false
	for i := 0; i < 30; i++ {
		ws.SetReadDeadline(time.Now().Add(2 * time.Second))
		_, msg, err := ws.ReadMessage()
		if err != nil {
			t.Logf("ws read error: %v", err)
			continue
		}
		out := string(msg)
		t.Logf("Received: %q", out)
		if strings.Contains(out, "root") {
			gotRoot = true
			break
		}
	}
	require.True(t, gotRoot, "Expected to receive 'root' from whoami command")

	// Send "ls\r"
	err = ws.WriteMessage(websocket.TextMessage, []byte("ls\r"))
	require.NoError(t, err)

	gotLs := false
	for i := 0; i < 30; i++ {
		ws.SetReadDeadline(time.Now().Add(2 * time.Second))
		_, msg, err := ws.ReadMessage()
		if err != nil {
			continue
		}
		out := string(msg)
		t.Logf("Received: %q", out)
		if strings.Contains(out, "bin dev proc lib home etc") {
			gotLs = true
			break
		}
	}
	require.True(t, gotLs, "Expected to receive ls output from ls command")
}
