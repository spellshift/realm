package portal

import (
	"context"
	"log/slog"
	"net/http"
	"sync"
	"time"

	"github.com/gorilla/websocket"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/portals"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/tracepb"
)

const (
	defaultMaxMessageSize  int64 = 10 * 1024 * 1024 // 10MB
	defaultKeepAliveInterval     = 30 * time.Second
	defaultWriteWaitTimeout      = 10 * time.Second
	defaultReadWaitTimeout       = 60 * time.Second
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		return true // Allow all origins for the websocket
	},
}

// A Handler for browser to server OpenPortal communication using websockets.
type Handler struct {
	graph *ent.Client
	mux   *mux.Mux

	maxMessageSize    int64
	keepAliveInterval time.Duration
	writeWaitTimeout  time.Duration
	readWaitTimeout   time.Duration
}

// NewHandler initializes and returns a new handler using the provided ent client and portal mux.
func NewHandler(graph *ent.Client, mux *mux.Mux) *Handler {
	return &Handler{
		graph: graph,
		mux:   mux,

		maxMessageSize:    defaultMaxMessageSize,
		keepAliveInterval: defaultKeepAliveInterval,
		writeWaitTimeout:  defaultWriteWaitTimeout,
		readWaitTimeout:   defaultReadWaitTimeout,
	}
}

func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	// Load Authenticated User
	authUser := auth.UserFromContext(r.Context())
	var (
		authUserName = "unknown"
		authUserID   = 0
	)
	if authUser != nil {
		authUserID = authUser.ID
		authUserName = authUser.Name
	}
	slog.InfoContext(r.Context(), "starting new portal websocket session", "user_id", authUserID, "user_name", authUserName)

	// Upgrade to websocket
	wsConn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		slog.ErrorContext(r.Context(), "websocket failed to upgrade connection", "err", err)
		http.Error(w, "failed to upgrade to websocket", http.StatusFailedDependency)
		return
	}
	defer slog.InfoContext(r.Context(), "websocket portal connection closed", "user_id", authUserID, "user_name", authUserName)

	// Configure Websocket Connection
	wsConn.SetReadLimit(h.maxMessageSize)
	wsConn.SetReadDeadline(time.Now().Add(h.readWaitTimeout))
	wsConn.SetPongHandler(func(string) error {
		wsConn.SetReadDeadline(time.Now().Add(h.readWaitTimeout))
		return nil
	})

	ctx, cancel := context.WithCancel(r.Context())
	defer cancel()

	// Wait for the first message to be the OpenPortalRequest (Registration)
	msgType, msgData, err := wsConn.ReadMessage()
	if err != nil {
		slog.ErrorContext(ctx, "failed to read registration message from websocket", "err", err)
		return
	}

	var registerMsg portalpb.OpenPortalRequest
	if msgType == websocket.BinaryMessage {
		if err := proto.Unmarshal(msgData, &registerMsg); err != nil {
			slog.ErrorContext(ctx, "failed to unmarshal binary OpenPortalRequest", "err", err)
			return
		}
	} else if msgType == websocket.TextMessage {
		if err := protojson.Unmarshal(msgData, &registerMsg); err != nil {
			slog.ErrorContext(ctx, "failed to unmarshal JSON OpenPortalRequest", "err", err)
			return
		}
	} else {
		slog.ErrorContext(ctx, "invalid message type for registration", "type", msgType)
		return
	}

	portalID := int(registerMsg.GetPortalId())
	if portalID <= 0 {
		slog.ErrorContext(ctx, "invalid portal ID in registration", "portal_id", portalID)
		return
	}

	p, err := h.graph.Portal.Get(ctx, portalID)
	if err != nil {
		slog.ErrorContext(ctx, "error loading portal ID", "portal_id", portalID, "err", err)
		return
	}
	if !p.ClosedAt.IsZero() {
		slog.ErrorContext(ctx, "portal is closed", "portal_id", portalID)
		return
	}

	// Open portal through mux
	cleanup, err := h.mux.OpenPortal(ctx, portalID)
	if err != nil {
		slog.ErrorContext(ctx, "failed to open portal", "portal_id", portalID, "error", err)
		return
	}
	defer cleanup()

	portalOutTopic := h.mux.TopicOut(portalID)
	recv, cleanupSub := h.mux.Subscribe(portalOutTopic)
	defer cleanupSub()

	done := make(chan struct{}, 2)
	var wg sync.WaitGroup

	// Start goroutine to subscribe to portal output and send to websocket stream
	wg.Add(1)
	go func() {
		defer wg.Done()
		h.sendPortalOutput(ctx, portalID, wsConn, recv)
		done <- struct{}{}
		cancel() // Cancel the context to stop the other goroutine
	}()

	// Send portal input from websocket stream to portal input topic
	wg.Add(1)
	go func() {
		defer wg.Done()
		h.sendPortalInput(ctx, portalID, wsConn, h.mux)
		done <- struct{}{}
		cancel() // Cancel the context to stop the other goroutine
	}()

	select {
	case <-ctx.Done():
		break
	case <-done:
		break
	}

	// Ensure we wait for both goroutines to finish
	wg.Wait()
}

