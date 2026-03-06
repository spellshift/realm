package ws

import (
	"context"
	"fmt"
	"io"
	"log/slog"
	"net"
	"net/http"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/gorilla/websocket"
	"golang.org/x/crypto/ssh"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin:     func(r *http.Request) bool { return true },
}

// SSHHandler handles websocket connections and proxies them to an SSH session
// over a TCP portal mote.
type SSHHandler struct {
	graph *ent.Client
	mux   *mux.Mux
}

func NewSSHHandler(graph *ent.Client, mux *mux.Mux) *SSHHandler {
	return &SSHHandler{
		graph: graph,
		mux:   mux,
	}
}

func (h *SSHHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	// Parse parameters
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

	target := r.URL.Query().Get("target")
	if target == "" {
		http.Error(w, "missing target", http.StatusBadRequest)
		return
	}

	// target format: user:password@host:port
	parts := strings.Split(target, "@")
	if len(parts) != 2 {
		http.Error(w, "invalid target format, expected user:password@host:port", http.StatusBadRequest)
		return
	}
	creds := strings.SplitN(parts[0], ":", 2)
	if len(creds) != 2 {
		http.Error(w, "invalid target credentials format, expected user:password", http.StatusBadRequest)
		return
	}
	user := creds[0]
	password := creds[1]

	hostPort := strings.Split(parts[1], ":")
	if len(hostPort) != 2 {
		http.Error(w, "invalid target host format, expected host:port", http.StatusBadRequest)
		return
	}
	host := hostPort[0]
	portStr := hostPort[1]
	port, err := strconv.Atoi(portStr)
	if err != nil {
		http.Error(w, "invalid port in target", http.StatusBadRequest)
		return
	}

	// Load Portal
	p, err := h.graph.Portal.Get(ctx, portalID)
	if err != nil {
		http.Error(w, fmt.Sprintf("failed to load portal: %v", err), http.StatusInternalServerError)
		return
	}
	if !p.ClosedAt.IsZero() {
		http.Error(w, "portal is closed", http.StatusBadRequest)
		return
	}

	// Upgrade Websocket
	wsConn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		slog.ErrorContext(ctx, "failed to upgrade websocket", "error", err)
		return
	}
	defer wsConn.Close()

	// Open Portal Mux
	cleanup, err := h.mux.OpenPortal(ctx, portalID)
	if err != nil {
		slog.ErrorContext(ctx, "failed to open portal", "portal_id", portalID, "error", err)
		wsConn.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseInternalServerErr, "failed to open portal"))
		return
	}
	defer cleanup()

	portalOutTopic := h.mux.TopicOut(portalID)
	recv, cleanupSub := h.mux.Subscribe(portalOutTopic)
	defer cleanupSub()

	// SSH Connection Setup using portal
	config := &ssh.ClientConfig{
		User: user,
		Auth: []ssh.AuthMethod{
			ssh.Password(password),
		},
		HostKeyCallback: ssh.InsecureIgnoreHostKey(), // TODO: Make secure
		Timeout:         10 * time.Second,
	}

	// Dial using portal wrapper
	portalConn := newPortalConn(ctx, portalID, host, uint32(port), h.mux, recv)
	defer portalConn.Close()

	sshConn, chans, reqs, err := ssh.NewClientConn(portalConn, fmt.Sprintf("%s:%d", host, port), config)
	if err != nil {
		slog.ErrorContext(ctx, "failed to connect to ssh", "error", err)
		wsConn.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseInternalServerErr, "failed to connect to ssh"))
		return
	}

	sshClient := ssh.NewClient(sshConn, chans, reqs)
	defer sshClient.Close()

	session, err := sshClient.NewSession()
	if err != nil {
		slog.ErrorContext(ctx, "failed to create ssh session", "error", err)
		wsConn.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseInternalServerErr, "failed to create ssh session"))
		return
	}
	defer session.Close()

	// Request pseudo terminal
	modes := ssh.TerminalModes{
		ssh.ECHO:          1,     // enable echoing
		ssh.TTY_OP_ISPEED: 14400, // input speed = 14.4kbaud
		ssh.TTY_OP_OSPEED: 14400, // output speed = 14.4kbaud
	}

	if err := session.RequestPty("xterm-256color", 40, 80, modes); err != nil {
		slog.ErrorContext(ctx, "request for pseudo terminal failed", "error", err)
		wsConn.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseInternalServerErr, "failed to request pty"))
		return
	}

	stdin, err := session.StdinPipe()
	if err != nil {
		slog.ErrorContext(ctx, "failed to setup stdin", "error", err)
		return
	}

	stdout, err := session.StdoutPipe()
	if err != nil {
		slog.ErrorContext(ctx, "failed to setup stdout", "error", err)
		return
	}

	stderr, err := session.StderrPipe()
	if err != nil {
		slog.ErrorContext(ctx, "failed to setup stderr", "error", err)
		return
	}

	if err := session.Shell(); err != nil {
		slog.ErrorContext(ctx, "failed to start shell", "error", err)
		wsConn.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseInternalServerErr, "failed to start shell"))
		return
	}

	_, cancel := context.WithCancel(ctx)
	defer cancel()

	errCh := make(chan error, 3)
	var wsMu sync.Mutex

	// Read from WS and write to SSH stdin
	go func() {
		defer cancel()
		for {
			_, msg, err := wsConn.ReadMessage()
			if err != nil {
				errCh <- err
				return
			}
			if _, err := stdin.Write(msg); err != nil {
				errCh <- err
				return
			}
		}
	}()

	// Read from SSH stdout and write to WS
	go func() {
		defer cancel()
		buf := make([]byte, 1024)
		for {
			n, err := stdout.Read(buf)
			if n > 0 {
				wsMu.Lock()
				wsConn.WriteMessage(websocket.TextMessage, buf[:n])
				wsMu.Unlock()
			}
			if err != nil {
				errCh <- err
				return
			}
		}
	}()

	// Read from SSH stderr and write to WS
	go func() {
		defer cancel()
		buf := make([]byte, 1024)
		for {
			n, err := stderr.Read(buf)
			if n > 0 {
				wsMu.Lock()
				wsConn.WriteMessage(websocket.TextMessage, buf[:n])
				wsMu.Unlock()
			}
			if err != nil {
				errCh <- err
				return
			}
		}
	}()

	<-errCh
}

