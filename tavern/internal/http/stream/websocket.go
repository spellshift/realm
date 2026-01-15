package stream

import (
	"context"
	"errors"
	"io"
	"log/slog"
	"net/http"
	"strconv"
	"sync"
	"time"

	"github.com/gorilla/websocket"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/shell"
	"realm.pub/tavern/internal/ent/user"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
	portalstream "realm.pub/tavern/portals/stream"
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
	CheckOrigin:     func(r *http.Request) bool { return true },
}

// NewShellHandler provides an HTTP handler which handles a websocket for shell io.
// It requires a query param "portal_id" be specified (must be an integer).
func NewShellHandler(graph *ent.Client, portalMux *mux.Mux) http.HandlerFunc {
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

		// Parse Portal ID
		portalIDStr := r.URL.Query().Get("portal_id")
		if portalIDStr == "" {
			// Fallback for backward compatibility or ease of use
			portalIDStr = r.URL.Query().Get("shell_id")
		}
		if portalIDStr == "" {
			http.Error(w, "must provide integer value for 'portal_id'", http.StatusBadRequest)
			return
		}
		portalID, err := strconv.ParseInt(portalIDStr, 10, 64)
		if err != nil {
			http.Error(w, "invalid 'portal_id' provided, must be integer", http.StatusBadRequest)
			return
		}

		// Track Active User (Legacy behavior, assuming portalID maps to shellID)
		var activeUserWG sync.WaitGroup
		activeUserDoneCh := make(chan struct{})
		if authUser != nil {
			activeUserWG.Add(1)
			go func(ctx context.Context, shellID, userID int) {
				defer activeUserWG.Done()
				manageActiveUser(ctx, activeUserDoneCh, graph, shellID, userID)
			}(ctx, int(portalID), authUser.ID)
		}

		// Start Websocket
		slog.InfoContext(ctx, "new shell websocket connection", "portal_id", portalID, "user_id", authUserID, "user_name", authUserName)
		ws, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			slog.ErrorContext(ctx, "websocket failed to upgrade connection", "err", err, "portal_id", portalID, "user_id", authUserID, "user_name", authUserName)
			return
		}
		defer slog.InfoContext(ctx, "websocket shell connection closed", "portal_id", portalID, "user_id", authUserID, "user_name", authUserName)

		// Open Portal Subscription
		teardown, err := portalMux.OpenPortal(ctx, int(portalID))
		if err != nil {
			slog.ErrorContext(ctx, "failed to open portal", "err", err, "portal_id", portalID)
			ws.Close()
			return
		}
		defer teardown()

		// Subscribe to outgoing messages (from Agent)
		rxChan, rxCancel := portalMux.Subscribe(portalMux.TopicOut(int(portalID)), mux.WithHistoryReplay())
		defer rxCancel()

		// Setup Ordered Reader (Agent -> User)
		reader := portalstream.NewOrderedReader(func() (*portalpb.Mote, error) {
			select {
			case <-ctx.Done():
				return nil, ctx.Err()
			case m, ok := <-rxChan:
				if !ok {
					return nil, io.EOF
				}
				return m, nil
			}
		})

		// Setup Ordered Writer (User -> Agent)
		writer := portalstream.NewOrderedWriter(
			"ws_user", // We don't have a unique stream ID for the user here, maybe uuid? For now static is fine if we don't need ordering across multiple user tabs.
			func(m *portalpb.Mote) error {
				return portalMux.Publish(ctx, portalMux.TopicIn(int(portalID)), m)
			},
		)

		// Read & Write Loops
		var wg sync.WaitGroup
		wg.Add(2)

		// Read from Websocket (User) -> Write to Portal (Agent)
		go func() {
			defer wg.Done()
			defer ws.Close()

			ws.SetReadLimit(maxMessageSize)
			ws.SetReadDeadline(time.Now().Add(pongWait))
			ws.SetPongHandler(func(string) error {
				ws.SetReadDeadline(time.Now().Add(pongWait))
				return nil
			})

			for {
				select {
				case <-ctx.Done():
					return
				default:
					_, message, err := ws.ReadMessage()
					if err != nil {
						if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
							slog.ErrorContext(ctx, "websocket closed unexpectedly", "portal_id", portalID, "error", err)
						}
						return
					}

					if err := writer.WriteShell(message); err != nil {
						slog.ErrorContext(ctx, "failed to publish shell message", "portal_id", portalID, "error", err)
						return
					}
				}
			}
		}()

		// Read from Portal (Agent) -> Write to Websocket (User)
		go func() {
			defer wg.Done()
			defer ws.Close()

			moteChan := make(chan *portalpb.Mote)
			errChan := make(chan error)

			go func() {
				for {
					m, err := reader.Read()
					if err != nil {
						errChan <- err
						return
					}
					moteChan <- m
				}
			}()

			ticker := time.NewTicker(pingPeriod)
			defer ticker.Stop()

			for {
				select {
				case <-ctx.Done():
					ws.WriteMessage(websocket.CloseMessage, []byte{})
					return
				case err := <-errChan:
					if !errors.Is(err, io.EOF) {
						slog.ErrorContext(ctx, "error reading from portal stream", "error", err)
					}
					ws.WriteMessage(websocket.CloseMessage, []byte{})
					return
				case mote := <-moteChan:
					// Handle Mote
					var data []byte

					switch p := mote.Payload.(type) {
					case *portalpb.Mote_Shell:
						data = p.Shell.Data
					case *portalpb.Mote_Bytes:
						if p.Bytes.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA {
							data = p.Bytes.Data
						}
					case *portalpb.Mote_Repl:
						data = p.Repl.Data
					}

					if len(data) > 0 {
						ws.SetWriteDeadline(time.Now().Add(writeWait))
						w, err := ws.NextWriter(websocket.BinaryMessage)
						if err != nil {
							return
						}
						if _, err := w.Write(data); err != nil {
							return
						}
						if err := w.Close(); err != nil {
							return
						}
					}
				case <-ticker.C:
					ws.SetWriteDeadline(time.Now().Add(writeWait))
					if err := ws.WriteMessage(websocket.PingMessage, nil); err != nil {
						return
					}
				}
			}
		}()

		wg.Wait()
		activeUserDoneCh <- struct{}{}
		activeUserWG.Wait()
	})
}

