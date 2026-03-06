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

type SSHHandler struct {
	client *ent.Client
	mux    *mux.Mux
}

func NewSSHHandler(client *ent.Client, mux *mux.Mux) *SSHHandler {
	return &SSHHandler{
		client: client,
		mux:    mux,
	}
}

func (h *SSHHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	// Parse Query Parameters
	portalIDStr := r.URL.Query().Get("portal_id")
	portalID, err := strconv.Atoi(portalIDStr)
	if err != nil || portalID <= 0 {
		http.Error(w, "invalid portal_id", http.StatusBadRequest)
		return
	}

	sessionID := r.URL.Query().Get("session_id")
	if sessionID == "" {
		http.Error(w, "missing session_id", http.StatusBadRequest)
		return
	}

	seqIDStr := r.URL.Query().Get("seq_id")
	var seqID uint64 = 0
	if seqIDStr != "" {
		parsed, err := strconv.ParseUint(seqIDStr, 10, 64)
		if err == nil {
			seqID = parsed
		}
	}

	target := r.URL.Query().Get("target") // format: user:password@host:port
	if target == "" {
		http.Error(w, "missing target", http.StatusBadRequest)
		return
	}

	// Parse target user:password@host:port
	lastAtIdx := strings.LastIndex(target, "@")
	if lastAtIdx == -1 {
		http.Error(w, "invalid target format, expected user:password@host:port", http.StatusBadRequest)
		return
	}

	credentials := target[:lastAtIdx]
	hostPort := target[lastAtIdx+1:]

	firstColonIdx := strings.Index(credentials, ":")
	if firstColonIdx == -1 {
		http.Error(w, "invalid target format, expected user:password@host:port", http.StatusBadRequest)
		return
	}

	user := credentials[:firstColonIdx]
	password := credentials[firstColonIdx+1:]

	// Check if portal exists and is open
	p, err := h.client.Portal.Get(ctx, portalID)
	if err != nil {
		http.Error(w, "error loading portal", http.StatusBadRequest)
		return
	}
	if !p.ClosedAt.IsZero() {
		http.Error(w, "portal is closed", http.StatusBadRequest)
		return
	}

	// Connect to Mux
	cleanupMux, err := h.mux.OpenPortal(ctx, portalID)
	if err != nil {
		slog.ErrorContext(ctx, "failed to open portal", "portal_id", portalID, "error", err)
		http.Error(w, "failed to open portal", http.StatusInternalServerError)
		return
	}
	defer cleanupMux()

	// Upgrade Websocket
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		slog.ErrorContext(ctx, "failed to upgrade websocket", "error", err)
		return
	}
	defer conn.Close()

	// Create Mux Adapter (net.Conn)
	muxConn := newPortalMuxConn(ctx, h.mux, portalID, sessionID, hostPort, seqID)
	defer muxConn.Close()

	// SSH Client Config
	sshConfig := &ssh.ClientConfig{
		User: user,
		Auth: []ssh.AuthMethod{
			ssh.Password(password),
		},
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
		Timeout:         15 * time.Second,
	}

	// Connect SSH Client over Portal Mux
	sshClientConn, chans, reqs, err := ssh.NewClientConn(muxConn, hostPort, sshConfig)
	if err != nil {
		slog.ErrorContext(ctx, "failed to connect ssh", "error", err)
		conn.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf("ssh error: %v\r\n", err)))
		return
	}
	sshClient := ssh.NewClient(sshClientConn, chans, reqs)
	defer sshClient.Close()

	// Create SSH Session
	sshSession, err := sshClient.NewSession()
	if err != nil {
		slog.ErrorContext(ctx, "failed to create ssh session", "error", err)
		conn.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf("ssh session error: %v\r\n", err)))
		return
	}
	defer sshSession.Close()

	// Request PTY
	modes := ssh.TerminalModes{
		ssh.ECHO:          1,
		ssh.TTY_OP_ISPEED: 14400,
		ssh.TTY_OP_OSPEED: 14400,
	}
	if err := sshSession.RequestPty("xterm", 80, 40, modes); err != nil {
		slog.ErrorContext(ctx, "failed to request pty", "error", err)
		conn.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf("pty error: %v\r\n", err)))
		return
	}

	// Setup Pipes
	stdinPipe, err := sshSession.StdinPipe()
	if err != nil {
		return
	}
	stdoutPipe, err := sshSession.StdoutPipe()
	if err != nil {
		return
	}
	stderrPipe, err := sshSession.StderrPipe()
	if err != nil {
		return
	}

	// Start Shell
	if err := sshSession.Shell(); err != nil {
		slog.ErrorContext(ctx, "failed to start shell", "error", err)
		conn.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf("shell error: %v\r\n", err)))
		return
	}

	// Use a channel to signal when websocket is closed
	done := make(chan struct{})
	var closeDone sync.Once
	var writeMu sync.Mutex

	// Forward Websocket -> SSH Stdin
	go func() {
		defer closeDone.Do(func() { close(done) })
		for {
			_, msg, err := conn.ReadMessage()
			if err != nil {
				break
			}
			stdinPipe.Write(msg)
		}
	}()

	// Forward SSH Stdout -> Websocket
	go func() {
		buf := make([]byte, 1024)
		for {
			n, err := stdoutPipe.Read(buf)
			if n > 0 {
				writeMu.Lock()
				conn.WriteMessage(websocket.TextMessage, buf[:n])
				writeMu.Unlock()
			}
			if err != nil {
				break
			}
		}
	}()

	// Forward SSH Stderr -> Websocket
	go func() {
		buf := make([]byte, 1024)
		for {
			n, err := stderrPipe.Read(buf)
			if n > 0 {
				writeMu.Lock()
				conn.WriteMessage(websocket.TextMessage, buf[:n])
				writeMu.Unlock()
			}
			if err != nil {
				break
			}
		}
	}()

	// Wait for session to exit or websocket to disconnect
	go func() {
		sshSession.Wait()
		closeDone.Do(func() { close(done) })
	}()

	<-done
}

