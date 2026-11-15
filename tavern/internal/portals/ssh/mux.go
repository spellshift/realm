package ssh

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"log/slog"
	"net"
	"strconv"
	"sync"
	"time"

	"github.com/google/uuid"
	gossh "golang.org/x/crypto/ssh"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/portals/pnet"
	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/stream"
)

var log = logging.For("portals.ssh")

func ensurePortalStreamAttrs(ctx context.Context, portalID int, sshStreamID string) context.Context {
	var attrs []slog.Attr
	hasPortalID := false
	for _, a := range logging.MetaFromContext(ctx).StreamAttrs {
		if a.Key == "portal_id" {
			hasPortalID = true
			break
		}
	}
	if !hasPortalID && portalID > 0 {
		attrs = append(attrs, slog.Int("portal_id", portalID))
	}
	if sshStreamID != "" {
		attrs = append(attrs, slog.String("ssh_stream_id", sshStreamID))
	}
	return logging.WithStreamAttrs(ctx, attrs...)
}

// AgentSenderFunc sends a mote directly over the gRPC stream to the connected agent.
type AgentSenderFunc = stream.SenderFunc

// Mux manages SSH sessions negotiated on the gRPC-receiving Tavern instance.
// It intercepts SSHPayload motes arriving from PubSub (PORTAL_IN), dials TCP motes directly
// to the agent over gRPC using pnet, performs the SSH handshake and session management,
// and publishes decrypted SSHPayload motes back over PubSub (PORTAL_OUT).
type Mux struct {
	portalID    int
	mux         *mux.Mux
	sendToAgent AgentSenderFunc

	mu         sync.RWMutex
	sshStreams map[string]*portalSSHStream       // keyed by sshStreamID
	tcpStreams map[string]*portalSSHStream       // keyed by tcpStreamID
	pending    map[string][]*portalpb.SSHPayload // keyed by sshStreamID
	closed     bool
}

// New creates a new SSH Mux for a gRPC portal stream.
func New(portalID int, portalMux *mux.Mux, sendToAgent AgentSenderFunc) *Mux {
	return &Mux{
		portalID:    portalID,
		mux:         portalMux,
		sendToAgent: sendToAgent,
		sshStreams:  make(map[string]*portalSSHStream),
		tcpStreams:  make(map[string]*portalSSHStream),
		pending:     make(map[string][]*portalpb.SSHPayload),
	}
}

// NewMux creates a new SSH Mux for a gRPC portal stream.
func NewMux(portalID int, portalMux *mux.Mux, sendToAgent AgentSenderFunc) *Mux {
	return New(portalID, portalMux, sendToAgent)
}

// HandleMote processes an incoming Mote with an SSHPayload from PORTAL_IN.
func (m *Mux) HandleMote(ctx context.Context, mote *portalpb.Mote) {
	sshPayload := mote.GetSsh()
	if sshPayload == nil || mote.StreamId == "" {
		return
	}

	switch sshPayload.Kind {
	case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CONNECT:
		m.startStream(ctx, mote.StreamId, sshPayload)
	case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_DATA, portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_RESIZE:
		m.mu.Lock()
		s, exists := m.sshStreams[mote.StreamId]
		if !exists {
			m.pending[mote.StreamId] = append(m.pending[mote.StreamId], sshPayload)
			m.mu.Unlock()
			log.DebugContext(ctx, "buffering SSH payload prior to connect",
				"portal_id", m.portalID,
				"stream_id", mote.StreamId,
				"kind", sshPayload.Kind.String(),
			)
			return
		}
		m.mu.Unlock()

		if sshPayload.Kind == portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_RESIZE {
			log.DebugContext(ctx, "queuing SSH window resize payload",
				"portal_id", m.portalID,
				"stream_id", mote.StreamId,
				"rows", sshPayload.Rows,
				"cols", sshPayload.Cols,
			)
		} else {
			log.TraceContext(ctx, "queuing SSH input data payload",
				"portal_id", m.portalID,
				"stream_id", mote.StreamId,
				"bytes", len(sshPayload.Data),
			)
		}

		select {
		case s.sshInCh <- sshPayload:
		case <-s.ctx.Done():
		default:
			log.WarnContext(ctx, "dropping SSH input payload due to full buffer", "portal_id", m.portalID, "stream_id", mote.StreamId)
		}
	case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CLOSE:
		m.mu.Lock()
		delete(m.pending, mote.StreamId)
		s, exists := m.sshStreams[mote.StreamId]
		if exists {
			delete(m.sshStreams, mote.StreamId)
			if s.tcpStreamID != "" {
				delete(m.tcpStreams, s.tcpStreamID)
			}
		}
		m.mu.Unlock()

		if exists {
			log.DebugContext(ctx, "closing SSH stream from operator close mote", "portal_id", m.portalID, "stream_id", mote.StreamId)
			s.Close("SSH session closed by operator")
		}
	}
}

