package http1

import (
	"context"
	"log/slog"
	"sync"
	"time"

	"google.golang.org/grpc"
)

const (
	// sessionHeader is the HTTP header used to identify a persistent stream session.
	sessionHeader = "X-Stream-Session-Id"

	// recvBufferSize is the max number of server messages buffered per session.
	recvBufferSize = 64

	// sessionIdleTimeout is how long a session persists without any polls before cleanup.
	sessionIdleTimeout = 30 * time.Second

	// sessionCleanupInterval is how often the reaper runs.
	sessionCleanupInterval = 10 * time.Second

	// recvDrainTimeout is how long to wait for responses after sending messages.
	recvDrainTimeout = 100 * time.Millisecond
)

// streamSession holds a persistent gRPC bidirectional stream and a buffer of
// server-side messages that have been received between polls.
type streamSession struct {
	stream   grpc.ClientStream
	cancel   context.CancelFunc
	mu       sync.Mutex
	recvBuf  chan []byte // buffered server→client messages
	lastPoll time.Time
	closed   bool
	cfg      streamConfig
}

// streamSessionManager maintains persistent gRPC streams across HTTP polls.
type streamSessionManager struct {
	mu       sync.Mutex
	sessions map[string]*streamSession
}

var sessionManager = &streamSessionManager{
	sessions: make(map[string]*streamSession),
}

func init() {
	go sessionManager.reapLoop()
}

// getOrCreate returns an existing session or creates a new one.
func (m *streamSessionManager) getOrCreate(sessionID string, conn *grpc.ClientConn, cfg streamConfig) (*streamSession, error) {
	m.mu.Lock()
	sess, ok := m.sessions[sessionID]
	if ok && !sess.closed {
		sess.mu.Lock()
		sess.lastPoll = time.Now()
		sess.mu.Unlock()
		m.mu.Unlock()
		return sess, nil
	}
	m.mu.Unlock()

	// Create new persistent stream with a long-lived context.
	ctx, cancel := context.WithCancel(context.Background())
	stream, err := conn.NewStream(
		ctx,
		&cfg.Desc,
		cfg.MethodPath,
		grpc.CallContentSubtype("raw"),
	)
	if err != nil {
		cancel()
		return nil, err
	}

	sess = &streamSession{
		stream:   stream,
		cancel:   cancel,
		recvBuf:  make(chan []byte, recvBufferSize),
		lastPoll: time.Now(),
		cfg:      cfg,
	}

	// Start background receiver goroutine.
	go sess.recvLoop()

	m.mu.Lock()
	m.sessions[sessionID] = sess
	m.mu.Unlock()

	slog.Info("http1 redirector: created persistent stream session", "session_id", sessionID, "method", cfg.MethodPath)
	return sess, nil
}

// remove closes and removes a session.
func (m *streamSessionManager) remove(sessionID string) {
	m.mu.Lock()
	sess, ok := m.sessions[sessionID]
	if ok {
		delete(m.sessions, sessionID)
	}
	m.mu.Unlock()
	if ok {
		sess.close()
		slog.Info("http1 redirector: removed stream session", "session_id", sessionID)
	}
}

// reapLoop periodically removes idle sessions.
func (m *streamSessionManager) reapLoop() {
	ticker := time.NewTicker(sessionCleanupInterval)
	defer ticker.Stop()

	for range ticker.C {
		var toRemove []string

		m.mu.Lock()
		now := time.Now()
		for id, sess := range m.sessions {
			sess.mu.Lock()
			if sess.closed || now.Sub(sess.lastPoll) > sessionIdleTimeout {
				toRemove = append(toRemove, id)
			}
			sess.mu.Unlock()
		}
		for _, id := range toRemove {
			if sess, ok := m.sessions[id]; ok {
				sess.close()
				delete(m.sessions, id)
			}
		}
		m.mu.Unlock()

		for _, id := range toRemove {
			slog.Info("http1 redirector: reaped idle stream session", "session_id", id)
		}
	}
}

// send sends a raw message on the persistent gRPC stream.
func (s *streamSession) send(msg []byte) error {
	return s.stream.SendMsg(msg)
}

// drain returns all currently buffered server messages (non-blocking after a short wait).
func (s *streamSession) drain() [][]byte {
	var msgs [][]byte

	// Wait briefly for at least one message.
	select {
	case msg, ok := <-s.recvBuf:
		if ok {
			msgs = append(msgs, msg)
		}
	case <-time.After(recvDrainTimeout):
	}

	// Drain any remaining buffered messages without blocking.
	for {
		select {
		case msg, ok := <-s.recvBuf:
			if !ok {
				return msgs
			}
			msgs = append(msgs, msg)
		default:
			return msgs
		}
	}
}

// recvLoop runs in a goroutine, continuously receiving server messages and
// buffering them for the next poll.
func (s *streamSession) recvLoop() {
	for {
		var msg []byte
		err := s.stream.RecvMsg(&msg)
		if err != nil {
			s.mu.Lock()
			s.closed = true
			s.mu.Unlock()
			close(s.recvBuf)
			slog.Debug("http1 redirector: stream session recv ended", "method", s.cfg.MethodPath, "error", err)
			return
		}
		select {
		case s.recvBuf <- msg:
		default:
			slog.Warn("http1 redirector: stream session recv buffer full, dropping message", "method", s.cfg.MethodPath)
		}
	}
}

// close marks the session closed and cancels the stream context.
func (s *streamSession) close() {
	s.mu.Lock()
	alreadyClosed := s.closed
	s.closed = true
	s.mu.Unlock()
	if !alreadyClosed {
		_ = s.stream.CloseSend()
		s.cancel()
	}
}
