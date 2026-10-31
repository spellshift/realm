package ssh

import (
	"context"
	"fmt"
	"io"
	"log/slog"
	"net"
	"strconv"
	"sync"
	"time"

	gossh "golang.org/x/crypto/ssh"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/portals/pnet"
	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/stream"
)

// AgentSenderFunc sends a mote directly over the gRPC stream to the connected agent.
type AgentSenderFunc = stream.SenderFunc

// PortalManager manages SSH sessions negotiated on the gRPC-receiving Tavern instance.
// It intercepts SSHPayload motes arriving from PubSub (PORTAL_IN), dials TCP motes directly
// to the agent over gRPC using pnet, performs the SSH handshake and session management,
// and publishes decrypted SSHPayload motes back over PubSub (PORTAL_OUT).
type PortalManager struct {
	portalID    int
	mux         *mux.Mux
	sendToAgent AgentSenderFunc

	mu      sync.RWMutex
	streams map[string]*portalSSHStream
	closed  bool
}

// NewPortalManager creates a new PortalManager for a gRPC portal stream.
func NewPortalManager(portalID int, portalMux *mux.Mux, sendToAgent AgentSenderFunc) *PortalManager {
	return &PortalManager{
		portalID:    portalID,
		mux:         portalMux,
		sendToAgent: sendToAgent,
		streams:     make(map[string]*portalSSHStream),
	}
}

// HandleMote processes an incoming Mote with an SSHPayload from PORTAL_IN.
func (m *PortalManager) HandleMote(ctx context.Context, mote *portalpb.Mote) {
	sshPayload := mote.GetSsh()
	if sshPayload == nil || mote.StreamId == "" {
		return
	}

	switch sshPayload.Kind {
	case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CONNECT:
		m.startStream(ctx, mote.StreamId, sshPayload)
	case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_DATA, portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_RESIZE:
		m.mu.RLock()
		s, exists := m.streams[mote.StreamId]
		m.mu.RUnlock()
		if !exists {
			return
		}
		select {
		case s.sshInCh <- sshPayload:
		case <-s.ctx.Done():
		default:
			slog.WarnContext(ctx, "dropping SSH input payload due to full buffer", "portal_id", m.portalID, "stream_id", mote.StreamId)
		}
	case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CLOSE:
		m.mu.Lock()
		s, exists := m.streams[mote.StreamId]
		if exists {
			delete(m.streams, mote.StreamId)
		}
		m.mu.Unlock()
		if exists {
			s.Close("SSH session closed by operator")
		}
	}
}

// RouteAgentMote checks if an incoming mote from the agent (over gRPC) belongs to an active
// gRPC-side SSH session. If so, it routes the mote to that session and returns true.
func (m *PortalManager) RouteAgentMote(mote *portalpb.Mote) bool {
	if mote == nil || mote.StreamId == "" {
		return false
	}

	m.mu.RLock()
	s, exists := m.streams[mote.StreamId]
	m.mu.RUnlock()
	if !exists {
		return false
	}

	select {
	case s.agentOutCh <- mote:
	case <-s.ctx.Done():
	default:
		slog.Warn("dropping agent TCP mote for SSH stream due to full buffer", "portal_id", m.portalID, "stream_id", mote.StreamId)
	}
	return true
}

// Close shuts down all active SSH streams for this portal.
func (m *PortalManager) Close() {
	m.mu.Lock()
	if m.closed {
		m.mu.Unlock()
		return
	}
	m.closed = true
	streams := make([]*portalSSHStream, 0, len(m.streams))
	for id, s := range m.streams {
		streams = append(streams, s)
		delete(m.streams, id)
	}
	m.mu.Unlock()

	for _, s := range streams {
		s.Close("Portal connection closed")
	}
}

type portalSSHStream struct {
	streamID   string
	manager    *PortalManager
	ctx        context.Context
	cancel     context.CancelFunc
	agentOutCh chan *portalpb.Mote
	sshInCh    chan *portalpb.SSHPayload
	outWriter  *stream.OrderedWriter
	closeOnce  sync.Once
}

