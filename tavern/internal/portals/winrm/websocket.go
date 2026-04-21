package winrm

import (
	"context"
	"encoding/json"
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
	winrmlib "github.com/masterzen/winrm"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/portal"
	"realm.pub/tavern/internal/http/shell"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/portals/pnet"
	"realm.pub/tavern/portals/portalpb"
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin:     func(r *http.Request) bool { return true },
}

// PivotSession represents an active WinRM session that can be shared among multiple websockets.
type PivotSession struct {
	PivotID       int
	StreamID      string
	Stdin         io.WriteCloser
	CancelContext context.CancelFunc

	mu            sync.Mutex
	history       []byte
	dirty         bool
	activeSockets map[*websocket.Conn]chan interface{}
}

// sessionCache maintains a map of active PivotSessions.
var (
	cacheMu      sync.Mutex
	sessionCache = make(map[int]*PivotSession)
)

func (ps *PivotSession) Broadcast(msg interface{}) {
	ps.mu.Lock()
	defer ps.mu.Unlock()

	// Append to history if it's an output message
	if outMsg, ok := msg.(shell.WebsocketTaskOutputMessage); ok {
		ps.history = append(ps.history, []byte(outMsg.Output)...)
		ps.dirty = true
	} else if errMsg, ok := msg.(shell.WebsocketTaskErrorMessage); ok {
		ps.history = append(ps.history, []byte(errMsg.Error)...)
		ps.dirty = true
	}

	for _, ch := range ps.activeSockets {
		select {
		case ch <- msg:
		default:
			// channel full, skip
		}
	}
}

func (ps *PivotSession) AddSocket(wsConn *websocket.Conn, outCh chan interface{}) {
	ps.mu.Lock()
	defer ps.mu.Unlock()

	ps.activeSockets[wsConn] = outCh

	// Send history to catch up
	if len(ps.history) > 0 {
		msg := shell.WebsocketTaskOutputMessage{
			Kind:   shell.WebsocketMessageKindOutput,
			Output: string(ps.history),
		}
		select {
		case outCh <- msg:
		default:
		}
	}
}

func (ps *PivotSession) RemoveSocket(wsConn *websocket.Conn) {
	ps.mu.Lock()
	defer ps.mu.Unlock()

	if ch, ok := ps.activeSockets[wsConn]; ok {
		close(ch)
	}
	delete(ps.activeSockets, wsConn)
}

func (ps *PivotSession) FlushHistory(ctx context.Context, graph *ent.Client) {
	ps.mu.Lock()
	dirty := ps.dirty
	history := make([]byte, len(ps.history))
	copy(history, ps.history)
	ps.dirty = false
	ps.mu.Unlock()

	if dirty {
		_ = graph.ShellPivot.UpdateOneID(ps.PivotID).SetData(string(history)).Exec(ctx)
	}
}

func (ps *PivotSession) Close(ctx context.Context, graph *ent.Client) {
	cacheMu.Lock()
	delete(sessionCache, ps.PivotID)
	cacheMu.Unlock()

	if ps.CancelContext != nil {
		ps.CancelContext()
	}

	// Final flush & close
	ps.FlushHistory(ctx, graph)
	_ = graph.ShellPivot.UpdateOneID(ps.PivotID).SetClosedAt(time.Now()).Exec(ctx)

	// Close all websockets and channels
	ps.mu.Lock()
	for ws, ch := range ps.activeSockets {
		ws.Close()
		close(ch)
	}
	ps.activeSockets = make(map[*websocket.Conn]chan interface{})
	ps.mu.Unlock()
}

// Handler handles websocket connections that tunnel WinRM sessions over portal network.
type Handler struct {
	graph *ent.Client
	mux   *mux.Mux
}

// NewHandler creates a new WinRM websocket handler.
func NewHandler(graph *ent.Client, mux *mux.Mux) *Handler {
	return &Handler{
		graph: graph,
		mux:   mux,
	}
}