// RouteAgentMote checks if an incoming mote from the agent (over gRPC) belongs to an active
// gRPC-side SSH session. If so, it routes the mote to that session and returns true.
func (m *Mux) RouteAgentMote(mote *portalpb.Mote) bool {
	if mote == nil || mote.StreamId == "" {
		return false
	}

	m.mu.RLock()
	s, exists := m.tcpStreams[mote.StreamId]
	m.mu.RUnlock()
	if !exists {
		return false
	}

	log.TraceContext(s.ctx, "routing agent TCP mote to SSH stream", "tcp_stream_id", mote.StreamId, "seq_id", mote.SeqId)

	select {
	case s.agentOutCh <- mote:
	case <-s.ctx.Done():
	default:
		log.WarnContext(s.ctx, "dropping agent TCP mote for SSH stream due to full buffer", "tcp_stream_id", mote.StreamId)
	}
	return true
}

// Close shuts down all active SSH streams for this portal.
func (m *Mux) Close() {
	m.mu.Lock()
	if m.closed {
		m.mu.Unlock()
		return
	}
	m.closed = true
	streams := make([]*portalSSHStream, 0, len(m.sshStreams))
	for id, s := range m.sshStreams {
		streams = append(streams, s)
		delete(m.sshStreams, id)
	}
	clear(m.tcpStreams)
	clear(m.pending)
	m.mu.Unlock()

	log.Debug("closing SSH mux for portal", "portal_id", m.portalID, "active_streams", len(streams))
	for _, s := range streams {
		s.Close("Portal connection closed")
	}
}

type portalSSHStream struct {
	sshStreamID string
	tcpStreamID string
	mux         *Mux
	ctx         context.Context
	cancel      context.CancelFunc
	agentOutCh  chan *portalpb.Mote
	sshInCh     chan *portalpb.SSHPayload
	outWriter   *stream.OrderedWriter
	closeOnce   sync.Once
}

func (m *Mux) startStream(ctx context.Context, sshStreamID string, connectPayload *portalpb.SSHPayload) {
	m.mu.Lock()
	if m.closed {
		m.mu.Unlock()
		return
	}
	if _, exists := m.sshStreams[sshStreamID]; exists {
		m.mu.Unlock()
		return
	}

	tcpStreamID := uuid.New().String()
	streamCtx, cancel := context.WithCancel(ensurePortalStreamAttrs(ctx, m.portalID, sshStreamID))
	topicOut := m.mux.TopicOut(m.portalID)
	outWriter := stream.NewOrderedWriter(sshStreamID, func(outMote *portalpb.Mote) error {
		pubCtx, pubCancel := context.WithTimeout(logging.DetachContext(streamCtx), 5*time.Second)
		defer pubCancel()
		return m.mux.Publish(pubCtx, topicOut, outMote)
	})

	s := &portalSSHStream{
		sshStreamID: sshStreamID,
		tcpStreamID: tcpStreamID,
		mux:         m,
		ctx:         streamCtx,
		cancel:      cancel,
		agentOutCh:  make(chan *portalpb.Mote, 1024),
		sshInCh:     make(chan *portalpb.SSHPayload, 1024),
		outWriter:   outWriter,
	}

	// Drain any pending pre-buffered payloads
	if pending, ok := m.pending[sshStreamID]; ok {
		for _, p := range pending {
			select {
			case s.sshInCh <- p:
			default:
			}
		}
		delete(m.pending, sshStreamID)
	}

	m.sshStreams[sshStreamID] = s
	m.tcpStreams[tcpStreamID] = s
	m.mu.Unlock()

	log.DebugContext(streamCtx, "starting portal SSH stream",
		"tcp_stream_id", tcpStreamID,
		"dst_addr", connectPayload.DstAddr,
		"dst_port", connectPayload.DstPort,
		"ssh_user", connectPayload.Username,
	)
	go s.run(connectPayload)
}