func (m *PortalManager) startStream(ctx context.Context, streamID string, connectPayload *portalpb.SSHPayload) {
	m.mu.Lock()
	if m.closed {
		m.mu.Unlock()
		return
	}
	if _, exists := m.streams[streamID]; exists {
		m.mu.Unlock()
		return
	}

	streamCtx, cancel := context.WithCancel(ctx)
	topicOut := m.mux.TopicOut(m.portalID)
	outWriter := stream.NewOrderedWriter(streamID, func(outMote *portalpb.Mote) error {
		pubCtx, pubCancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer pubCancel()
		return m.mux.Publish(pubCtx, topicOut, outMote)
	})

	s := &portalSSHStream{
		streamID:   streamID,
		manager:    m,
		ctx:        streamCtx,
		cancel:     cancel,
		agentOutCh: make(chan *portalpb.Mote, 1024),
		sshInCh:    make(chan *portalpb.SSHPayload, 1024),
		outWriter:  outWriter,
	}
	m.streams[streamID] = s
	m.mu.Unlock()

	go s.run(connectPayload)
}

func (s *portalSSHStream) sendConnected() {
	if err := s.outWriter.WriteSSH(&portalpb.SSHPayload{
		Kind: portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_DATA,
	}); err != nil {
		slog.Debug("failed to publish SSH connected mote", "stream_id", s.streamID, "err", err)
	}
}

func (s *portalSSHStream) sendData(data []byte) {
	if len(data) == 0 {
		return
	}
	cp := make([]byte, len(data))
	copy(cp, data)
	if err := s.outWriter.WriteSSH(&portalpb.SSHPayload{
		Kind: portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_DATA,
		Data: cp,
	}); err != nil {
		slog.Debug("failed to publish SSH data mote", "stream_id", s.streamID, "err", err)
	}
}

func (s *portalSSHStream) Close(reason string) {
	s.closeOnce.Do(func() {
		s.cancel()
		_ = s.outWriter.WriteSSH(&portalpb.SSHPayload{
			Kind: portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CLOSE,
			Data: []byte(reason),
		})
		s.manager.mu.Lock()
		if current, ok := s.manager.streams[s.streamID]; ok && current == s {
			delete(s.manager.streams, s.streamID)
		}
		s.manager.mu.Unlock()
	})
}

