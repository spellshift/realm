package pty

import (
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"strconv"
	"sync"
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

// PivotSession represents an active PTY session that can be shared among multiple websockets.
type PivotSession struct {
	PivotID       int
	StreamID      string
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

// Handler handles websocket connections that tunnel PTY sessions over portal network.
type Handler struct {
	graph *ent.Client
	mux   *mux.Mux
}

// NewHandler creates a new PTY websocket handler.
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
		slog.ErrorContext(ctx, "failed to upgrade pty websocket", "error", err)
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
		if portalIDStr == "" {
			sendWsError("missing portal_id")
			return
		}

		portalID, err := strconv.Atoi(portalIDStr)
		if err != nil {
			sendWsError("invalid portal_id")
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

		// Stream ID
		streamID := fmt.Sprintf("pty-ws-%d", time.Now().UnixNano())

		// Create ShellPivot Ent
		createReq := h.graph.ShellPivot.Create().
			SetStreamID(streamID).
			SetKind("pty").
			SetDestination("pty").
			SetPort(0).
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

		// Setup portal communication
		sessionCtx, cancelSession := context.WithCancel(context.Background())

		recvCh, recvCleanup := h.mux.Subscribe(topicOut)

		pivotSession = &PivotSession{
			PivotID:  pivotEnt.ID,
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

		// Background goroutine: read portal output -> broadcast to websockets
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
					// Only accept motes matching our streamID
					if m.StreamId != streamID {
						continue
					}
					// Check for close
					if m.GetBytes() != nil && m.GetBytes().Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
						closePivot()
						return
					}
					// Handle PTY data
					if bytesPayload := m.GetBytes(); bytesPayload != nil && bytesPayload.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_PTY {
						pivotSession.Broadcast(shell.WebsocketTaskOutputMessage{
							Kind:   shell.WebsocketMessageKindOutput,
							Output: string(bytesPayload.Data),
						})
					}
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

		// Send initial PTY request mote to the agent to spawn the PTY
		initMote := &portalpb.Mote{
			StreamId: streamID,
			SeqId:    0,
			Payload: &portalpb.Mote_Bytes{
				Bytes: &portalpb.BytesPayload{
					Data: []byte{},
					Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_PTY,
				},
			},
		}
		if err := h.mux.Publish(sessionCtx, topicIn, initMote); err != nil {
			sendWsError(fmt.Sprintf("Failed to send PTY init mote: %v", err))
			closePivot()
			return
		}

		// WebSocket -> Portal PTY Input goroutine (spawned below in the common path)
		// needs access to topicIn and sessionCtx, so we wrap them in a closure
		startInputForwarder := func(wsConn *websocket.Conn, errCh chan error) {
			go func() {
				var seqID uint64
				for {
					_, msg, err := wsConn.ReadMessage()
					if err != nil {
						errCh <- err
						return
					}
					var inputMsg shell.WebsocketTaskInputMessage
					if err := json.Unmarshal(msg, &inputMsg); err != nil || inputMsg.Kind != shell.WebsocketMessageKindInput {
						continue
					}
					seqID++
					mote := &portalpb.Mote{
						StreamId: streamID,
						SeqId:    seqID,
						Payload: &portalpb.Mote_Bytes{
							Bytes: &portalpb.BytesPayload{
								Data: []byte(inputMsg.Input),
								Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_PTY,
							},
						},
					}
					if err := h.mux.Publish(sessionCtx, topicIn, mote); err != nil {
						errCh <- err
						return
					}
				}
			}()
		}

		// Attach current WS to PivotSession
		outCh := make(chan interface{}, 32)
		pivotSession.AddSocket(wsConn, outCh)
		defer pivotSession.RemoveSocket(wsConn)

		errCh := make(chan error, 2)

		// Writer goroutine: PivotSession output -> websocket
		go func() {
			for msg := range outCh {
				if err := wsConn.WriteJSON(msg); err != nil {
					errCh <- err
					return
				}
			}
		}()

		// Reader goroutine: websocket -> portal PTY input
		startInputForwarder(wsConn, errCh)

		// Wait for disconnect
		<-errCh
		return
	}

	// Reconnecting to existing session - attach websocket
	outCh := make(chan interface{}, 32)
	pivotSession.AddSocket(wsConn, outCh)
	defer pivotSession.RemoveSocket(wsConn)

	errCh := make(chan error, 2)

	// Writer goroutine
	go func() {
		for msg := range outCh {
			if err := wsConn.WriteJSON(msg); err != nil {
				errCh <- err
				return
			}
		}
	}()

	// Reader goroutine - for reconnects we need the portal info
	// The pivot session tracks the stream context, but we need topicIn
	// For reconnects, we look up the portal from the pivot
	go func() {
		for {
			_, msg, err := wsConn.ReadMessage()
			if err != nil {
				errCh <- err
				return
			}
			var inputMsg shell.WebsocketTaskInputMessage
			if err := json.Unmarshal(msg, &inputMsg); err != nil || inputMsg.Kind != shell.WebsocketMessageKindInput {
				continue
			}

			// For reconnects, look up the portal from the pivot
			pivot, err := h.graph.ShellPivot.Get(ctx, pivotSession.PivotID)
			if err != nil {
				errCh <- err
				return
			}
			portalEnt, err := pivot.QueryPortal().Only(ctx)
			if err != nil {
				errCh <- err
				return
			}

			topicIn := h.mux.TopicIn(portalEnt.ID)
			mote := &portalpb.Mote{
				StreamId: pivotSession.StreamID,
				SeqId:    0,
				Payload: &portalpb.Mote_Bytes{
					Bytes: &portalpb.BytesPayload{
						Data: []byte(inputMsg.Input),
						Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_PTY,
					},
				},
			}
			if err := h.mux.Publish(ctx, topicIn, mote); err != nil {
				slog.ErrorContext(ctx, "failed to publish pty input", "error", err)
			}
		}
	}()

	// Wait for disconnect
	<-errCh
}
