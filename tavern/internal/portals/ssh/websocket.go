package ssh

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
	"sync/atomic"
	"time"

	"github.com/gorilla/websocket"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/portal"
	"realm.pub/tavern/internal/http/shell"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin:     func(r *http.Request) bool { return true },
}

// WebsocketResizeOrCloseMessage represents control messages (resize/close) from the client.
type WebsocketControlMessage struct {
	Kind  string `json:"kind"`
	Input string `json:"input,omitempty"`
	Rows  uint32 `json:"rows,omitempty"`
	Cols  uint32 `json:"cols,omitempty"`
}

// PivotSession represents an active SSH session that can be shared among multiple websockets.
// SSH protocol negotiation happens on the gRPC-receiving Tavern instance via SSHPayload motes.
type PivotSession struct {
	PivotID       int
	PortalID      int
	StreamID      string
	CancelContext context.CancelFunc
	seqID         uint64

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

func (ps *PivotSession) nextSeqID() uint64 {
	return atomic.AddUint64(&ps.seqID, 1)
}

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

	// Notify and close all active socket channels so writer goroutines flush before closing
	ps.mu.Lock()
	for _, ch := range ps.activeSockets {
		select {
		case ch <- shell.WebsocketStatusMessage{
			Kind:   shell.WebsocketMessageKindStatus,
			Status: "closed",
		}:
		default:
		}
		close(ch)
	}
	ps.activeSockets = make(map[*websocket.Conn]chan interface{})
	ps.mu.Unlock()
}

// Handler handles websocket connections that tunnel SSH sessions over portal network.
type Handler struct {
	graph *ent.Client
	mux   *mux.Mux
}

// NewHandler creates a new SSH websocket handler.
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
		slog.ErrorContext(ctx, "failed to upgrade ssh websocket", "error", err)
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
			// If session is no longer active in memory, serve any persisted history from DB and report closed status
			if pivotEnt, err := h.graph.ShellPivot.Get(ctx, pivotID); err == nil {
				if pivotEnt.Data != "" {
					_ = wsConn.WriteJSON(shell.WebsocketTaskOutputMessage{
						Kind:   shell.WebsocketMessageKindOutput,
						Output: pivotEnt.Data,
					})
				}
				if pivotEnt.ClosedAt.IsZero() {
					_ = h.graph.ShellPivot.UpdateOneID(pivotID).SetClosedAt(time.Now()).Exec(ctx)
				}
				_ = wsConn.WriteJSON(shell.WebsocketStatusMessage{
					Kind:   shell.WebsocketMessageKindStatus,
					Status: "closed",
				})
			}
			sendWsError("pivot session not found or expired")
			_ = wsConn.WriteControl(
				websocket.CloseMessage,
				websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""),
				time.Now().Add(time.Second),
			)
			_ = wsConn.SetReadDeadline(time.Now().Add(250 * time.Millisecond))
			_, _, _ = wsConn.ReadMessage()
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

		// Parse target: defaults to root@<target>:22
		user := "root"
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
				host = hostPortStr
				portStr = "22"
				hostPortStr = net.JoinHostPort(host, portStr)
			} else {
				sendWsError(fmt.Sprintf("invalid target host format: %v", err))
				return
			}
		} else {
			hostPortStr = net.JoinHostPort(host, portStr)
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

		// Stream ID
		streamID := fmt.Sprintf("ssh-ws-%d", time.Now().UnixNano())

		// Extract port as uint32
		dstPort, _ := strconv.ParseUint(portStr, 10, 32)
		if dstPort == 0 {
			dstPort = 22
		}

		// Create ShellPivot Ent
		createReq := h.graph.ShellPivot.Create().
			SetStreamID(streamID).
			SetKind("ssh").
			SetDestination(hostPortStr).
			SetPort(int(dstPort)).
			SetPortalID(portalID)

		if shellIDStr != "" {
			if shellID, err := strconv.Atoi(shellIDStr); err == nil {
				createReq = createReq.SetShellID(shellID)
			}
		}

		pivotEnt, err := createReq.Save(ctx)
		if err != nil {
			sendWsError(fmt.Sprintf("Failed to create shell pivot: %v", err))
			cleanupPortal()
			return
		}

		sessionCtx, cancelSession := context.WithCancel(context.Background())
		recvCh, recvCleanup := h.mux.Subscribe(topicOut)

		pivotSession = &PivotSession{
			PivotID:  pivotEnt.ID,
			PortalID: portalID,
			StreamID: streamID,
			CancelContext: func() {
				cancelSession()
				recvCleanup()
				cleanupPortal()
			},
			activeSockets: make(map[*websocket.Conn]chan interface{}),
		}

		cacheMu.Lock()
		sessionCache[pivotEnt.ID] = pivotSession
		cacheMu.Unlock()

		var closeOnce sync.Once
		closePivot := func() {
			closeOnce.Do(func() {
				pivotSession.Close(context.Background(), h.graph)
			})
		}

		// Background goroutine: read portal SSH output motes -> broadcast to websockets.
		go func() {
			for {
				select {
				case <-sessionCtx.Done():
					return
				case m, ok := <-recvCh:
					if !ok {
						closePivot()
						return
					}
					if m.StreamId != streamID {
						continue
					}
					if sshPayload := m.GetSsh(); sshPayload != nil {
						switch sshPayload.Kind {
						case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_DATA:
							if len(sshPayload.Data) > 0 {
								pivotSession.Broadcast(shell.WebsocketTaskOutputMessage{
									Kind:   shell.WebsocketMessageKindOutput,
									Output: string(sshPayload.Data),
								})
							}
						case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CLOSE:
							if len(sshPayload.Data) > 0 {
								pivotSession.Broadcast(shell.WebsocketErrorMessage{
									Kind:  shell.WebsocketMessageKindError,
									Error: string(sshPayload.Data),
								})
							}
							closePivot()
							return
						}
						continue
					}
					if m.GetBytes() != nil && m.GetBytes().Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
						closePivot()
						return
					}
				}
			}
		}()

		// Periodic history flush
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

		// Attach current WS to PivotSession BEFORE publishing CONNECT so fast responses aren't missed
		outCh := make(chan interface{}, 64)
		pivotSession.AddSocket(wsConn, outCh)
		defer pivotSession.RemoveSocket(wsConn)

		// Publish SSH_PAYLOAD_KIND_CONNECT mote to instruct the gRPC-receiving Tavern instance
		// to initiate the SSH protocol over the portal stream.
		connectMote := &portalpb.Mote{
			StreamId: streamID,
			SeqId:    0,
			Payload: &portalpb.Mote_Ssh{
				Ssh: &portalpb.SSHPayload{
					Kind:     portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CONNECT,
					DstAddr:  host,
					DstPort:  uint32(dstPort),
					Username: user,
					Password: password,
					Rows:     40,
					Cols:     80,
				},
			},
		}
		if err := h.mux.Publish(sessionCtx, topicIn, connectMote); err != nil {
			sendWsError(fmt.Sprintf("Failed to send SSH connect mote: %v", err))
			closePivot()
			return
		}

		h.serveSocketIO(ctx, wsConn, pivotSession, outCh, closePivot)
		return
	}

	// Attach current WS to existing PivotSession
	outCh := make(chan interface{}, 64)
	pivotSession.AddSocket(wsConn, outCh)
	defer pivotSession.RemoveSocket(wsConn)

	closePivot := func() {
		pivotSession.Close(context.Background(), h.graph)
	}

	h.serveSocketIO(ctx, wsConn, pivotSession, outCh, closePivot)
}

