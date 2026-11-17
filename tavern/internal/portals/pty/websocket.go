package pty

import (
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"strconv"
	"sync"
	"sync/atomic"
	"time"

	"github.com/gorilla/websocket"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/portal"
	"realm.pub/tavern/internal/http/shell"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
)

var log = logging.For("portals.pty")

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin:     func(r *http.Request) bool { return true },
}

// WebsocketControlMessage represents a message from the websocket client (input, resize, or close).
type WebsocketControlMessage struct {
	Kind  string `json:"kind"`
	Input string `json:"input"`
	Rows  uint32 `json:"rows"`
	Cols  uint32 `json:"cols"`
}

// PivotSession represents an active PTY session that can be shared among multiple websockets.
type PivotSession struct {
	PivotID       int
	PortalID      int
	BeaconID      int
	HostID        int
	StreamID      string
	SessionCtx    context.Context
	CancelContext context.CancelFunc
	seqID         uint64

	mu            sync.Mutex
	closed        bool
	history       []byte
	dirty         bool
	activeSockets map[*websocket.Conn]chan interface{}
}

func (ps *PivotSession) nextSeqID() uint64 {
	return atomic.AddUint64(&ps.seqID, 1)
}

func (ps *PivotSession) auditArgs() []any {
	args := append(
		logging.TargetIntAttrs("pivot", ps.PivotID, "pty"),
		slog.Int("pivot_id", ps.PivotID),
		slog.Int("portal_id", ps.PortalID),
	)
	if ps.BeaconID > 0 {
		args = append(args, slog.Int("beacon_id", ps.BeaconID))
	}
	if ps.HostID > 0 {
		args = append(args, slog.Int("host_id", ps.HostID))
	}
	return args
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

	// Notify socket of pivot ID and connected status
	select {
	case outCh <- shell.WebsocketStatusMessage{
		Kind:    shell.WebsocketMessageKindStatus,
		Status:  "connected",
		PivotID: ps.PivotID,
	}:
	default:
	}

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
	ps.mu.Lock()
	if ps.closed {
		ps.mu.Unlock()
		return
	}
	ps.closed = true
	ps.mu.Unlock()

	cacheMu.Lock()
	delete(sessionCache, ps.PivotID)
	cacheMu.Unlock()

	if ps.CancelContext != nil {
		ps.CancelContext()
	}

	dbCtx := logging.DetachContext(ctx)

	// Final flush & close
	ps.FlushHistory(dbCtx, graph)
	_ = graph.ShellPivot.UpdateOneID(ps.PivotID).SetClosedAt(time.Now()).Exec(dbCtx)

	auditCtx := ps.SessionCtx
	if actor := logging.ActorFromContext(ctx); actor.Kind != logging.ActorUnknown && actor.Kind != logging.ActorAnonymous {
		auditCtx = dbCtx
	}
	if auditCtx == nil {
		auditCtx = dbCtx
	}
	log.DebugContext(auditCtx, "closed PTY pivot session")
	logging.Audit(auditCtx, "portal", "portal.pty_pivot_closed", "PTY pivot closed", ps.auditArgs()...)

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

// ClosePivotSession closes and removes an active cached PTY session by pivot ID.
func ClosePivotSession(ctx context.Context, graph *ent.Client, pivotID int) bool {
	cacheMu.Lock()
	sess, ok := sessionCache[pivotID]
	cacheMu.Unlock()
	if !ok {
		return false
	}
	sess.Close(ctx, graph)
	return true
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

func enrichHTTPWebsocketContext(r *http.Request) context.Context {
	ctx := r.Context()
	if logging.EntrypointFromContext(ctx) == "" {
		ctx = logging.WithEntrypoint(ctx, "websocket")
	}
	if logging.ClientIPFromContext(ctx) == "" {
		clientIP := logging.NormalizeClientIP(
			r.RemoteAddr,
			r.Header.Get("X-Redirected-For"),
			r.Header.Get("X-Forwarded-For"),
		)
		ctx = logging.WithClientIP(ctx, clientIP)
	}
	ctx, _, _ = logging.EnsureTrace(ctx)

	if u := auth.UserFromContext(ctx); u != nil {
		if cur := logging.ActorFromContext(ctx); cur.Kind == logging.ActorUnknown || cur.Kind == logging.ActorAnonymous {
			ctx = logging.WithActor(ctx, logging.Actor{
				Kind:    logging.ActorUser,
				ID:      strconv.Itoa(u.ID),
				Name:    u.Name,
				IsAdmin: u.IsAdmin,
			})
		}
	}
	return ctx
}

func bindPivotStreamAttrs(ctx context.Context, pivotID, portalID, beaconID, hostID int) context.Context {
	var attrs []slog.Attr
	if pivotID > 0 {
		attrs = append(attrs, slog.Int("pivot_id", pivotID))
	}
	if portalID > 0 {
		attrs = append(attrs, slog.Int("portal_id", portalID))
	}
	if beaconID > 0 {
		attrs = append(attrs, slog.Int("beacon_id", beaconID))
	}
	if hostID > 0 {
		attrs = append(attrs, slog.Int("host_id", hostID))
	}
	return logging.WithStreamAttrs(ctx, attrs...)
}

func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	ctx := enrichHTTPWebsocketContext(r)

	// Upgrade to websocket first to send errors over ws
	wsConn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.ErrorContext(ctx, "failed to upgrade pty websocket", "error", err)
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
	var initRows, initCols uint32
	if rStr := r.URL.Query().Get("rows"); rStr != "" {
		if rVal, err := strconv.ParseUint(rStr, 10, 32); err == nil {
			initRows = uint32(rVal)
		}
	}
	if cStr := r.URL.Query().Get("cols"); cStr != "" {
		if cVal, err := strconv.ParseUint(cStr, 10, 32); err == nil {
			initCols = uint32(cVal)
		}
	}

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
			pivotEnt, err := shell.RetryOnLock(ctx, func() (*ent.ShellPivot, error) {
				return h.graph.ShellPivot.Get(ctx, pivotID)
			})
			if err == nil {
				var portalID, beaconID, hostID int
				if pEnt, pErr := pivotEnt.QueryPortal().WithBeacon(func(bq *ent.BeaconQuery) { bq.WithHost() }).Only(ctx); pErr == nil && pEnt != nil {
					portalID = pEnt.ID
					if pEnt.Edges.Beacon != nil {
						beaconID = pEnt.Edges.Beacon.ID
						if pEnt.Edges.Beacon.Edges.Host != nil {
							hostID = pEnt.Edges.Beacon.Edges.Host.ID
						}
					}
				}
				ctx = bindPivotStreamAttrs(ctx, pivotID, portalID, beaconID, hostID)

				reconnectArgs := append(
					logging.TargetIntAttrs("pivot", pivotID, "pty"),
					slog.Int("pivot_id", pivotID),
				)
				if portalID > 0 {
					reconnectArgs = append(reconnectArgs, slog.Int("portal_id", portalID))
				}
				if beaconID > 0 {
					reconnectArgs = append(reconnectArgs, slog.Int("beacon_id", beaconID))
				}
				if hostID > 0 {
					reconnectArgs = append(reconnectArgs, slog.Int("host_id", hostID))
				}
				log.DebugContext(ctx, "serving persisted history for closed PTY pivot")
				logging.Audit(ctx, "portal", "portal.pty_pivot_reconnected", "operator reconnected to PTY pivot", reconnectArgs...)

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
			} else {
				log.WarnContext(ctx, "PTY pivot session not found for reconnect", "pivot_id", pivotID, "error", err)
				sendWsError("pivot session not found or expired")
			}
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
		ctx = bindPivotStreamAttrs(ctx, sess.PivotID, sess.PortalID, sess.BeaconID, sess.HostID)
		log.DebugContext(ctx, "operator reconnected to active PTY pivot")
		logging.Audit(ctx, "portal", "portal.pty_pivot_reconnected", "operator reconnected to PTY pivot", sess.auditArgs()...)
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
		p, err := shell.RetryOnLock(ctx, func() (*ent.Portal, error) {
			return h.graph.Portal.Query().
				Where(portal.ID(portalID)).
				WithBeacon(func(bq *ent.BeaconQuery) {
					bq.WithHost()
				}).
				Only(ctx)
		})
		if err != nil {
			log.WarnContext(ctx, "portal not found for PTY pivot", "portal_id", portalID, "error", err)
			sendWsError("portal not found")
			return
		}
		if !p.ClosedAt.IsZero() {
			log.WarnContext(ctx, "attempted to open PTY pivot on closed portal", "portal_id", portalID)
			sendWsError("portal is closed")
			return
		}

		var beaconID, hostID int
		if p.Edges.Beacon != nil {
			beaconID = p.Edges.Beacon.ID
			if p.Edges.Beacon.Edges.Host != nil {
				hostID = p.Edges.Beacon.Edges.Host.ID
			}
		}

		// Open Portal Mux
		cleanupPortal, err := h.mux.OpenPortal(ctx, portalID)
		if err != nil {
			log.ErrorContext(ctx, "failed to open portal for PTY pivot", "portal_id", portalID, "error", err)
			sendWsError(fmt.Sprintf("Failed to open portal: %v", err))
			return
		}

		topicIn := h.mux.TopicIn(portalID)
		topicOut := h.mux.TopicOut(portalID)

		// Stream ID
		streamID := fmt.Sprintf("pty-ws-%d", time.Now().UnixNano())

		// Create ShellPivot Ent
		pivotEnt, err := shell.RetryOnLock(ctx, func() (*ent.ShellPivot, error) {
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

			return createReq.Save(ctx)
		})
		if err != nil {
			log.ErrorContext(ctx, "failed to create PTY shell pivot", "portal_id", portalID, "error", err)
			sendWsError(fmt.Sprintf("Failed to create shell pivot: %v", err))
			cleanupPortal()
			return
		}

		ctx = bindPivotStreamAttrs(ctx, pivotEnt.ID, portalID, beaconID, hostID)

		// Setup portal communication
		detachedSessionBase := logging.DetachContext(ctx)
		sessionCtx, cancelSession := context.WithCancel(detachedSessionBase)

		recvCh, recvCleanup := h.mux.Subscribe(topicOut)

		pivotSession = &PivotSession{
			PivotID:    pivotEnt.ID,
			PortalID:   portalID,
			BeaconID:   beaconID,
			HostID:     hostID,
			StreamID:   streamID,
			SessionCtx: detachedSessionBase,
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
		closePivot := func(closeCtx context.Context) {
			closeOnce.Do(func() {
				pivotSession.Close(closeCtx, h.graph)
			})
		}

		// Background goroutine: read portal output -> broadcast to websockets.
		// Each mote is delivered exactly once: in single-server mode the
		// fast-path in Publish dispatches it and the receiveLoop is a no-op
		// (loopback detection). In multi-server mode the receiveLoop on the
		// websocket server receives it from GCP PubSub. No deduplication is
		// needed because loopback detection prevents any path from dispatching
		// the same mote twice.
		go func() {
			for {
				select {
				case <-sessionCtx.Done():
					return
				case m, ok := <-recvCh:
					if !ok {
						closePivot(detachedSessionBase)
						return
					}
					// Only accept motes matching our streamID
					if m.StreamId != streamID {
						continue
					}
					// Check for close
					if m.GetBytes() != nil && m.GetBytes().Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
						log.DebugContext(sessionCtx, "received PTY close mote from portal")
						closePivot(detachedSessionBase)
						return
					}
					// Handle PTY data
					if bytesPayload := m.GetBytes(); bytesPayload != nil && bytesPayload.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_PTY {
						log.TraceContext(sessionCtx, "forwarding PTY output frame to websocket", "bytes", len(bytesPayload.Data))
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
					pivotSession.FlushHistory(detachedSessionBase, h.graph)
				}
			}
		}()

		// Send initial PTY request mote to the agent to spawn the PTY.
		// Data is kept empty so agents that do not support PTYResize never write
		// control metadata into PTY stdin.
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
			log.ErrorContext(ctx, "failed to publish PTY init mote", "error", err)
			sendWsError(fmt.Sprintf("Failed to send PTY init mote: %v", err))
			cacheMu.Lock()
			delete(sessionCache, pivotEnt.ID)
			cacheMu.Unlock()
			pivotSession.CancelContext()
			return
		}

		log.DebugContext(ctx, "operator opened PTY pivot", "stream_id", streamID, "rows", initRows, "cols", initCols)
		logging.Audit(ctx, "portal", "portal.pty_pivot_opened", "operator opened PTY pivot", pivotSession.auditArgs()...)

		if initRows > 0 && initCols > 0 {
			initResizeMote := &portalpb.Mote{
				StreamId: streamID,
				SeqId:    0,
				Payload: &portalpb.Mote_PtyResize{
					PtyResize: &portalpb.PTYResize{
						Rows: initRows,
						Cols: initCols,
					},
				},
			}
			if err := h.mux.Publish(sessionCtx, topicIn, initResizeMote); err != nil {
				log.ErrorContext(sessionCtx, "failed to publish initial pty resize", "error", err)
			}
		}

		// WebSocket -> Portal PTY Input goroutine (spawned below in the common path)
		// needs access to topicIn and sessionCtx, so we wrap them in a closure
		startInputForwarder := func(wsConn *websocket.Conn, errCh chan error) {
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
					case "close":
						log.DebugContext(ctx, "received websocket PTY close control message")
						closeMote := &portalpb.Mote{
							StreamId: streamID,
							SeqId:    pivotSession.nextSeqID(),
							Payload: &portalpb.Mote_Bytes{
								Bytes: &portalpb.BytesPayload{
									Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE,
								},
							},
						}
						_ = h.mux.Publish(sessionCtx, topicIn, closeMote)
						closePivot(ctx)
						errCh <- fmt.Errorf("closed")
						return
					case "resize":
						if ctrlMsg.Rows > 0 && ctrlMsg.Cols > 0 {
							log.DebugContext(ctx, "forwarding websocket PTY window resize", "rows", ctrlMsg.Rows, "cols", ctrlMsg.Cols)
							resizeMote := &portalpb.Mote{
								StreamId: streamID,
								SeqId:    0,
								Payload: &portalpb.Mote_PtyResize{
									PtyResize: &portalpb.PTYResize{
										Rows: ctrlMsg.Rows,
										Cols: ctrlMsg.Cols,
									},
								},
							}
							if err := h.mux.Publish(sessionCtx, topicIn, resizeMote); err != nil {
								errCh <- err
								return
							}
						}
					case string(shell.WebsocketMessageKindInput):
						log.TraceContext(ctx, "forwarding websocket PTY input frame", "bytes", len(ctrlMsg.Input))
						mote := &portalpb.Mote{
							StreamId: streamID,
							SeqId:    pivotSession.nextSeqID(),
							Payload: &portalpb.Mote_Bytes{
								Bytes: &portalpb.BytesPayload{
									Data: []byte(ctrlMsg.Input),
									Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_PTY,
								},
							},
						}
						if err := h.mux.Publish(sessionCtx, topicIn, mote); err != nil {
							errCh <- err
							return
						}
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
				log.TraceContext(ctx, "writing PTY websocket frame")
				if err := wsConn.WriteJSON(msg); err != nil {
					errCh <- err
					return
				}
			}
			errCh <- fmt.Errorf("closed")
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
			log.TraceContext(ctx, "writing PTY websocket frame")
			if err := wsConn.WriteJSON(msg); err != nil {
				errCh <- err
				return
			}
		}
		errCh <- fmt.Errorf("closed")
	}()

	// Reader goroutine - for reconnects we use the stored portalID (or query once if unset).
	go func() {
		portalID := pivotSession.PortalID
		if portalID <= 0 {
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
			portalID = portalEnt.ID
		}
		topicIn := h.mux.TopicIn(portalID)

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
			case "close":
				log.DebugContext(ctx, "received websocket PTY close control message")
				closeMote := &portalpb.Mote{
					StreamId: pivotSession.StreamID,
					SeqId:    pivotSession.nextSeqID(),
					Payload: &portalpb.Mote_Bytes{
						Bytes: &portalpb.BytesPayload{
							Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE,
						},
					},
				}
				_ = h.mux.Publish(ctx, topicIn, closeMote)
				pivotSession.Close(ctx, h.graph)
				errCh <- fmt.Errorf("closed")
				return
			case "resize":
				if ctrlMsg.Rows > 0 && ctrlMsg.Cols > 0 {
					log.DebugContext(ctx, "forwarding websocket PTY window resize", "rows", ctrlMsg.Rows, "cols", ctrlMsg.Cols)
					resizeMote := &portalpb.Mote{
						StreamId: pivotSession.StreamID,
						SeqId:    0,
						Payload: &portalpb.Mote_PtyResize{
							PtyResize: &portalpb.PTYResize{
								Rows: ctrlMsg.Rows,
								Cols: ctrlMsg.Cols,
							},
						},
					}
					if err := h.mux.Publish(ctx, topicIn, resizeMote); err != nil {
						log.ErrorContext(ctx, "failed to publish pty resize", "error", err)
					}
				}
			case string(shell.WebsocketMessageKindInput):
				log.TraceContext(ctx, "forwarding websocket PTY input frame", "bytes", len(ctrlMsg.Input))
				mote := &portalpb.Mote{
					StreamId: pivotSession.StreamID,
					SeqId:    pivotSession.nextSeqID(),
					Payload: &portalpb.Mote_Bytes{
						Bytes: &portalpb.BytesPayload{
							Data: []byte(ctrlMsg.Input),
							Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_PTY,
						},
					},
				}
				if err := h.mux.Publish(ctx, topicIn, mote); err != nil {
					log.ErrorContext(ctx, "failed to publish pty input", "error", err)
				}
			}
		}
	}()

	// Wait for disconnect
	<-errCh
}
