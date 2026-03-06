package ssh

import (
	"context"
	"fmt"
	"io"
	"log/slog"
	"net"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/google/uuid"
	"github.com/gorilla/websocket"
	"golang.org/x/crypto/ssh"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/stream"
)

// Handle initiates an SSH connection to the specified target.
func Handle(ctx context.Context, ws *websocket.Conn, portalMux *mux.Mux, portalID int, target string) error {
	// Parse target: user:password@host:port
	// We split by the last '@' to handle passwords that might contain '@'.
	// E.g. root:toor@test@127.0.0.1:22 -> user: "root", pass: "toor@test", host: "127.0.0.1", port: "22"
	lastAt := strings.LastIndex(target, "@")
	if lastAt == -1 {
		return fmt.Errorf("invalid target format, expected user:password@host:port")
	}

	authPart := target[:lastAt]
	hostPart := target[lastAt+1:]

	firstColon := strings.Index(authPart, ":")
	if firstColon == -1 {
		return fmt.Errorf("invalid authentication format, expected user:password")
	}
	user := authPart[:firstColon]
	password := authPart[firstColon+1:]

	lastColon := strings.LastIndex(hostPart, ":")
	if lastColon == -1 {
		return fmt.Errorf("invalid host format, expected host:port")
	}
	dstAddr := hostPart[:lastColon]
	dstPort := hostPart[lastColon+1:]

	// Open portal to agent
	cleanup, err := portalMux.OpenPortal(ctx, portalID)
	if err != nil {
		return fmt.Errorf("failed to open portal: %w", err)
	}
	defer cleanup()

	// Setup portal subscription
	portalOutTopic := portalMux.TopicOut(portalID)
	recvCh, subCleanup := portalMux.Subscribe(portalOutTopic)
	defer subCleanup()

	// Create custom net.Conn
	conn := &portalTCPConn{
		ctx:       ctx,
		portalMux: portalMux,
		portalID:  portalID,
		recvCh:    recvCh,
		dstAddr:   dstAddr,
		dstPort:   dstPort,
		reader: stream.NewOrderedReader(func() (*portalpb.Mote, error) {
			mote, ok := <-recvCh
			if !ok {
				return nil, io.EOF
			}
			return mote, nil
		}),
		writer: stream.NewOrderedWriter(uuid.New().String(), func(m *portalpb.Mote) error {
			return portalMux.Publish(ctx, portalMux.TopicIn(portalID), m)
		}),
		readBuf: make([]byte, 0),
	}

	// Connect SSH Client
	clientConfig := &ssh.ClientConfig{
		User: user,
		Auth: []ssh.AuthMethod{
			ssh.Password(password),
		},
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
		Timeout:         10 * time.Second,
	}

	c, chans, reqs, err := ssh.NewClientConn(conn, dstAddr+":"+dstPort, clientConfig)
	if err != nil {
		return fmt.Errorf("failed to connect via SSH: %w", err)
	}
	sshClient := ssh.NewClient(c, chans, reqs)
	defer sshClient.Close()

	// Start a session
	session, err := sshClient.NewSession()
	if err != nil {
		return fmt.Errorf("failed to create SSH session: %w", err)
	}
	defer session.Close()

	// Request pseudo terminal
	modes := ssh.TerminalModes{
		ssh.ECHO:          1,
		ssh.TTY_OP_ISPEED: 14400,
		ssh.TTY_OP_OSPEED: 14400,
	}
	if err := session.RequestPty("xterm", 80, 40, modes); err != nil {
		return fmt.Errorf("failed to request pty: %w", err)
	}

	// Connect I/O
	stdin, err := session.StdinPipe()
	if err != nil {
		return fmt.Errorf("failed to get stdin pipe: %w", err)
	}
	stdout, err := session.StdoutPipe()
	if err != nil {
		return fmt.Errorf("failed to get stdout pipe: %w", err)
	}
	stderr, err := session.StderrPipe()
	if err != nil {
		return fmt.Errorf("failed to get stderr pipe: %w", err)
	}

	if err := session.Shell(); err != nil {
		return fmt.Errorf("failed to start shell: %w", err)
	}

	var wsMu sync.Mutex
	doneCh := make(chan struct{})

	go func() {
		defer close(doneCh)
		for {
			_, msg, err := ws.ReadMessage()
			if err != nil {
				return
			}
			stdin.Write(msg)
		}
	}()

	go func() {
		buf := make([]byte, 1024)
		for {
			n, err := stdout.Read(buf)
			if n > 0 {
				wsMu.Lock()
				ws.WriteMessage(websocket.BinaryMessage, buf[:n])
				wsMu.Unlock()
			}
			if err != nil {
				return
			}
		}
	}()

	go func() {
		buf := make([]byte, 1024)
		for {
			n, err := stderr.Read(buf)
			if n > 0 {
				wsMu.Lock()
				ws.WriteMessage(websocket.BinaryMessage, buf[:n])
				wsMu.Unlock()
			}
			if err != nil {
				return
			}
		}
	}()

	// Wait for session to finish or connection to close
	errCh := make(chan error, 1)
	go func() {
		errCh <- session.Wait()
	}()

	select {
	case <-ctx.Done():
		return ctx.Err()
	case <-doneCh:
		// Websocket read loop exited, likely client disconnected.
		slog.InfoContext(ctx, "websocket disconnected, tearing down ssh session")
		return nil
	case err := <-errCh:
		if err != nil && err != io.EOF {
			slog.WarnContext(ctx, "ssh session ended with error", "error", err)
		}
		return err
	}
}