func (s *portalSSHStream) run(connectPayload *portalpb.SSHPayload) {
	defer s.Close("SSH session ended")

	host := connectPayload.DstAddr
	port := int(connectPayload.DstPort)
	user := connectPayload.Username
	if host == "" || port <= 0 || user == "" {
		s.Close("Invalid SSH connection parameters")
		return
	}

	var authMethods []gossh.AuthMethod
	if len(connectPayload.PrivateKey) > 0 {
		signer, err := gossh.ParsePrivateKey(connectPayload.PrivateKey)
		if err != nil {
			s.Close(fmt.Sprintf("Invalid SSH private key: %v", err))
			return
		}
		authMethods = append(authMethods, gossh.PublicKeys(signer))
	}
	if connectPayload.Password != "" {
		authMethods = append(authMethods, gossh.Password(connectPayload.Password))
	}

	sshConfig := &gossh.ClientConfig{
		User:            user,
		Auth:            authMethods,
		HostKeyCallback: gossh.InsecureIgnoreHostKey(),
		Timeout:         30 * time.Second,
	}

	hostPortStr := net.JoinHostPort(host, strconv.Itoa(port))
	receiver := func() (*portalpb.Mote, error) {
		select {
		case <-s.ctx.Done():
			return nil, io.EOF
		case m, ok := <-s.agentOutCh:
			if !ok {
				return nil, io.EOF
			}
			return m, nil
		}
	}

	conn, err := pnet.Dial("tcp", hostPortStr, s.streamID, s.manager.sendToAgent, receiver)
	if err != nil {
		s.Close(fmt.Sprintf("Failed to initialize portal TCP stream: %v", err))
		return
	}
	defer conn.Close()

	// Enforce a strict deadline on the SSH handshake and session setup
	_ = conn.SetDeadline(time.Now().Add(15 * time.Second))

	c, chans, reqs, err := gossh.NewClientConn(conn, hostPortStr, sshConfig)
	if err != nil {
		s.Close(fmt.Sprintf("SSH handshake failed (%s@%s): %v", user, hostPortStr, err))
		return
	}
	sshClient := gossh.NewClient(c, chans, reqs)
	defer sshClient.Close()

	sshSession, err := sshClient.NewSession()
	if err != nil {
		s.Close(fmt.Sprintf("Failed to open SSH session: %v", err))
		return
	}
	defer sshSession.Close()

	rows := int(connectPayload.Rows)
	if rows <= 0 {
		rows = 40
	}
	cols := int(connectPayload.Cols)
	if cols <= 0 {
		cols = 80
	}

	modes := gossh.TerminalModes{
		gossh.ECHO:          1,
		gossh.TTY_OP_ISPEED: 14400,
		gossh.TTY_OP_OSPEED: 14400,
	}
	if err := sshSession.RequestPty("xterm-256color", rows, cols, modes); err != nil {
		s.Close(fmt.Sprintf("Failed to request PTY: %v", err))
		return
	}

	sshStdin, err := sshSession.StdinPipe()
	if err != nil {
		s.Close(fmt.Sprintf("Failed to bind stdin pipe: %v", err))
		return
	}
	sshStdout, err := sshSession.StdoutPipe()
	if err != nil {
		s.Close(fmt.Sprintf("Failed to bind stdout pipe: %v", err))
		return
	}
	sshStderr, err := sshSession.StderrPipe()
	if err != nil {
		s.Close(fmt.Sprintf("Failed to bind stderr pipe: %v", err))
		return
	}

	if err := sshSession.Shell(); err != nil {
		s.Close(fmt.Sprintf("Failed to start remote shell: %v", err))
		return
	}

	// Handshake and shell startup succeeded; clear the handshake deadline and notify connected
	_ = conn.SetDeadline(time.Time{})
	s.sendConnected()

	// Periodic SSH keepalive to detect stalled or dropped target connections
	go func() {
		ticker := time.NewTicker(15 * time.Second)
		defer ticker.Stop()
		for {
			select {
			case <-s.ctx.Done():
				return
			case <-ticker.C:
				keepaliveDone := make(chan error, 1)
				go func() {
					_, _, err := sshClient.SendRequest("keepalive@openssh.com", true, nil)
					keepaliveDone <- err
				}()
				select {
				case <-s.ctx.Done():
					return
				case err := <-keepaliveDone:
					if err != nil {
						s.Close(fmt.Sprintf("SSH connection lost: %v", err))
						return
					}
				case <-time.After(10 * time.Second):
					s.Close("SSH connection timed out (keepalive unresponsive)")
					return
				}
			}
		}
	}()

	var wg sync.WaitGroup
	wg.Add(2)

	// Pump stdout
	go func() {
		defer wg.Done()
		buf := make([]byte, 8192)
		for {
			n, readErr := sshStdout.Read(buf)
			if n > 0 {
				s.sendData(buf[:n])
			}
			if readErr != nil {
				return
			}
		}
	}()

	// Pump stderr
	go func() {
		defer wg.Done()
		buf := make([]byte, 8192)
		for {
			n, readErr := sshStderr.Read(buf)
			if n > 0 {
				s.sendData(buf[:n])
			}
			if readErr != nil {
				return
			}
		}
	}()

	// Close session when stdout & stderr both finish or session waits
	sessionDone := make(chan struct{})
	go func() {
		_ = sshSession.Wait()
		wg.Wait()
		close(sessionDone)
	}()

	for {
		select {
		case <-s.ctx.Done():
			return
		case <-sessionDone:
			return
		case payload, ok := <-s.sshInCh:
			if !ok {
				return
			}
			switch payload.Kind {
			case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_DATA:
				if len(payload.Data) > 0 {
					if _, writeErr := sshStdin.Write(payload.Data); writeErr != nil {
						return
					}
				}
			case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_RESIZE:
				if payload.Rows > 0 && payload.Cols > 0 {
					_ = sshSession.WindowChange(int(payload.Rows), int(payload.Cols))
				}
			case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CLOSE:
				return
			}
		}
	}
}