func (s *portalSSHStream) sendConnected() {
	if err := s.outWriter.WriteSSH(&portalpb.SSHPayload{
		Kind: portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_DATA,
	}); err != nil {
		log.DebugContext(s.ctx, "failed to publish SSH connected mote", "error", err)
	}
}

func (s *portalSSHStream) sendData(data []byte) {
	if len(data) == 0 {
		return
	}
	cp := make([]byte, len(data))
	copy(cp, data)
	log.TraceContext(s.ctx, "publishing SSH output data mote", "bytes", len(cp))
	if err := s.outWriter.WriteSSH(&portalpb.SSHPayload{
		Kind: portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_DATA,
		Data: cp,
	}); err != nil {
		log.DebugContext(s.ctx, "failed to publish SSH data mote", "error", err)
	}
}

func (s *portalSSHStream) Close(reason string) {
	s.closeOnce.Do(func() {
		log.DebugContext(logging.DetachContext(s.ctx), "closing portal SSH stream", "reason", reason)
		s.cancel()
		_ = s.outWriter.WriteSSH(&portalpb.SSHPayload{
			Kind: portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CLOSE,
			Data: []byte(reason),
		})
		s.mux.mu.Lock()
		if current, ok := s.mux.sshStreams[s.sshStreamID]; ok && current == s {
			delete(s.mux.sshStreams, s.sshStreamID)
		}
		if current, ok := s.mux.tcpStreams[s.tcpStreamID]; ok && current == s {
			delete(s.mux.tcpStreams, s.tcpStreamID)
		}
		delete(s.mux.pending, s.sshStreamID)
		s.mux.mu.Unlock()
	})
}

const handshakeIdleTimeout = 45 * time.Second

type activityConn struct {
	net.Conn
	mu          sync.Mutex
	idleTimeout time.Duration
}

func (c *activityConn) SetIdleTimeout(d time.Duration) {
	c.mu.Lock()
	c.idleTimeout = d
	c.mu.Unlock()
	if d <= 0 {
		_ = c.Conn.SetDeadline(time.Time{})
		return
	}
	_ = c.Conn.SetDeadline(time.Now().Add(d))
}

func (c *activityConn) getIdleTimeout() time.Duration {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.idleTimeout
}

func (c *activityConn) Read(b []byte) (int, error) {
	if d := c.getIdleTimeout(); d > 0 {
		_ = c.Conn.SetReadDeadline(time.Now().Add(d))
	}
	n, err := c.Conn.Read(b)
	if n > 0 {
		if d := c.getIdleTimeout(); d > 0 {
			_ = c.Conn.SetReadDeadline(time.Now().Add(d))
		}
	}
	return n, err
}

func (c *activityConn) Write(b []byte) (int, error) {
	if d := c.getIdleTimeout(); d > 0 {
		_ = c.Conn.SetWriteDeadline(time.Now().Add(d))
	}
	n, err := c.Conn.Write(b)
	if n > 0 {
		if d := c.getIdleTimeout(); d > 0 {
			_ = c.Conn.SetReadDeadline(time.Now().Add(d))
			_ = c.Conn.SetWriteDeadline(time.Now().Add(d))
		}
	}
	return n, err
}