// portalTCPConn implements net.Conn using portal TCP Motes.
type portalTCPConn struct {
	ctx       context.Context
	portalMux *mux.Mux
	portalID  int
	recvCh    <-chan *portalpb.Mote
	dstAddr   string
	dstPort   string
	reader    *stream.OrderedReader
	writer    *stream.OrderedWriter

	mu      sync.Mutex
	readBuf []byte
}

func (c *portalTCPConn) Read(b []byte) (n int, err error) {
	c.mu.Lock()
	if len(c.readBuf) > 0 {
		n = copy(b, c.readBuf)
		c.readBuf = c.readBuf[n:]
		c.mu.Unlock()
		return n, nil
	}
	c.mu.Unlock()

	for {
		select {
		case <-c.ctx.Done():
			return 0, io.EOF
		default:
		}

		mote, err := c.reader.Read()
		if err != nil {
			return 0, err
		}

		if mote == nil {
			return 0, io.EOF
		}

		// Close signal
		if bp := mote.GetBytes(); bp != nil && bp.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
			return 0, io.EOF
		}

		tcpPayload := mote.GetTcp()
		if tcpPayload == nil {
			continue // Skip non-TCP
		}

		c.mu.Lock()
		c.readBuf = append(c.readBuf, tcpPayload.Data...)
		n = copy(b, c.readBuf)
		c.readBuf = c.readBuf[n:]
		c.mu.Unlock()

		if n > 0 {
			return n, nil
		}
		// If n == 0 because len(b) == 0, wait? net.Conn usually returns 0, nil for 0 byte slice.
		return n, nil
	}
}

func (c *portalTCPConn) Write(b []byte) (n int, err error) {
	port, _ := strconv.Atoi(c.dstPort)
	if err := c.writer.WriteTCP(b, c.dstAddr, uint32(port)); err != nil {
		return 0, err
	}
	return len(b), nil
}

func (c *portalTCPConn) Close() error {
	// Send close mote
	return c.writer.WriteBytes([]byte("close"), portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE)
}

func (c *portalTCPConn) LocalAddr() net.Addr {
	return &net.TCPAddr{IP: net.ParseIP("127.0.0.1"), Port: 0}
}

func (c *portalTCPConn) RemoteAddr() net.Addr {
	port, _ := strconv.Atoi(c.dstPort)
	return &net.TCPAddr{IP: net.ParseIP(c.dstAddr), Port: port}
}

func (c *portalTCPConn) SetDeadline(t time.Time) error {
	return nil
}

func (c *portalTCPConn) SetReadDeadline(t time.Time) error {
	return nil
}

func (c *portalTCPConn) SetWriteDeadline(t time.Time) error {
	return nil
}
