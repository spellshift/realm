package stream

import (
	"context"
	"log/slog"
	"net/http"
	"strconv"
	"sync"
	"time"

	"github.com/gorilla/websocket"
	"gocloud.dev/pubsub"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/shell"
	"realm.pub/tavern/internal/ent/user"
	"realm.pub/tavern/internal/logging"
)

const (
	// Time allowed to write a message to the peer.
	writeWait = 10 * time.Second

	// Time allowed to read the next pong message from the peer.
	pongWait = 60 * time.Second

	// Send pings to peer with this period. Must be less than pongWait.
	pingPeriod = (pongWait * 9) / 10

	// Maximum message size allowed from peer.
	maxMessageSize = 256 * 1024 // 256KB
)

type connector struct {
	*Stream
	mux  *Mux
	ws   *websocket.Conn
	kind string
}

// WriteToWebsocket will read messages from the Mux and write them to the underlying websocket.
func (c *connector) WriteToWebsocket(ctx context.Context) {
	defer c.ws.Close()

	// Register with mux to receive messages
	c.mux.Register(c.Stream)
	defer c.mux.Unregister(c.Stream)

	// Keep Alive
	ticker := time.NewTicker(pingPeriod)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			c.ws.WriteMessage(websocket.CloseMessage, []byte{})
			return
		case message, ok := <-c.Messages():
			c.ws.SetWriteDeadline(time.Now().Add(writeWait))
			if !ok {
				// The mux closed the channel.
				c.ws.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}

			// Check if stream has closed
			hasClosed, ok := message.Metadata[MetadataStreamClose]
			if ok && hasClosed != "" {
				// The producer ended the stream.
				muxLog.DebugContext(ctx, "websocket closed due to producer ending stream",
					"stream_id", c.Stream.id,
					"stream_order_key", c.Stream.orderKey,
				)
				c.ws.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}

			// Filter by kind
			kind := message.Metadata[MetadataMsgKind]
			if kind == "" {
				kind = "data"
			}
			if kind != c.kind {
				continue
			}

			muxLog.TraceContext(ctx, "writing stream message to websocket",
				"stream_id", c.Stream.id,
				"kind", kind,
				"bytes", len(message.Body),
			)
			w, err := c.ws.NextWriter(websocket.BinaryMessage)
			if err != nil {
				return
			}
			if _, err := w.Write(message.Body); err != nil {
				muxLog.ErrorContext(ctx, "failed to write message from producer to websocket",
					"stream_id", c.Stream.id,
					"stream_order_key", c.Stream.orderKey,
					"error", err,
				)
			}

			// Flush queued messages to the current websocket message.
			n := len(c.Messages())
			for i := 0; i < n; i++ {
				additionalMsg := <-c.Messages()

				// Filter additional messages too
				kind := additionalMsg.Metadata[MetadataMsgKind]
				if kind == "" {
					kind = "data"
				}
				if kind != c.kind {
					continue
				}

				if _, err := w.Write(additionalMsg.Body); err != nil {
					muxLog.ErrorContext(ctx, "failed to write additional message from producer to websocket",
						"stream_id", c.Stream.id,
						"stream_order_key", c.Stream.orderKey,
						"error", err,
					)
				}
			}

			if err := w.Close(); err != nil {
				return
			}
		case <-ticker.C:
			muxLog.TraceContext(ctx, "sending websocket keepalive ping", "stream_id", c.Stream.id)
			c.ws.SetWriteDeadline(time.Now().Add(writeWait))
			if err := c.ws.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}

// ReadFromWebsocket will read messages from the underlying websocket and send them to the configured Mux.
func (c *connector) ReadFromWebsocket(ctx context.Context) {
	defer c.ws.Close()

	// Configure connection info
	c.ws.SetReadLimit(maxMessageSize)
	c.ws.SetReadDeadline(time.Now().Add(pongWait))
	c.ws.SetPongHandler(func(string) error {
		c.ws.SetReadDeadline(time.Now().Add(pongWait))
		return nil
	})

	for {
		select {
		case <-ctx.Done():
			return
		default:
			_, message, err := c.ws.ReadMessage()
			if err != nil {
				if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
					muxLog.ErrorContext(ctx, "websocket closed unexpectedly",
						"stream_id", c.Stream.id,
						"stream_order_key", c.Stream.orderKey,
						"error", err,
					)
				}
				return
			}

			msgLen := len(message)
			muxLog.TraceContext(ctx, "publishing websocket frame to stream mux",
				"stream_id", c.Stream.id,
				"kind", c.kind,
				"bytes", msgLen,
			)
			if err := c.Stream.SendMessage(ctx, &pubsub.Message{
				Body: message,
				Metadata: map[string]string{
					metadataID:      c.id,
					MetadataMsgKind: c.kind,
				},
			}, c.mux); err != nil {
				muxLog.ErrorContext(ctx, "websocket failed to publish message",
					"stream_id", c.Stream.id,
					"stream_order_key", c.Stream.orderKey,
					"msg_len", msgLen,
					"error", err,
				)
				return
			}
		}
	}
}

