package stream

import (
	"context"
	"log"
	"net/http"
	"strconv"
	"sync"
	"time"

	"github.com/gorilla/websocket"
	"gocloud.dev/pubsub"
	"realm.pub/tavern/internal/ent"
)

const (
	// Time allowed to write a message to the peer.
	writeWait = 10 * time.Second

	// Time allowed to read the next pong message from the peer.
	pongWait = 60 * time.Second

	// Send pings to peer with this period. Must be less than pongWait.
	pingPeriod = (pongWait * 9) / 10

	// Maximum message size allowed from peer.
	maxMessageSize = 512
)

type connector struct {
	*Stream
	mux *Mux
	ws  *websocket.Conn
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
			log.Printf("GETTING WEBSOCKET MESSAGE: %q", string(message.Body))
			c.ws.SetWriteDeadline(time.Now().Add(writeWait))
			if !ok {
				// The mux closed the channel.
				c.ws.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}

			w, err := c.ws.NextWriter(websocket.TextMessage)
			if err != nil {
				return
			}
			w.Write(message.Body)

			// Add queued messages to the current websocket message.
			// n := len(c.Messages())
			// for i := 0; i < n; i++ {
			// 	additionalMsg := <-c.Messages()
			// 	w.Write(additionalMsg.Body)
			// }

			if err := w.Close(); err != nil {
				return
			}
		case <-ticker.C:
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
					log.Printf("[WS][ERROR] websocket closed unexpectedly: %v", err)
				}
				return
			}
			if err := c.Stream.SendMessage(ctx, &pubsub.Message{
				Body: message,
				Metadata: map[string]string{
					"id": c.id,
				},
			}, c.mux); err != nil {
				log.Printf("[WS][ERROR] failed to publish message: %v", err)
				return
			}
		}
	}
}

// NewShellHandler provides an HTTP handler which handles a websocket for shell io.
// It requires a query param "shell_id" be specified (must be an integer).
// This ID represents which Shell ent the websocket will connect to.
func NewShellHandler(graph *ent.Client, mux *Mux) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()
		log.Printf("[WS] New Shell Websocket Connection")

		ws, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			log.Printf("[WS][ERROR] Failed to upgrade connection to websocket: %v", err)
			return
		}

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

		// Start Websocket
		log.Printf("[WS] New Shell Websocket Connection (shell_id=%d)", shellID)
		stream := New(shellIDStr)

		// Create Connector
		conn := &connector{
			Stream: stream,
			mux:    mux,
			ws:     ws,
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