// portalMuxConn wraps a portal mux connection to implement net.Conn
type portalMuxConn struct {
	ctx            context.Context
	cancel         context.CancelFunc
	mux            *mux.Mux
	portalID       int
	sessionID      string
	hostPort       string
	seqID          uint64
	seqMu          sync.Mutex
	expectedSeqOut uint64
	readBuffer     []byte
	recv           <-chan *portalpb.Mote
	cleanupSub     func()
}

func newPortalMuxConn(ctx context.Context, portalMux *mux.Mux, portalID int, sessionID string, hostPort string, seqID uint64) *portalMuxConn {
	ctx, cancel := context.WithCancel(ctx)
	topicOut := portalMux.TopicOut(portalID)
	recv, cleanupSub := portalMux.Subscribe(topicOut)

	return &portalMuxConn{
		ctx:            ctx,
		cancel:         cancel,
		mux:            portalMux,
		portalID:       portalID,
		sessionID:      sessionID,
		hostPort:       hostPort,
		seqID:          seqID,
		expectedSeqOut: 0,
		readBuffer:     make([]byte, 0),
		recv:           recv,
		cleanupSub:     cleanupSub,
	}
}

func (c *portalMuxConn) Read(b []byte) (n int, err error) {
	if len(c.readBuffer) > 0 {
		n = copy(b, c.readBuffer)
		c.readBuffer = c.readBuffer[n:]
		return n, nil
	}

	for {
		select {
		case <-c.ctx.Done():
			return 0, io.EOF
		case mote, ok := <-c.recv:
			if !ok {
				return 0, io.EOF
			}

			if mote.GetStreamId() != c.sessionID {
				continue
			}
			if tcpPayload := mote.GetTcp(); tcpPayload != nil {
				data := tcpPayload.GetData()
				if len(data) > 0 {
					n = copy(b, data)
					if n < len(data) {
						c.readBuffer = append(c.readBuffer, data[n:]...)
					}
					return n, nil
				}
			} else if bytesPayload := mote.GetBytes(); bytesPayload != nil && bytesPayload.GetKind() == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
				return 0, io.EOF
			}
		}
	}
}

func (c *portalMuxConn) Write(b []byte) (n int, err error) {
	c.seqMu.Lock()
	defer c.seqMu.Unlock()
	hostPortParts := strings.Split(c.hostPort, ":")
	host := hostPortParts[0]
	port := uint32(22)
	if len(hostPortParts) > 1 {
		p, err := strconv.ParseUint(hostPortParts[1], 10, 32)
		if err == nil {
			port = uint32(p)
		}
	}

	// Copy buffer to prevent data corruption from crypto/ssh reuse
	dataCopy := make([]byte, len(b))
	copy(dataCopy, b)

	mote := &portalpb.Mote{
		StreamId: c.sessionID,
		SeqId:    c.seqID,
		Payload: &portalpb.Mote_Tcp{
			Tcp: &portalpb.TCPPayload{
				Data:    dataCopy,
				DstAddr: host,
				DstPort: port,
			},
		},
	}
	c.seqID++

	err = c.mux.Publish(c.ctx, c.mux.TopicIn(c.portalID), mote)
	if err != nil {
		return 0, err
	}
	return len(b), nil
}

func (c *portalMuxConn) Close() error {
	c.seqMu.Lock()
	defer c.seqMu.Unlock()
	// Send close mote
	closeMote := &portalpb.Mote{
		StreamId: c.sessionID,
		SeqId:    c.seqID,
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE,
				Data: []byte("close"),
			},
		},
	}
	c.mux.Publish(c.ctx, c.mux.TopicIn(c.portalID), closeMote)
	c.seqID++

	c.cancel()
	c.cleanupSub()
	return nil
}

func (c *portalMuxConn) LocalAddr() net.Addr {
	return &net.TCPAddr{IP: net.ParseIP("127.0.0.1"), Port: 0}
}

func (c *portalMuxConn) RemoteAddr() net.Addr {
	return &net.TCPAddr{IP: net.ParseIP("127.0.0.1"), Port: 22}
}

func (c *portalMuxConn) SetDeadline(t time.Time) error {
	return nil
}

func (c *portalMuxConn) SetReadDeadline(t time.Time) error {
	return nil
}

func (c *portalMuxConn) SetWriteDeadline(t time.Time) error {
	return nil
}