func manageActiveUser(ctx context.Context, done <-chan struct{}, graph *ent.Client, shellID int, userID int) {
	defer func() {
		cleanupCtx := logging.DetachContext(ctx)
		muxLog.DebugContext(cleanupCtx, "websocket checking user activity for shell before removal", "user_id", userID, "shell_id", shellID)

		wasAdded, err := graph.Shell.Query().
			Where(shell.ID(shellID)).
			QueryActiveUsers().
			Where(user.ID(userID)).
			Exist(cleanupCtx)
		if err != nil {
			muxLog.ErrorContext(cleanupCtx, "websocket failed to check user activity for shell", "error", err, "user_id", userID, "shell_id", shellID)
			return
		}
		if !wasAdded {
			return
		}

		if _, err := graph.Shell.UpdateOneID(shellID).
			RemoveActiveUserIDs(userID).
			Save(cleanupCtx); err != nil {
			muxLog.ErrorContext(cleanupCtx, "websocket failed to remove inactive user from shell", "error", err, "user_id", userID, "shell_id", shellID)
			return
		}
	}()

	ticker := time.NewTicker(5 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
		case <-done:
			return
		case <-ticker.C:
			// Handle case where user has multiple shells open
			alreadyAdded, err := graph.Shell.Query().
				Where(shell.ID(shellID)).
				QueryActiveUsers().
				Where(user.ID(userID)).
				Exist(ctx)
			if err != nil {
				muxLog.ErrorContext(ctx, "websocket failed to check user activity for shell", "error", err, "user_id", userID, "shell_id", shellID)
				continue
			}
			if alreadyAdded {
				continue
			}

			if _, err := graph.Shell.UpdateOneID(shellID).
				AddActiveUserIDs(userID).
				Save(ctx); err != nil {
				muxLog.ErrorContext(ctx, "websocket failed to add active user to shell", "error", err, "user_id", userID, "shell_id", shellID)
			}
		}
	}
}

func enrichStreamRequestContext(r *http.Request) (context.Context, *ent.User) {
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

	authUser := auth.UserFromContext(ctx)
	if authUser != nil {
		if cur := logging.ActorFromContext(ctx); cur.Kind == logging.ActorUnknown || cur.Kind == logging.ActorAnonymous {
			ctx = logging.WithActor(ctx, logging.Actor{
				Kind:    logging.ActorUser,
				ID:      strconv.Itoa(authUser.ID),
				Name:    authUser.Name,
				IsAdmin: authUser.IsAdmin,
			})
		}
	}
	return ctx, authUser
}

