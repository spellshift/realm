package stream

import (
	"context"
	"log/slog"
	"net/http"
	"strconv"
	"sync"
	"time"

	"github.com/gorilla/websocket"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/portal"
	"realm.pub/tavern/internal/ent/user"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
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

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin: func(r *http.Request) bool {
		return true // Allow all origins for now (dev/internal)
	},
}

type connector struct {
	mux      *mux.Mux
	portalID int
	ws       *websocket.Conn
}

// WriteToWebsocket will read messages from the Mux and write them to the underlying websocket.
func (c *connector) WriteToWebsocket(ctx context.Context) {
	defer c.ws.Close()

	topicOut := c.mux.TopicOut(c.portalID)
	recv, cleanup := c.mux.Subscribe(topicOut)
	defer cleanup()

	ticker := time.NewTicker(pingPeriod)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			c.ws.WriteMessage(websocket.CloseMessage, []byte{})
			return
		case <-ticker.C:
			c.ws.SetWriteDeadline(time.Now().Add(writeWait))
			if err := c.ws.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		case mote, ok := <-recv:
			c.ws.SetWriteDeadline(time.Now().Add(writeWait))
			if !ok {
				c.ws.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}

			// Extract Payload
			if mote.GetRepl() != nil {
				// REPL Data -> Text/Binary Message
				w, err := c.ws.NextWriter(websocket.BinaryMessage)
				if err != nil {
					return
				}
				if _, err := w.Write(mote.GetRepl().Data); err != nil {
					slog.ErrorContext(ctx, "failed to write repl message to websocket", "error", err)
				}
				if err := w.Close(); err != nil {
					return
				}
			} else if mote.GetBytes() != nil {
				// Handle BytesPayload (e.g. Ping/Keepalive)
				// Currently we just ignore it as we send our own pings
			}
		}
	}
}

// ReadFromWebsocket will read messages from the underlying websocket and send them to the configured Mux.
func (c *connector) ReadFromWebsocket(ctx context.Context) {
	defer c.ws.Close()

	c.ws.SetReadLimit(maxMessageSize)
	c.ws.SetReadDeadline(time.Now().Add(pongWait))
	c.ws.SetPongHandler(func(string) error {
		c.ws.SetReadDeadline(time.Now().Add(pongWait))
		return nil
	})

	topicIn := c.mux.TopicIn(c.portalID)

	for {
		select {
		case <-ctx.Done():
			return
		default:
			_, message, err := c.ws.ReadMessage()
			if err != nil {
				if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
					slog.ErrorContext(ctx, "websocket closed unexpectedly", "error", err)
				}
				return
			}

			// Wrap in Mote
			mote := &portalpb.Mote{
				StreamId: "repl", // Default stream ID for WS
				Payload: &portalpb.Mote_Repl{
					Repl: &portalpb.REPLMessage{
						Data: message,
					},
				},
			}

			if err := c.mux.Publish(ctx, topicIn, mote); err != nil {
				slog.ErrorContext(ctx, "websocket failed to publish message", "error", err)
				return
			}
		}
	}
}

func manageActiveUser(ctx context.Context, done <-chan struct{}, graph *ent.Client, portalID int, userID int) {
	defer func() {
		slog.DebugContext(ctx, "websocket checking user activity for portal before removal", "user_id", userID, "portal_id", portalID)

		wasAdded, err := graph.Portal.Query().
			Where(portal.ID(portalID)).
			QueryActiveUsers().
			Where(user.ID(userID)).
			Exist(ctx)
		if err != nil {
			slog.ErrorContext(ctx, "websocket failed to check user activity for portal", "err", err, "user_id", userID, "portal_id", portalID)
			return
		}
		if !wasAdded {
			return
		}

		if _, err := graph.Portal.UpdateOneID(portalID).
			RemoveActiveUserIDs(userID).
			Save(ctx); err != nil {
			slog.ErrorContext(ctx, "websocket failed to remove inactive user from portal", "err", err, "user_id", userID, "portal_id", portalID)
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
			alreadyAdded, err := graph.Portal.Query().
				Where(portal.ID(portalID)).
				QueryActiveUsers().
				Where(user.ID(userID)).
				Exist(ctx)
			if err != nil {
				slog.ErrorContext(ctx, "websocket failed to check user activity for portal", "err", err, "user_id", userID, "portal_id", portalID)
				continue
			}
			if alreadyAdded {
				continue
			}

			if _, err := graph.Portal.UpdateOneID(portalID).
				AddActiveUserIDs(userID).
				Save(ctx); err != nil {
				slog.ErrorContext(ctx, "websocket failed to add active user to portal", "err", err, "user_id", userID, "portal_id", portalID)
			}
		}
	}
}

// NewShellHandler provides an HTTP handler which handles a websocket for shell io.
// It requires a query param "shell_id" be specified (must be an integer).
// This ID represents which Portal ent the websocket will connect to (mapped from shell_id).
func NewShellHandler(graph *ent.Client, mux *mux.Mux) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		// Load Authenticated User
		authUser := auth.UserFromContext(ctx)
		var (
			authUserName = "unknown"
			authUserID   = 0
		)
		if authUser != nil {
			authUserID = authUser.ID
			authUserName = authUser.Name
		}

		// Parse Shell ID (Portal ID)
		shellIDStr := r.URL.Query().Get("shell_id")
		if shellIDStr == "" {
			http.Error(w, "must provide integer value for 'shell_id'", http.StatusBadRequest)
			return
		}
		portalID, err := strconv.Atoi(shellIDStr)
		if err != nil {
			http.Error(w, "invalid 'shell_id' provided, must be integer", http.StatusBadRequest)
			return
		}

		// Load Portal
		p, err := graph.Portal.Query().
			Where(portal.ID(portalID)).
			Select(portal.FieldClosedAt).
			Only(ctx)
		if err != nil {
			if ent.IsNotFound(err) {
				http.Error(w, "portal not found", http.StatusNotFound)
			} else {
				slog.ErrorContext(ctx, "websocket failed to load portal", "err", err, "portal_id", portalID, "user_id", authUserID, "user_name", authUserName)
				http.Error(w, "failed to load portal", http.StatusInternalServerError)
			}
			return
		}

		// Track Active User
		var activeUserWG sync.WaitGroup
		activeUserDoneCh := make(chan struct{})
		if authUser != nil {
			activeUserWG.Add(1)
			go func(ctx context.Context, portalID, userID int) {
				defer activeUserWG.Done()
				manageActiveUser(ctx, activeUserDoneCh, graph, portalID, userID)
			}(ctx, p.ID, authUser.ID)
		}

		// Prevent opening closed portals
		if !p.ClosedAt.IsZero() {
			http.Error(w, "portal already closed", http.StatusBadRequest)
			return
		}

		// Start Websocket
		slog.InfoContext(ctx, "new portal websocket connection", "portal_id", portalID, "user_id", authUserID, "user_name", authUserName)
		ws, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			slog.ErrorContext(ctx, "websocket failed to upgrade connection", "err", err, "portal_id", portalID, "user_id", authUserID, "user_name", authUserName)
			return
		}
		defer slog.InfoContext(ctx, "websocket portal connection closed", "portal_id", portalID, "user_id", authUserID, "user_name", authUserName)

		// Create Connector
		conn := &connector{
			mux:      mux,
			portalID: portalID,
			ws:       ws,
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

// NewPingHandler is a no-op handler to satisfy legacy routes if needed,
// or we can remove it. For now, let's keep it but just close.
func NewPingHandler(graph *ent.Client, mux *mux.Mux) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "not implemented", http.StatusNotImplemented)
	})
}
