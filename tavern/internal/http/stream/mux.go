package stream

import (
	"context"
	"fmt"
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
	// maxRegistrationBufSize defines the maximum receivers that can be buffered in the registration / unregistration channel
	// before new calls to `mux.Register()` and `mux.Unregister()` will block.
	maxRegistrationBufSize = 256

	// maxRecvMsgBufSize defines the maximum number of messages that can be buffered for a receiver before causing the Mux to block.
	maxRecvMsgBufSize = 1024

	// Time allowed to write a message to the peer.
	writeWait = 10 * time.Second

	// Time allowed to read the next pong message from the peer.
	pongWait = 60 * time.Second

	// Send pings to peer with this period. Must be less than pongWait.
	pingPeriod = (pongWait * 9) / 10

	// Maximum message size allowed from peer.
	maxMessageSize = 512
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin:     func(r *http.Request) bool { return true },
}

type connector struct {
	*Receiver
	mux *Mux
	ws  *websocket.Conn
}

// WriteToWebsocket will read messages from the Mux and write them to the underlying websocket.
func (c *connector) WriteToWebsocket(ctx context.Context) {
	defer c.ws.Close()

	// Register with mux to receive messages
	c.mux.Register(c.Receiver)
	defer c.mux.Unregister(c.Receiver)

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
			if err := c.mux.Send(ctx, &pubsub.Message{
				Body: message,
				Metadata: map[string]string{
					"id": c.id,
				},
			}); err != nil {
				log.Printf("[WS][ERROR] failed to publish message: %v", err)
				return
			}
		}
	}
}

// A Mux enables multiplexing subscription messages to multiple Receivers.
// Receivers will only receive a Message if their configured ID matches the incoming metadata of a Message.
// Additionally, new messages may be published using the Mux.
type Mux struct {
	name       string
	pub        *pubsub.Topic
	sub        *pubsub.Subscription
	register   chan *Receiver
	unregister chan *Receiver
	receivers  map[*Receiver]bool
}

// NewMux initializes and returns a new Mux with the provided pubsub info.
func NewMux(name string, pub *pubsub.Topic, sub *pubsub.Subscription) *Mux {
	return &Mux{
		name:       name,
		pub:        pub,
		sub:        sub,
		register:   make(chan *Receiver, maxRegistrationBufSize),
		unregister: make(chan *Receiver, maxRegistrationBufSize),
		receivers:  make(map[*Receiver]bool),
	}
}

// Send a new message to the configured publish topic.
// The provided message MUST include an id metadata.
func (mux *Mux) Send(ctx context.Context, m *pubsub.Message) error {
	fmt.Printf("[%s] Publishing: %q", mux.name, string(m.Body))
	if _, ok := m.Metadata["id"]; !ok {
		return fmt.Errorf("must set 'id' metadata before publishing")
	}
	return mux.pub.Send(ctx, m)
}

// Register a new receiver with the Mux, which will receive broadcast messages from a pubsub subscription
// if the Message metadata ID matches the receiver ID.
func (mux *Mux) Register(r *Receiver) {
	mux.register <- r
}

// registerReceivers inserts all registered receivers into the receivers map.
func (mux *Mux) registerReceivers() {
	for {
		select {
		case r := <-mux.register:
			log.Printf("[MUX] Registering Receiver (id=%q)", r.id)
			mux.receivers[r] = true
		default:
			return
		}
	}
}

// Unregister a receiver when it should no longer receive Messages from the Mux.
// Typically this is done via defer after registering a Receiver.
// Unregistering a receiver that is not registered will still close the receiver channel.
func (mux *Mux) Unregister(r *Receiver) {
	mux.unregister <- r
}

// unregisterReceivers deletes all unregistered receivers from the receivers map.
func (mux *Mux) unregisterReceivers() {
	for {
		select {
		case r := <-mux.unregister:
			delete(mux.receivers, r)
			r.Close()
		default:
			return
		}
	}
}

// Start the mux, returning an error if polling ever fails.
func (mux *Mux) Start(ctx context.Context) error {
	for {
		// Manage Receivers
		mux.registerReceivers()
		mux.unregisterReceivers()

		// Poll for new messages
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			mux.poll(ctx)
		}
	}
}

// poll for a new message, broadcasting to all registered receivers.
// poll will also register & unregister receivers after a new message is received.
func (mux *Mux) poll(ctx context.Context) error {
	// Block waiting for message
	msg, err := mux.sub.Receive(ctx)
	if err != nil {
		return fmt.Errorf("failed to poll for new message: %w", err)
	}

	log.Printf("[%s] RECEIVED MESSAGE: %q", mux.name, string(msg.Body))

	// Always acknowledge the message
	defer msg.Ack()

	// Manage Receivers
	mux.registerReceivers()
	mux.unregisterReceivers()

	// Broadcast Message
	for r := range mux.receivers {
		if msg.Metadata["id"] == r.id {
			r.recv(msg)
		}
	}

	// Acknowledge Message
	// msg.Ack()

	return nil
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
		rcv := NewReceiver(shellIDStr)

		// Create Connector
		conn := &connector{
			Receiver: rcv,
			mux:      mux,
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
	})
}
