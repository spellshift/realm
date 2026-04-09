package ws

import (
	"sync"

	"context"
	"io"
	"log/slog"
	"net"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
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

type SSHPortalHandler struct {
	graph *ent.Client
	mux   *mux.Mux
}

func NewSSHPortalHandler(graph *ent.Client, mux *mux.Mux) *SSHPortalHandler {
	return &SSHPortalHandler{
		graph: graph,
		mux:   mux,
	}
}

func (h *SSHPortalHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	// 1. Parse params
	sessionIDStr := r.URL.Query().Get("session_id")
	if sessionIDStr == "" {
		http.Error(w, "missing session_id", http.StatusBadRequest)
		return
	}
	sessionID, err := strconv.Atoi(sessionIDStr)
	if err != nil {
		http.Error(w, "invalid session_id", http.StatusBadRequest)
		return
	}

	seqIDStr := r.URL.Query().Get("seq_id")
	seqID := uint64(0)
	if seqIDStr != "" {
		seqID, _ = strconv.ParseUint(seqIDStr, 10, 64)
	}

	target := r.URL.Query().Get("target")
	if target == "" {
		http.Error(w, "missing target", http.StatusBadRequest)
		return
	}

	// Parse target user:password@host:port
	parts := strings.Split(target, "@")
	if len(parts) != 2 {
		http.Error(w, "invalid target format", http.StatusBadRequest)
		return
	}
	userPass := strings.Split(parts[0], ":")
	if len(userPass) != 2 {
		http.Error(w, "invalid target format", http.StatusBadRequest)
		return
	}
	user := userPass[0]
	pass := userPass[1]

	hostPort := parts[1]
	hostParts := strings.Split(hostPort, ":")
	if len(hostParts) != 2 {
		http.Error(w, "invalid target format", http.StatusBadRequest)
		return
	}
	host := hostParts[0]
	port, err := strconv.ParseUint(hostParts[1], 10, 32)
	if err != nil {
		http.Error(w, "invalid port in target", http.StatusBadRequest)
		return
	}

	// 2. Upgrade websocket
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		slog.Error("failed to upgrade websocket", "error", err)
		return
	}
	defer conn.Close()

	// 3. Connect to Portal Mux
	ctx, cancel := context.WithCancel(r.Context())
	defer cancel()

	streamID := uuid.New().String()

	// 4. Setup Portal Mux Net Conn
	portalNetConn := &PortalNetConn{
		ctx:      ctx,
		mux:      h.mux,
		portalID: sessionID,
		streamID: streamID,
		seqID:    seqID,
		host:     host,
		port:     uint32(port),
		readCh:   make(chan *portalpb.Mote, 1024),
		nextSeq:  1,
		recvBufs: make(map[uint64][]byte),
	}

	// Subscribe to portal output
	cleanup, err := h.mux.OpenPortal(ctx, sessionID)
	if err != nil {
		slog.ErrorContext(ctx, "failed to open portal", "error", err)
		return
	}
	defer cleanup()

	portalOutTopic := h.mux.TopicOut(sessionID)
	recv, subCleanup := h.mux.Subscribe(portalOutTopic)
	defer subCleanup()

	// Goroutine to receive motes and put them in the portalNetConn readCh
	go func() {
		for {
			select {
			case <-ctx.Done():
				return
			case mote, ok := <-recv:
				if !ok {
					return
				}
				if tcpPayload := mote.GetTcp(); tcpPayload != nil {
					// Only process motes for our stream (or we could broadcast, but standard is stream match)
					if mote.StreamId == streamID {
						portalNetConn.readCh <- mote
					}
				}
			}
		}
	}()

	// Start SSH client
	config := &ssh.ClientConfig{
		User: user,
		Auth: []ssh.AuthMethod{
			ssh.Password(pass),
		},
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
		Timeout:         10 * time.Second,
	}

	sshConn, chans, reqs, err := ssh.NewClientConn(portalNetConn, hostPort, config)
	if err != nil {
		slog.ErrorContext(ctx, "failed to connect ssh", "error", err)
		return
	}
	defer sshConn.Close()

	client := ssh.NewClient(sshConn, chans, reqs)
	defer client.Close()

	session, err := client.NewSession()
	if err != nil {
		slog.ErrorContext(ctx, "failed to create ssh session", "error", err)
		return
	}
	defer session.Close()

	// Setup piping
	session.Stdin = &wsReader{conn: conn, ctx: ctx}
	session.Stdout = &wsWriter{conn: conn, ctx: ctx}
	session.Stderr = &wsWriter{conn: conn, ctx: ctx}

	// Request PTY
	if err := session.RequestPty("xterm", 80, 40, ssh.TerminalModes{
		ssh.ECHO:          1,     // enable echoing
		ssh.TTY_OP_ISPEED: 14400, // input speed = 14.4kbaud
		ssh.TTY_OP_OSPEED: 14400, // output speed = 14.4kbaud
	}); err != nil {
		slog.ErrorContext(ctx, "failed to request pty", "error", err)
		return
	}

	// Start shell
	if err := session.Shell(); err != nil {
		slog.ErrorContext(ctx, "failed to start shell", "error", err)
		return
	}

	session.Wait()
}

