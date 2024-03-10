package stream

import (
	"context"
	"fmt"
	"net/http"
	"strconv"

	"gocloud.dev/pubsub"
	"golang.org/x/net/websocket"
	"realm.pub/tavern/internal/ent"
)

// maxRegistrationBufSize defines the maximum receivers that can be buffered in the registration / unregistration channel
// before new calls to `mux.Register()` and `mux.Unregister()` will block.
const maxRegistrationBufSize = 256

// maxRecvMsgBufSize defines the maximum number of messages that can be buffered for a receiver before causing the Mux to block.
const maxRecvMsgBufSize = 1024

// A Receiver is registered with a Mux to receive filtered messages from a pubsub subscription.
type Receiver struct {
	id string
	ch chan *pubsub.Message
}

// NewReceiver initializes a new receiver that will only receive messages with the provided ID.
// It must be registered on a Mux to begin receiving messages.
func NewReceiver(id string) *Receiver {
	return &Receiver{
		id: id,
		ch: make(chan *pubsub.Message, maxRecvMsgBufSize),
	}
}

// Messages returns a channel for receiving new pubsub messages.
func (r *Receiver) Messages() <-chan *pubsub.Message {
	return r.ch
}

// A Mux enables multiplexing subscription messages to multiple Receivers.
// Receivers will only receive a Message if their configured ID matches the incoming metadata of a Message.
// Additionally, new messages may be published using the Mux.
type Mux struct {
	pub        *pubsub.Topic
	sub        *pubsub.Subscription
	register   chan *Receiver
	unregister chan *Receiver
	receivers  map[*Receiver]bool
}

// NewMux initializes and returns a new Mux with the provided pubsub info.
func NewMux(pub *pubsub.Topic, sub *pubsub.Subscription) *Mux {
	return &Mux{
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
			close(r.ch)
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

	// Always acknowledge the message
	defer msg.Ack()

	// Manage Receivers
	mux.registerReceivers()
	mux.unregisterReceivers()

	// Broadcast Message
	for r := range mux.receivers {
		if msg.Metadata["id"] == r.id {
			r.ch <- msg
		}
	}

	// Acknowledge Message
	msg.Ack()

	return nil
}

// NewShellHandler provides an HTTP handler which handles a websocket for shell io.
// It requires a query param "shell_id" be specified (must be an integer).
// This ID represents which Shell ent the websocket will connect to.
func NewShellHandler(graph *ent.Client, mux *Mux) http.HandlerFunc {

	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {

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
		handler := newShellWebsocketHandler(graph, shellID, mux)
		handler.ServeHTTP(w, r)
	})

}

func newShellWebsocketHandler(graph *ent.Client, shellID int, mux *Mux) websocket.Handler {
	return func(ws *websocket.Conn) {
		ctx := ws.Request().Context()

		// Load corresponding Shell
		shell, err := graph.Shell.Get(ctx, shellID)
		if err != nil {
			// TODO: Handle Error
			return
		}

		// Write all existing Shell output
		if err := websocket.JSON.Send(ws, &pubsub.Message{
			Body: shell.Output,
		}); err != nil {
			// TODO: Handle Error
			return
		}

		// Register output receiver
		r := NewReceiver(fmt.Sprintf("%d", shellID))
		mux.Register(r)
		defer mux.Unregister(r)

		done := make(chan struct{}, 1)
		go func() {
			for msg := range r.Messages() {
				if err := websocket.JSON.Send(ws, msg); err != nil {
					// TODO: Handle Error
					return
				}
			}
			done <- struct{}{}
		}()

		// Receive Websocket Messages and publish them
		for {
			select {
			case <-done:
			case <-ctx.Done():
				return
			default:
				var msg *pubsub.Message
				if err := websocket.JSON.Receive(ws, msg); err != nil {
					// TODO: Handle Error
					return
				}
				if err := mux.Send(ctx, msg); err != nil {
					// TODO: Handle Error
					return
				}
			}
		}
	}
}
