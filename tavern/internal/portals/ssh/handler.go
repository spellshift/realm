package ssh

import (
	"context"
	"io"
	"log/slog"
	"net"
	"net/http"
	"strconv"
	"sync"
	"time"

	"github.com/gorilla/websocket"
	"golang.org/x/crypto/ssh"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  4096,
	WriteBufferSize: 4096,
	CheckOrigin:     func(r *http.Request) bool { return true },
}

// Handler handles websocket connections for portal ssh sessions
type Handler struct {
	graph *ent.Client
	mux   *mux.Mux
}

// NewHandler creates a new SSH Handler
func NewHandler(graph *ent.Client, m *mux.Mux) *Handler {
	return &Handler{
		graph: graph,
		mux:   m,
	}
}

// ServeHTTP handles the websocket connection, establishes an SSH connection over the portal,
// and acts as a bridge between the browser websocket and the SSH session.
func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	// Parse Portal ID
	portalIDStr := r.URL.Query().Get("portal_id")
	if portalIDStr == "" {
		http.Error(w, "missing portal_id", http.StatusBadRequest)
		return
	}
	portalID, err := strconv.Atoi(portalIDStr)
	if err != nil {
		http.Error(w, "invalid portal_id", http.StatusBadRequest)
		return
	}

	// Parse target (user:password@host:port or user:password@host)
	target := r.URL.Query().Get("target")
	if target == "" {
		http.Error(w, "missing target", http.StatusBadRequest)
		return
	}

	// Simple parse of target
	// format: user:password@host:port (port optional, default 22)
	// we will need to parse this appropriately for x/crypto/ssh
	userPart := target
	hostPart := ""
	for i := 0; i < len(target); i++ {
		if target[i] == '@' {
			userPart = target[:i]
			hostPart = target[i+1:]
			break
		}
	}

	if hostPart == "" {
		http.Error(w, "invalid target format, expected user:password@host", http.StatusBadRequest)
		return
	}

	user := userPart
	password := ""
	for i := 0; i < len(userPart); i++ {
		if userPart[i] == ':' {
			user = userPart[:i]
			password = userPart[i+1:]
			break
		}
	}

	host := hostPart
	port := "22"
	for i := 0; i < len(hostPart); i++ {
		if hostPart[i] == ':' {
			host = hostPart[:i]
			port = hostPart[i+1:]
			break
		}
	}
	portNum, err := strconv.Atoi(port)
	if err != nil {
		http.Error(w, "invalid port in target", http.StatusBadRequest)
		return
	}

	// Upgrade to websocket
	wsConn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		slog.ErrorContext(ctx, "failed to upgrade ssh portal to websocket", "err", err)
		return
	}
	defer wsConn.Close()

	// Connect to Portal
	portalCleanup, err := h.mux.OpenPortal(ctx, portalID)
	if err != nil {
		wsConn.WriteMessage(websocket.TextMessage, []byte("\x1b[31mFailed to connect to portal.\x1b[0m\r\n"))
		return
	}
	defer portalCleanup()

	// Wait for Mux to establish and return conn
	// Since we need to bridge a generic net.Conn with x/crypto/ssh, we will use an in-memory pipe
	// and send/recv Motes.

	p1, p2 := newBufferedPipe()

	// The goroutine below will read from p2 and send TCP motes over the portal
	streamID := "ssh-" + strconv.FormatInt(time.Now().UnixNano(), 10)

	// Subscribe to output motes from the portal
	outCh, subCleanup := h.mux.Subscribe(h.mux.TopicOut(portalID))
	defer subCleanup()

	ctx, cancel := context.WithCancel(ctx)
	defer cancel()

	// Goroutine: Read from Portal (outCh), write to p2
	go func() {
		for {
			select {
			case <-ctx.Done():
				p2.Close()
				return
			case m, ok := <-outCh:
				if !ok {
					p2.Close()
					return
				}
				if m.StreamId != streamID {
					continue
				}
				switch p := m.Payload.(type) {
				case *portalpb.Mote_Tcp:
					p2.Write(p.Tcp.Data)
				case *portalpb.Mote_Bytes:
					if p.Bytes.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
						p2.Close()
						return
					}
				}
			}
		}
	}()

	// Goroutine: Read from p2, write to Portal (TopicIn)
	go func() {
		buf := make([]byte, 32*1024)
		seq := uint64(0)
		for {
			n, err := p2.Read(buf)
			if n > 0 {
				seq++

				// Make a copy of the buffer slice to avoid data races
				dataCopy := make([]byte, n)
				copy(dataCopy, buf[:n])

				mote := &portalpb.Mote{
					StreamId: streamID,
					SeqId:    seq,
					Payload: &portalpb.Mote_Tcp{
						Tcp: &portalpb.TCPPayload{
							Data:    dataCopy,
							DstAddr: host,
							DstPort: uint32(portNum),
						},
					},
				}
				h.mux.Publish(ctx, h.mux.TopicIn(portalID), mote)
			}
			if err != nil {
				seq++
				closeMote := &portalpb.Mote{
					StreamId: streamID,
					SeqId:    seq,
					Payload: &portalpb.Mote_Bytes{
						Bytes: &portalpb.BytesPayload{
							Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE,
						},
					},
				}
				h.mux.Publish(ctx, h.mux.TopicIn(portalID), closeMote)
				p2.Close()
				return
			}
		}
	}()

	// Perform SSH Handshake over p1
	config := &ssh.ClientConfig{
		User: user,
		Auth: []ssh.AuthMethod{
			ssh.Password(password),
		},
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
		Timeout:         10 * time.Second,
	}

	wsConn.WriteMessage(websocket.TextMessage, []byte("Establishing SSH connection...\r\n"))

	// Create an ssh client connection using the pipe
	sshConn, chans, reqs, err := ssh.NewClientConn(p1, target, config)
	if err != nil {
		slog.ErrorContext(ctx, "ssh handshake failed", "err", err)
		wsConn.WriteMessage(websocket.TextMessage, []byte("\x1b[31mSSH Handshake failed: "+err.Error()+"\x1b[0m\r\n"))
		return
	}
	sshClient := ssh.NewClient(sshConn, chans, reqs)
	defer sshClient.Close()

	// Open a session
	session, err := sshClient.NewSession()
	if err != nil {
		slog.ErrorContext(ctx, "failed to create ssh session", "err", err)
		wsConn.WriteMessage(websocket.TextMessage, []byte("\x1b[31mFailed to create SSH session: "+err.Error()+"\x1b[0m\r\n"))
		return
	}
	defer session.Close()

	// Setup pseudo terminal
	modes := ssh.TerminalModes{
		ssh.ECHO:          1,
		ssh.TTY_OP_ISPEED: 14400,
		ssh.TTY_OP_OSPEED: 14400,
	}

	if err := session.RequestPty("xterm", 80, 40, modes); err != nil {
		slog.ErrorContext(ctx, "request for pseudo terminal failed", "err", err)
		wsConn.WriteMessage(websocket.TextMessage, []byte("\x1b[31mFailed to request pty: "+err.Error()+"\x1b[0m\r\n"))
		return
	}

	// Setup stdin/stdout
	stdin, err := session.StdinPipe()
	if err != nil {
		return
	}
	stdout, err := session.StdoutPipe()
	if err != nil {
		return
	}
	stderr, err := session.StderrPipe()
	if err != nil {
		return
	}

	// Start remote shell
	if err := session.Shell(); err != nil {
		slog.ErrorContext(ctx, "failed to start shell", "err", err)
		wsConn.WriteMessage(websocket.TextMessage, []byte("\x1b[31mFailed to start shell: "+err.Error()+"\x1b[0m\r\n"))
		return
	}

	wsConn.WriteMessage(websocket.TextMessage, []byte("\x1b[2J\x1b[H")) // clear screen

	var wg sync.WaitGroup
	wg.Add(3)

	// Read from WS and write to SSH stdin
	go func() {
		defer wg.Done()
		for {
			_, message, err := wsConn.ReadMessage()
			if err != nil {
				// If the websocket disconnects, we should kill the SSH session to unblock
				// the stdout/stderr goroutines and prevent a leak.
				sshClient.Close()
				return
			}
			stdin.Write(message)
		}
	}()

	// Read from SSH stdout and write to WS
	go func() {
		defer wg.Done()
		buf := make([]byte, 4096)
		for {
			n, err := stdout.Read(buf)
			if n > 0 {
				wsConn.WriteMessage(websocket.TextMessage, buf[:n])
			}
			if err != nil {
				return
			}
		}
	}()

	// Read from SSH stderr and write to WS
	go func() {
		defer wg.Done()
		buf := make([]byte, 4096)
		for {
			n, err := stderr.Read(buf)
			if n > 0 {
				wsConn.WriteMessage(websocket.TextMessage, buf[:n])
			}
			if err != nil {
				return
			}
		}
	}()

	wg.Wait()
}