// PortalNetConn implements net.Conn using Portal Motes
type PortalNetConn struct {
	ctx      context.Context
	mux      *mux.Mux
	portalID int
	streamID string
	seqID    uint64
	host     string
	port     uint32
	readCh   chan *portalpb.Mote
	nextSeq  uint64
	recvBufs map[uint64][]byte
	readBuf  []byte
	mu       sync.Mutex
}

func (c *PortalNetConn) Read(b []byte) (n int, err error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	// Drain any contiguous buffered motes first
	c.drainRecvBufs()

	if len(c.readBuf) > 0 {
		n = copy(b, c.readBuf)
		c.readBuf = c.readBuf[n:]
		return n, nil
	}

	for {
		c.mu.Unlock()
		select {
		case <-c.ctx.Done():
			c.mu.Lock()
			return 0, io.EOF
		case mote, ok := <-c.readCh:
			c.mu.Lock()
			if !ok {
				return 0, io.EOF
			}

			// Store new mote
			if mote.SeqId >= c.nextSeq {
				c.recvBufs[mote.SeqId] = mote.GetTcp().Data
			}

			// Drain contiguous
			c.drainRecvBufs()

			if len(c.readBuf) > 0 {
				n = copy(b, c.readBuf)
				c.readBuf = c.readBuf[n:]
				return n, nil
			}
		}
	}
}

func (c *PortalNetConn) drainRecvBufs() {
	for {
		if data, ok := c.recvBufs[c.nextSeq]; ok {
			delete(c.recvBufs, c.nextSeq)
			c.nextSeq++
			c.readBuf = append(c.readBuf, data...)
		} else {
			break
		}
	}
}

func (c *PortalNetConn) Write(b []byte) (n int, err error) {
	c.mu.Lock()
	c.seqID++
	seq := c.seqID
	c.mu.Unlock()

	mote := &portalpb.Mote{
		StreamId: c.streamID,
		SeqId:    seq,
		Payload: &portalpb.Mote_Tcp{
			Tcp: &portalpb.TCPPayload{
				Data:    b,
				DstAddr: c.host,
				DstPort: c.port,
			},
		},
	}

	topicIn := c.mux.TopicIn(c.portalID)
	if err := c.mux.Publish(c.ctx, topicIn, mote); err != nil {
		return 0, err
	}
	return len(b), nil
}

func (c *PortalNetConn) Close() error { return nil }
func (c *PortalNetConn) LocalAddr() net.Addr {
	return &net.TCPAddr{IP: net.IPv4(127, 0, 0, 1), Port: 0}
}
func (c *PortalNetConn) RemoteAddr() net.Addr {
	return &net.TCPAddr{IP: net.ParseIP(c.host), Port: int(c.port)}
}
func (c *PortalNetConn) SetDeadline(t time.Time) error      { return nil }
func (c *PortalNetConn) SetReadDeadline(t time.Time) error  { return nil }
func (c *PortalNetConn) SetWriteDeadline(t time.Time) error { return nil }

type wsReader struct {
	conn *websocket.Conn
	ctx  context.Context
	buf  []byte
}

func (r *wsReader) Read(p []byte) (int, error) {
	if len(r.buf) > 0 {
		n := copy(p, r.buf)
		r.buf = r.buf[n:]
		return n, nil
	}
	for {
		select {
		case <-r.ctx.Done():
			return 0, io.EOF
		default:
			_, msg, err := r.conn.ReadMessage()
			if err != nil {
				return 0, io.EOF
			}
			if len(msg) > 0 {
				n := copy(p, msg)
				if n < len(msg) {
					r.buf = msg[n:]
				}
				return n, nil
			}
		}
	}
}

type wsWriter struct {
	conn *websocket.Conn
	ctx  context.Context
}

func (w *wsWriter) Write(p []byte) (int, error) {
	err := w.conn.WriteMessage(websocket.TextMessage, p)
	if err != nil {
		return 0, err
	}
	return len(p), nil
}