func (h *Handler) serveSocketIO(
	ctx context.Context,
	wsConn *websocket.Conn,
	pivotSession *PivotSession,
	outCh chan interface{},
	closePivot func(),
) {
	errCh := make(chan error, 2)
	topicIn := h.mux.TopicIn(pivotSession.PortalID)

	// Single writer goroutine for websocket
	go func() {
		for msg := range outCh {
			if err := wsConn.WriteJSON(msg); err != nil {
				errCh <- err
				return
			}
		}
		errCh <- io.EOF
	}()

	// WebSocket -> SSH portal motes
	go func() {
		for {
			_, msg, err := wsConn.ReadMessage()
			if err != nil {
				errCh <- err
				return
			}
			var ctrlMsg WebsocketControlMessage
			if err := json.Unmarshal(msg, &ctrlMsg); err != nil {
				continue
			}
			switch ctrlMsg.Kind {
			case shell.WebsocketMessageKindInput:
				mote := &portalpb.Mote{
					StreamId: pivotSession.StreamID,
					SeqId:    pivotSession.nextSeqID(),
					Payload: &portalpb.Mote_Ssh{
						Ssh: &portalpb.SSHPayload{
							Kind: portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_DATA,
							Data: []byte(ctrlMsg.Input),
						},
					},
				}
				if err := h.mux.Publish(ctx, topicIn, mote); err != nil {
					errCh <- err
					return
				}
			case "resize":
				if ctrlMsg.Rows > 0 && ctrlMsg.Cols > 0 {
					mote := &portalpb.Mote{
						StreamId: pivotSession.StreamID,
						SeqId:    pivotSession.nextSeqID(),
						Payload: &portalpb.Mote_Ssh{
							Ssh: &portalpb.SSHPayload{
								Kind: portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_RESIZE,
								Rows: ctrlMsg.Rows,
								Cols: ctrlMsg.Cols,
							},
						},
					}
					_ = h.mux.Publish(ctx, topicIn, mote)
				}
			case "close":
				mote := &portalpb.Mote{
					StreamId: pivotSession.StreamID,
					SeqId:    pivotSession.nextSeqID(),
					Payload: &portalpb.Mote_Ssh{
						Ssh: &portalpb.SSHPayload{
							Kind: portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CLOSE,
						},
					},
				}
				_ = h.mux.Publish(ctx, topicIn, mote)
				closePivot()
				errCh <- io.EOF
				return
			}
		}
	}()

	// Wait for websocket to disconnect or session to close
	<-errCh
}