type portalConn struct {
	ctx      context.Context
	cancel   context.CancelFunc
	portalID int
	host     string
	port     uint32
	mux      *mux.Mux
	recv     <-chan *portalpb.Mote

	readBuf  []byte
	mu       sync.Mutex
	cond     *sync.Cond
	closed   bool
	streamID string
	seqID    uint64
}

func newPortalConn(ctx context.Context, portalID int, host string, port uint32, m *mux.Mux, recv <-chan *portalpb.Mote) *portalConn {
	ctx, cancel := context.WithCancel(ctx)
	pc := &portalConn{
		ctx:      ctx,
		cancel:   cancel,
		portalID: portalID,
		host:     host,
		port:     port,
		mux:      m,
		recv:     recv,
		streamID: "ssh-" + strconv.Itoa(portalID),
	}
	pc.cond = sync.NewCond(&pc.mu)

	go pc.readLoop()

	return pc
}

func (pc *portalConn) readLoop() {
	for {
		select {
		case <-pc.ctx.Done():
			return
		case mote, ok := <-pc.recv:
			if !ok {
				pc.Close()
				return
			}

			if mote.StreamId != pc.streamID {
				continue
			}

			if tcp := mote.GetTcp(); tcp != nil {
				pc.mu.Lock()
				pc.readBuf = append(pc.readBuf, tcp.Data...)
				pc.cond.Broadcast()
				pc.mu.Unlock()
			} else if bytes := mote.GetBytes(); bytes != nil && bytes.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
				pc.Close()
				return
			}
		}
	}
}

func (pc *portalConn) Read(b []byte) (n int, err error) {
	pc.mu.Lock()
	defer pc.mu.Unlock()

	for len(pc.readBuf) == 0 && !pc.closed {
		pc.cond.Wait()
	}

	if pc.closed && len(pc.readBuf) == 0 {
		return 0, io.EOF
	}

	n = copy(b, pc.readBuf)
	pc.readBuf = pc.readBuf[n:]
	return n, nil
}

func (pc *portalConn) Write(b []byte) (n int, err error) {
	pc.mu.Lock()
	if pc.closed {
		pc.mu.Unlock()
		return 0, io.ErrClosedPipe
	}
	pc.seqID++
	seqID := pc.seqID
	pc.mu.Unlock()

	// Make sure we copy the buffer, otherwise protobuf might try to serialize it
	// while ssh is reusing it or doing weird things.
	// Actually wait, let's copy it here just to be safe.
	bCopy := make([]byte, len(b))
	copy(bCopy, b)

	mote := &portalpb.Mote{
		StreamId: pc.streamID,
		SeqId:    seqID,
		Payload: &portalpb.Mote_Tcp{
			Tcp: &portalpb.TCPPayload{
				Data:    bCopy,
				DstAddr: pc.host,
				DstPort: pc.port,
			},
		},
	}

	err = pc.mux.Publish(pc.ctx, pc.mux.TopicIn(pc.portalID), mote)
	if err != nil {
		return 0, err
	}
	return len(b), nil
}

func (pc *portalConn) Close() error {
	pc.mu.Lock()
	if pc.closed {
		pc.mu.Unlock()
		return nil
	}
	pc.closed = true
	pc.cond.Broadcast()
	pc.mu.Unlock()

	pc.cancel()
	return nil
}

func (pc *portalConn) LocalAddr() net.Addr {
	return &net.TCPAddr{IP: net.ParseIP("127.0.0.1"), Port: 0}
}

func (pc *portalConn) RemoteAddr() net.Addr {
	return &net.TCPAddr{IP: net.ParseIP(pc.host), Port: int(pc.port)}
}

func (pc *portalConn) SetDeadline(t time.Time) error {
	return nil
}

func (pc *portalConn) SetReadDeadline(t time.Time) error {
	return nil
}

func (pc *portalConn) SetWriteDeadline(t time.Time) error {
	return nil
}