func combineAlgorithms(supported, insecure []string) []string {
	seen := make(map[string]struct{}, len(supported)+len(insecure))
	out := make([]string, 0, len(supported)+len(insecure))
	for _, alg := range supported {
		if _, ok := seen[alg]; !ok {
			seen[alg] = struct{}{}
			out = append(out, alg)
		}
	}
	for _, alg := range insecure {
		if _, ok := seen[alg]; !ok {
			seen[alg] = struct{}{}
			out = append(out, alg)
		}
	}
	return out
}

func normalizePrivateKeyBytes(raw []byte) []byte {
	trimmed := bytes.TrimSpace(raw)
	if len(trimmed) == 0 {
		return nil
	}
	normalized := bytes.ReplaceAll(trimmed, []byte("\r\n"), []byte("\n"))
	return append(normalized, '\n')
}

func (s *portalSSHStream) run(connectPayload *portalpb.SSHPayload) {
	defer s.Close("SSH session ended")

	host := connectPayload.DstAddr
	port := int(connectPayload.DstPort)
	user := connectPayload.Username
	if host == "" || port <= 0 || user == "" {
		log.WarnContext(s.ctx, "invalid SSH connection parameters", "dst_addr", host, "dst_port", port, "ssh_user", user)
		s.Close("Invalid SSH connection parameters")
		return
	}

	var authMethods []gossh.AuthMethod
	if keyBytes := normalizePrivateKeyBytes(connectPayload.PrivateKey); len(keyBytes) > 0 {
		signer, err := gossh.ParsePrivateKey(keyBytes)
		if err != nil && connectPayload.Password != "" {
			signer, err = gossh.ParsePrivateKeyWithPassphrase(keyBytes, []byte(connectPayload.Password))
		}
		if err != nil {
			log.WarnContext(s.ctx, "invalid SSH private key", "ssh_user", user, "error", err)
			s.Close(fmt.Sprintf("Invalid SSH private key: %v", err))
			return
		}
		authMethods = append(authMethods, gossh.PublicKeys(signer))
	}
	if connectPayload.Password != "" {
		password := connectPayload.Password
		authMethods = append(authMethods,
			gossh.Password(password),
			gossh.KeyboardInteractive(func(name, instruction string, questions []string, echos []bool) ([]string, error) {
				answers := make([]string, len(questions))
				for i := range questions {
					answers[i] = password
				}
				return answers, nil
			}),
		)
	}

	supportedAlgs := gossh.SupportedAlgorithms()
	insecureAlgs := gossh.InsecureAlgorithms()

	sshConfig := &gossh.ClientConfig{
		Config: gossh.Config{
			KeyExchanges: combineAlgorithms(supportedAlgs.KeyExchanges, insecureAlgs.KeyExchanges),
			Ciphers:      combineAlgorithms(supportedAlgs.Ciphers, insecureAlgs.Ciphers),
			MACs:         combineAlgorithms(supportedAlgs.MACs, insecureAlgs.MACs),
		},
		User:              user,
		Auth:              authMethods,
		HostKeyCallback:   gossh.InsecureIgnoreHostKey(),
		HostKeyAlgorithms: combineAlgorithms(supportedAlgs.HostKeys, insecureAlgs.HostKeys),
		Timeout:           handshakeIdleTimeout,
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

	rawConn, err := pnet.Dial("tcp", hostPortStr, s.tcpStreamID, s.mux.sendToAgent, receiver)
	if err != nil {
		log.WarnContext(s.ctx, "failed to initialize portal TCP stream for SSH", "target", hostPortStr, "tcp_stream_id", s.tcpStreamID, "error", err)
		s.Close(fmt.Sprintf("Failed to initialize portal TCP stream: %v", err))
		return
	}
	defer rawConn.Close()

	// Enforce an activity-extending per-operation deadline during the SSH handshake and session setup
	conn := &activityConn{Conn: rawConn}
	conn.SetIdleTimeout(handshakeIdleTimeout)

	log.DebugContext(s.ctx, "initiating SSH handshake over portal", "target", hostPortStr, "ssh_user", user, "tcp_stream_id", s.tcpStreamID)
	c, chans, reqs, err := gossh.NewClientConn(conn, hostPortStr, sshConfig)
	if err != nil {
		log.WarnContext(s.ctx, "SSH handshake failed", "target", hostPortStr, "ssh_user", user, "error", err)
		s.Close(fmt.Sprintf("SSH handshake failed (%s@%s): %v", user, hostPortStr, err))
		return
	}
	sshClient := gossh.NewClient(c, chans, reqs)
	defer sshClient.Close()

	sshSession, err := sshClient.NewSession()
	if err != nil {
		log.WarnContext(s.ctx, "failed to open SSH session", "target", hostPortStr, "ssh_user", user, "error", err)
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
		log.WarnContext(s.ctx, "failed to request remote SSH PTY", "target", hostPortStr, "ssh_user", user, "error", err)
		s.Close(fmt.Sprintf("Failed to request PTY: %v", err))
		return
	}

	sshStdin, err := sshSession.StdinPipe()
	if err != nil {
		log.WarnContext(s.ctx, "failed to bind SSH stdin pipe", "target", hostPortStr, "error", err)
		s.Close(fmt.Sprintf("Failed to bind stdin pipe: %v", err))
		return
	}
	sshStdout, err := sshSession.StdoutPipe()
	if err != nil {
		log.WarnContext(s.ctx, "failed to bind SSH stdout pipe", "target", hostPortStr, "error", err)
		s.Close(fmt.Sprintf("Failed to bind stdout pipe: %v", err))
		return
	}
	sshStderr, err := sshSession.StderrPipe()
	if err != nil {
		log.WarnContext(s.ctx, "failed to bind SSH stderr pipe", "target", hostPortStr, "error", err)
		s.Close(fmt.Sprintf("Failed to bind stderr pipe: %v", err))
		return
	}

	if err := sshSession.Shell(); err != nil {
		log.WarnContext(s.ctx, "failed to start remote SSH shell", "target", hostPortStr, "ssh_user", user, "error", err)
		s.Close(fmt.Sprintf("Failed to start remote shell: %v", err))
		return
	}

	// Handshake and shell startup succeeded; clear the handshake deadline and notify connected
	conn.SetIdleTimeout(0)
	log.DebugContext(s.ctx, "SSH session established over portal", "target", hostPortStr, "ssh_user", user, "rows", rows, "cols", cols)
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
				log.TraceContext(s.ctx, "sending SSH keepalive request", "target", hostPortStr)
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
						log.WarnContext(s.ctx, "SSH keepalive failed", "target", hostPortStr, "error", err)
						s.Close(fmt.Sprintf("SSH connection lost: %v", err))
						return
					}
				case <-time.After(10 * time.Second):
					log.WarnContext(s.ctx, "SSH keepalive timed out", "target", hostPortStr)
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
					log.TraceContext(s.ctx, "writing SSH stdin frame", "bytes", len(payload.Data))
					if _, writeErr := sshStdin.Write(payload.Data); writeErr != nil {
						return
					}
				}
			case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_RESIZE:
				if payload.Rows > 0 && payload.Cols > 0 {
					log.DebugContext(s.ctx, "applying SSH window change", "rows", payload.Rows, "cols", payload.Cols)
					_ = sshSession.WindowChange(int(payload.Rows), int(payload.Cols))
				}
			case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CLOSE:
				log.DebugContext(s.ctx, "received SSH close payload on stream")
				return
			}
		}
	}
}