func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	// Upgrade to websocket first to send errors over ws
	wsConn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		slog.ErrorContext(ctx, "failed to upgrade winrm websocket", "error", err)
		return
	}
	defer wsConn.Close()

	// Helper to send errors over websocket
	sendWsError := func(errMsg string) {
		wsConn.WriteJSON(shell.WebsocketErrorMessage{
			Kind:  shell.WebsocketMessageKindError,
			Error: errMsg,
		})
	}

	// Parse Query Parameters
	pivotIDStr := r.URL.Query().Get("pivot_id")
	portalIDStr := r.URL.Query().Get("portal_id")
	target := r.URL.Query().Get("target")
	shellIDStr := r.URL.Query().Get("shell_id")

	var pivotSession *PivotSession

	if pivotIDStr != "" {
		// Reconnect to existing session
		pivotID, err := strconv.Atoi(pivotIDStr)
		if err != nil {
			sendWsError("invalid pivot_id")
			return
		}

		cacheMu.Lock()
		sess, ok := sessionCache[pivotID]
		cacheMu.Unlock()

		if !ok {
			sendWsError("pivot session not found or expired")
			return
		}
		pivotSession = sess
	} else {
		// Create new session
		if portalIDStr == "" || target == "" {
			sendWsError("missing portal_id or target")
			return
		}

		portalID, err := strconv.Atoi(portalIDStr)
		if err != nil {
			sendWsError("invalid portal_id")
			return
		}

		// Require a shellID
		if shellIDStr == "" {
			sendWsError("missing shell_id")
			return
		}

		// Parse target: defaults to Administrator@<target>:5985
		user := "Administrator"
		password := ""
		hostPortStr := target

		parts := strings.SplitN(target, "@", 2)
		if len(parts) == 2 {
			userPassStr := parts[0]
			hostPortStr = parts[1]

			userPass := strings.SplitN(userPassStr, ":", 2)
			user = userPass[0]
			if len(userPass) == 2 {
				password = userPass[1]
			}
		}

		host, portStr, err := net.SplitHostPort(hostPortStr)
		if err != nil {
			if strings.Contains(err.Error(), "missing port in address") || strings.Contains(err.Error(), "too many colons in address") {
				hostPortStr = net.JoinHostPort(hostPortStr, "5985")
				portStr = "5985"
				host, _, _ = net.SplitHostPort(hostPortStr)
			} else {
				sendWsError(fmt.Sprintf("invalid target host format: %v", err))
				return
			}
		}

		winrmPort, err := strconv.Atoi(portStr)
		if err != nil {
			sendWsError(fmt.Sprintf("invalid target port: %v", err))
			return
		}

		// Verify Portal
		p, err := h.graph.Portal.Query().Where(portal.ID(portalID)).Only(ctx)
		if err != nil {
			sendWsError("portal not found")
			return
		}
		if !p.ClosedAt.IsZero() {
			sendWsError("portal is closed")
			return
		}

		// Open Portal Mux
		cleanupPortal, err := h.mux.OpenPortal(ctx, portalID)
		if err != nil {
			sendWsError(fmt.Sprintf("Failed to open portal: %v", err))
			return
		}

		topicIn := h.mux.TopicIn(portalID)
		topicOut := h.mux.TopicOut(portalID)

		// Stream ID prefix for this WinRM session
		sessionID := fmt.Sprintf("winrm-ws-%d", time.Now().UnixNano())

		// Extract port as uint32 for ShellPivot
		dstPort, _ := strconv.ParseUint(portStr, 10, 32)

		// Create ShellPivot Ent
		createReq := h.graph.ShellPivot.Create().
			SetStreamID(sessionID).
			SetKind("winrm").
			SetDestination(net.JoinHostPort(host, portStr)).
			SetPort(int(dstPort)).
			SetPortalID(portalID)

		if shellID, err := strconv.Atoi(shellIDStr); err == nil {
			createReq = createReq.SetShellID(shellID)
		}

		pivotEnt, err := createReq.Save(ctx)
		if err != nil {
			sendWsError(fmt.Sprintf("Failed to create shell pivot: %v", err))
			cleanupPortal()
			return
		}

		sessionCtx, cancelSession := context.WithCancel(context.Background())

		// dialMu protects cleanups slice from concurrent dial calls
		var dialMu sync.Mutex
		var connCleanups []func()

		// dial creates a new pnet TCP connection for each HTTP connection WinRM needs.
		// Each connection gets its own stream ID and mux subscription so that motes
		// are correctly routed without interfering with other concurrent connections.
		var dialCounter int64
		dial := func(network, addr string) (net.Conn, error) {
			dialMu.Lock()
			dialCounter++
			connStreamID := fmt.Sprintf("%s-conn-%d", sessionID, dialCounter)
			dialMu.Unlock()

			sender := func(m *portalpb.Mote) error {
				return h.mux.Publish(sessionCtx, topicIn, m)
			}

			recvCh, recvCleanup := h.mux.Subscribe(topicOut)

			receiver := func() (*portalpb.Mote, error) {
				for {
					select {
					case <-sessionCtx.Done():
						return nil, io.EOF
					case m, ok := <-recvCh:
						if !ok {
							return nil, io.EOF
						}
						// Only accept motes for this connection's stream
						if m.StreamId != connStreamID {
							continue
						}
						if m.GetBytes() != nil && m.GetBytes().Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
							return nil, io.EOF
						}
						return m, nil
					}
				}
			}

			conn, err := pnet.Dial("tcp", net.JoinHostPort(host, portStr), connStreamID, sender, receiver)
			if err != nil {
				recvCleanup()
				return nil, err
			}

			dialMu.Lock()
			connCleanups = append(connCleanups, func() {
				conn.Close()
				recvCleanup()
			})
			dialMu.Unlock()

			return conn, nil
		}

		// Build WinRM client with custom Dial that routes through the portal
		params := winrmlib.NewParameters("PT60S", "en-US", 153600)
		params.Dial = dial

		endpoint := winrmlib.NewEndpoint(host, winrmPort, false, true, nil, nil, nil, 30*time.Second)
		winrmClient, err := winrmlib.NewClientWithParameters(endpoint, user, password, params)
		if err != nil {
			sendWsError(fmt.Sprintf("Failed to create WinRM client: %v", err))
			cancelSession()
			cleanupPortal()
			return
		}

		// Create WinRM shell
		winrmShell, err := winrmClient.CreateShell()
		if err != nil {
			sendWsError(fmt.Sprintf("Failed to create WinRM shell: %v", err))
			cancelSession()
			dialMu.Lock()
			for _, cleanup := range connCleanups {
				cleanup()
			}
			dialMu.Unlock()
			cleanupPortal()
			return
		}

		// Execute cmd.exe for interactive session
		cmd, err := winrmShell.ExecuteWithContext(sessionCtx, "cmd.exe")
		if err != nil {
			sendWsError(fmt.Sprintf("Failed to start cmd.exe: %v", err))
			winrmShell.Close()
			cancelSession()
			dialMu.Lock()
			for _, cleanup := range connCleanups {
				cleanup()
			}
			dialMu.Unlock()
			cleanupPortal()
			return
		}

		pivotSession = &PivotSession{
			PivotID:  pivotEnt.ID,
			StreamID: sessionID,
			Stdin:    cmd.Stdin,
			CancelContext: func() {
				cmd.Close()
				winrmShell.Close()
				cancelSession()
				dialMu.Lock()
				for _, cleanup := range connCleanups {
					cleanup()
				}
				dialMu.Unlock()
				cleanupPortal()
			},
			activeSockets: make(map[*websocket.Conn]chan interface{}),
		}

		cacheMu.Lock()
		sessionCache[pivotEnt.ID] = pivotSession
		cacheMu.Unlock()

		// We use sync.Once to ensure Close is only called once.
		var closeOnce sync.Once
		closePivot := func() {
			closeOnce.Do(func() {
				pivotSession.Close(context.Background(), h.graph)
			})
		}

		// Background goroutine: stdout -> broadcast to websockets
		go func() {
			buf := make([]byte, 1024)
			for {
				n, err := cmd.Stdout.Read(buf)
				if n > 0 {
					pivotSession.Broadcast(shell.WebsocketTaskOutputMessage{
						Kind:   shell.WebsocketMessageKindOutput,
						Output: string(buf[:n]),
					})
				}
				if err != nil {
					closePivot()
					return
				}
			}
		}()

		// Background goroutine: stderr -> broadcast to websockets
		go func() {
			buf := make([]byte, 1024)
			for {
				n, err := cmd.Stderr.Read(buf)
				if n > 0 {
					pivotSession.Broadcast(shell.WebsocketTaskErrorMessage{
						Kind:  shell.WebsocketMessageKindTaskError,
						Error: string(buf[:n]),
					})
				}
				if err != nil {
					closePivot()
					return
				}
			}
		}()

		// Background goroutine: periodic history flush
		go func() {
			ticker := time.NewTicker(5 * time.Second)
			defer ticker.Stop()
			for {
				select {
				case <-sessionCtx.Done():
					return
				case <-ticker.C:
					pivotSession.FlushHistory(context.Background(), h.graph)
				}
			}
		}()
	}

	// Attach current WS to PivotSession
	outCh := make(chan interface{}, 32)
	pivotSession.AddSocket(wsConn, outCh)
	defer pivotSession.RemoveSocket(wsConn)

	errCh := make(chan error, 2)

	// Single writer goroutine for websocket
	go func() {
		for msg := range outCh {
			if err := wsConn.WriteJSON(msg); err != nil {
				errCh <- err
				return
			}
		}
	}()

	// WebSocket -> WinRM stdin
	go func() {
		for {
			_, msg, err := wsConn.ReadMessage()
			if err != nil {
				errCh <- err
				return
			}
			var inputMsg shell.WebsocketTaskInputMessage
			if err := json.Unmarshal(msg, &inputMsg); err == nil && inputMsg.Kind == shell.WebsocketMessageKindInput {
				pivotSession.Stdin.Write([]byte(inputMsg.Input))
			}
		}
	}()

	// Wait for websocket to disconnect
	<-errCh
}