func (h *Handler) sendPortalInput(ctx context.Context, portalID int, wsConn *websocket.Conn, mux *mux.Mux) {
	portalInTopic := mux.TopicIn(portalID)

	for {
		select {
		case <-ctx.Done():
			return
		default:
		}

		msgType, msgData, err := wsConn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				slog.ErrorContext(ctx, "portal websocket closed unexpectedly", "error", err, "portal_id", portalID)
			} else {
				slog.InfoContext(ctx, "portal websocket stream ended", "portal_id", portalID, "err", err)
			}
			return
		}

		var req portalpb.OpenPortalRequest
		if msgType == websocket.BinaryMessage {
			if err := proto.Unmarshal(msgData, &req); err != nil {
				slog.ErrorContext(ctx, "failed to unmarshal binary OpenPortalRequest in input loop", "err", err, "portal_id", portalID)
				continue
			}
		} else if msgType == websocket.TextMessage {
			if err := protojson.Unmarshal(msgData, &req); err != nil {
				slog.ErrorContext(ctx, "failed to unmarshal JSON OpenPortalRequest in input loop", "err", err, "portal_id", portalID)
				continue
			}
		} else {
			continue // Skip other types (e.g. Ping/Pong)
		}

		mote := req.GetMote()
		if mote == nil {
			continue
		}

		// Skip keepalive motes
		if bytesMote := mote.GetBytes(); bytesMote != nil && bytesMote.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_KEEPALIVE {
			continue
		}

		// TRACE: Server User Recv
		if err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_RECV); err != nil {
			slog.ErrorContext(ctx, "failed to add trace event (Server User Recv)", "error", err)
		}

		// TRACE: Server User Pub
		if err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_PUB); err != nil {
			slog.ErrorContext(ctx, "failed to add trace event (Server User Pub)", "error", err)
		}

		// Publish to portal input topic
		if err := mux.Publish(ctx, portalInTopic, mote); err != nil {
			slog.ErrorContext(ctx, "failed to publish mote to portal input topic",
				"portal_id", portalID,
				"error", err,
			)
		}
	}
}

func (h *Handler) sendPortalOutput(ctx context.Context, portalID int, wsConn *websocket.Conn, recv <-chan *portalpb.Mote) {
	// Keep Alives
	keepAliveTimer := time.NewTicker(h.keepAliveInterval)
	defer keepAliveTimer.Stop()

	for {
		select {
		case <-ctx.Done():
			slog.InfoContext(ctx, "portal output loop context done", "portal_id", portalID, "error", ctx.Err())
			return
		case <-keepAliveTimer.C:
			wsConn.SetWriteDeadline(time.Now().Add(h.writeWaitTimeout))
			if err := wsConn.WriteMessage(websocket.PingMessage, nil); err != nil {
				slog.ErrorContext(ctx, "failed to send ping message to websocket", "error", err)
			}
		case mote, ok := <-recv:
			if !ok {
				slog.InfoContext(ctx, "portal output channel closed", "portal_id", portalID)
				// Send a close message to the websocket client
				wsConn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}

			// TRACE: Server User Sub
			if err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_SUB); err != nil {
				slog.ErrorContext(ctx, "failed to add trace event (Server User Sub)", "error", err)
			}

			// TRACE: Server User Send
			if err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_SEND); err != nil {
				slog.ErrorContext(ctx, "failed to add trace event (Server User Send)", "error", err)
			}

			resp := &portalpb.OpenPortalResponse{
				Mote: mote,
			}

			// We need to marshal it to send over WebSocket
			// Let's use protojson to be browser friendly by default (or binary, but text is easier for some things).
			// We can use TextMessage for this instance to match typical JS usage.
			opts := protojson.MarshalOptions{
				EmitUnpopulated: true,
			}
			msgData, err := opts.Marshal(resp)
			if err != nil {
				slog.ErrorContext(ctx, "failed to marshal OpenPortalResponse", "error", err)
				continue
			}

			wsConn.SetWriteDeadline(time.Now().Add(h.writeWaitTimeout))
			if err := wsConn.WriteMessage(websocket.TextMessage, msgData); err != nil {
				slog.ErrorContext(ctx, "failed to send portal output message to websocket", "portal_id", portalID, "error", err)
				return // Client disconnected or error writing
			}

			if payload := mote.GetBytes(); payload != nil && payload.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
				slog.InfoContext(ctx, "received portal close, disconnecting client", "portal_id", portalID, "reason", string(payload.Data))
				wsConn.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseNormalClosure, "portal closed"))
				return
			}
		}
	}
}