// NewPingHandler provides an HTTP handler which handles a websocket for latency pings.
func NewPingHandler(graph *ent.Client, portalMux *mux.Mux) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		portalIDStr := r.URL.Query().Get("portal_id")
		if portalIDStr == "" {
			portalIDStr = r.URL.Query().Get("shell_id")
		}
		if portalIDStr == "" {
			http.Error(w, "must provide integer value for 'portal_id'", http.StatusBadRequest)
			return
		}
		portalID, err := strconv.ParseInt(portalIDStr, 10, 64)
		if err != nil {
			http.Error(w, "invalid 'portal_id' provided, must be integer", http.StatusBadRequest)
			return
		}

		ws, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			return
		}

		teardown, err := portalMux.OpenPortal(ctx, int(portalID))
		if err != nil {
			ws.Close()
			return
		}
		defer teardown()

		rxChan, rxCancel := portalMux.Subscribe(portalMux.TopicOut(int(portalID)), mux.WithHistoryReplay())
		defer rxCancel()

		reader := portalstream.NewOrderedReader(func() (*portalpb.Mote, error) {
			select {
			case <-ctx.Done():
				return nil, ctx.Err()
			case m, ok := <-rxChan:
				if !ok {
					return nil, io.EOF
				}
				return m, nil
			}
		})

		writer := portalstream.NewOrderedWriter(
			"ws_ping",
			func(m *portalpb.Mote) error {
				return portalMux.Publish(ctx, portalMux.TopicIn(int(portalID)), m)
			},
		)

		var wg sync.WaitGroup
		wg.Add(2)

		go func() {
			defer wg.Done()
			defer ws.Close()

			// Ping Read Loop (User -> Agent)
			// User sends WS messages (Pings), we send BytesPayload(Ping) to Agent.
			for {
				select {
				case <-ctx.Done():
					return
				default:
					_, message, err := ws.ReadMessage()
					if err != nil {
						return
					}
					// Send Ping to Agent
					if err := writer.WriteBytes(message, portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_PING); err != nil {
						return
					}
				}
			}
		}()

		go func() {
			defer wg.Done()
			defer ws.Close()

			// Ping Write Loop (Agent -> User)
			moteChan := make(chan *portalpb.Mote)
			errChan := make(chan error)

			go func() {
				for {
					m, err := reader.Read()
					if err != nil {
						errChan <- err
						return
					}
					moteChan <- m
				}
			}()

			for {
				select {
				case <-ctx.Done():
					return
				case err := <-errChan:
					if !errors.Is(err, io.EOF) {
						slog.ErrorContext(ctx, "error reading from portal stream", "error", err)
					}
					return
				case mote := <-moteChan:
					// Filter for Pings
					if b, ok := mote.Payload.(*portalpb.Mote_Bytes); ok && b.Bytes.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_PING {
						ws.SetWriteDeadline(time.Now().Add(writeWait))
						w, err := ws.NextWriter(websocket.BinaryMessage)
						if err != nil {
							return
						}
						if _, err := w.Write(b.Bytes.Data); err != nil {
							return
						}
						if err := w.Close(); err != nil {
							return
						}
					}
				}
			}
		}()

		wg.Wait()
	})
}

func manageActiveUser(ctx context.Context, done <-chan struct{}, graph *ent.Client, shellID int, userID int) {
	defer func() {
		wasAdded, err := graph.Shell.Query().
			Where(shell.ID(shellID)).
			QueryActiveUsers().
			Where(user.ID(userID)).
			Exist(ctx)
		if err != nil {
			return
		}
		if !wasAdded {
			return
		}

		if _, err := graph.Shell.UpdateOneID(shellID).
			RemoveActiveUserIDs(userID).
			Save(ctx); err != nil {
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
			alreadyAdded, err := graph.Shell.Query().
				Where(shell.ID(shellID)).
				QueryActiveUsers().
				Where(user.ID(userID)).
				Exist(ctx)
			if err != nil {
				continue
			}
			if alreadyAdded {
				continue
			}

			if _, err := graph.Shell.UpdateOneID(shellID).
				AddActiveUserIDs(userID).
				Save(ctx); err != nil {
			}
		}
	}
}