// NewShellHandler provides an HTTP handler which handles a websocket for shell io.
// It requires a query param "shell_id" be specified (must be an integer).
// This ID represents which Shell ent the websocket will connect to.
func NewShellHandler(graph *ent.Client, mux *Mux) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ctx, authUser := enrichStreamRequestContext(r)

		// Parse Shell ID
		shellIDStr := r.URL.Query().Get("shell_id")
		if shellIDStr == "" {
			http.Error(w, "must provide integer value for 'shell_id'", http.StatusBadRequest)
			return
		}
		shellID, err := strconv.Atoi(shellIDStr)
		if err != nil {
			http.Error(w, "invalid 'shell_id' provided, must be integer", http.StatusBadRequest)
			return
		}

		// Load Shell
		revShell, err := graph.Shell.Query().
			Where(shell.ID(shellID)).
			WithBeacon(func(bq *ent.BeaconQuery) {
				bq.WithHost()
			}).
			Select(shell.FieldClosedAt).
			Only(ctx)
		if err != nil {
			if ent.IsNotFound(err) {
				http.Error(w, "shell not found", http.StatusNotFound)
			} else {
				muxLog.ErrorContext(ctx, "websocket failed to load shell", "error", err, "shell_id", shellID)
				http.Error(w, "failed to load shell", http.StatusInternalServerError)
			}
			return
		}

		var beaconID, hostID int
		if revShell.Edges.Beacon != nil {
			beaconID = revShell.Edges.Beacon.ID
			if revShell.Edges.Beacon.Edges.Host != nil {
				hostID = revShell.Edges.Beacon.Edges.Host.ID
			}
		}
		streamAttrs := []slog.Attr{slog.Int("shell_id", shellID)}
		if beaconID > 0 {
			streamAttrs = append(streamAttrs, slog.Int("beacon_id", beaconID))
		}
		if hostID > 0 {
			streamAttrs = append(streamAttrs, slog.Int("host_id", hostID))
		}
		ctx = logging.WithStreamAttrs(ctx, streamAttrs...)

		// Prevent opening closed shells
		if !revShell.ClosedAt.IsZero() {
			http.Error(w, "shell already closed", http.StatusBadRequest)
			return
		}

		// Start Websocket
		muxLog.DebugContext(ctx, "new shell websocket connection")
		ws, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			muxLog.ErrorContext(ctx, "websocket failed to upgrade connection", "error", err)
			return
		}

		shellAuditArgs := append(
			logging.TargetIntAttrs("shell", shellID, ""),
			logging.ShellAttrs(shellID, beaconID, hostID)...,
		)
		logging.Audit(ctx, "shell", "shell.ws_connected", "operator connected to shell websocket", shellAuditArgs...)
		defer func() {
			disconnectCtx := logging.DetachContext(ctx)
			muxLog.DebugContext(disconnectCtx, "websocket shell connection closed")
			logging.Audit(disconnectCtx, "shell", "shell.ws_disconnected", "operator disconnected from shell websocket", shellAuditArgs...)
		}()

		// Track Active User
		var activeUserWG sync.WaitGroup
		activeUserDoneCh := make(chan struct{})
		if authUser != nil {
			activeUserWG.Add(1)
			go func(ctx context.Context, shellID, userID int) {
				defer activeUserWG.Done()
				manageActiveUser(ctx, activeUserDoneCh, graph, shellID, userID)
			}(ctx, revShell.ID, authUser.ID)
		}

		// Initialize Stream
		stream := New(shellIDStr)

		// Create Connector
		conn := &connector{
			Stream: stream,
			mux:    mux,
			ws:     ws,
			kind:   "data",
		}

		// Read & Write
		var wg sync.WaitGroup
		wg.Add(2)
		go func() {
			defer wg.Done()
			conn.ReadFromWebsocket(ctx)
		}()
		go func() {
			defer wg.Done()
			conn.WriteToWebsocket(ctx)
		}()

		wg.Wait()
		activeUserDoneCh <- struct{}{}
		activeUserWG.Wait()
	})
}

// NewPingHandler provides an HTTP handler which handles a websocket for latency pings.
// It requires a query param "shell_id" be specified (must be an integer).
func NewPingHandler(graph *ent.Client, mux *Mux) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ctx, _ := enrichStreamRequestContext(r)

		// Parse Shell ID
		shellIDStr := r.URL.Query().Get("shell_id")
		if shellIDStr == "" {
			http.Error(w, "must provide integer value for 'shell_id'", http.StatusBadRequest)
			return
		}
		shellID, err := strconv.Atoi(shellIDStr)
		if err != nil {
			http.Error(w, "invalid 'shell_id' provided, must be integer", http.StatusBadRequest)
			return
		}
		ctx = logging.WithStreamAttrs(ctx, slog.Int("shell_id", shellID))

		// Check if shell exists (optional, but good for consistency)
		exists, err := graph.Shell.Query().Where(shell.ID(shellID)).Exist(ctx)
		if err != nil || !exists {
			http.Error(w, "shell not found", http.StatusNotFound)
			return
		}

		// Start Websocket
		ws, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			return
		}

		// Initialize Stream
		stream := New(shellIDStr)

		// Create Connector
		conn := &connector{
			Stream: stream,
			mux:    mux,
			ws:     ws,
			kind:   "ping",
		}

		// Read & Write
		var wg sync.WaitGroup
		wg.Add(2)
		go func() {
			defer wg.Done()
			conn.ReadFromWebsocket(ctx)
		}()
		go func() {
			defer wg.Done()
			conn.WriteToWebsocket(ctx)
		}()

		wg.Wait()
	})
}