// bufferedConn is a net.Conn that wraps an io.Reader and io.Writer
// This avoids the synchronous deadlock of net.Pipe when one side is doing full-duplex non-blocking I/O.
type bufferedConn struct {
	r io.Reader
	w io.Writer
}

func (c *bufferedConn) Read(b []byte) (n int, err error) {
	return c.r.Read(b)
}

func (c *bufferedConn) Write(b []byte) (n int, err error) {
	return c.w.Write(b)
}

func (c *bufferedConn) Close() error {
	var err1, err2 error
	if closer, ok := c.r.(io.Closer); ok {
		err1 = closer.Close()
	}
	if closer, ok := c.w.(io.Closer); ok {
		err2 = closer.Close()
	}
	if err1 != nil {
		return err1
	}
	return err2
}

func (c *bufferedConn) LocalAddr() net.Addr {
	return nil
}

func (c *bufferedConn) RemoteAddr() net.Addr {
	return nil
}

func (c *bufferedConn) SetDeadline(t time.Time) error {
	return nil
}

func (c *bufferedConn) SetReadDeadline(t time.Time) error {
	return nil
}

func (c *bufferedConn) SetWriteDeadline(t time.Time) error {
	return nil
}

func newBufferedPipe() (net.Conn, net.Conn) {
	r1, w1 := io.Pipe()
	r2, w2 := io.Pipe()
	return &bufferedConn{r: r1, w: w2}, &bufferedConn{r: r2, w: w1}
}
