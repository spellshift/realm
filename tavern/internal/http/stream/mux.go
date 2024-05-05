package stream

import (
	"context"
	"fmt"
	"net/http"

	"github.com/gorilla/websocket"
	"gocloud.dev/pubsub"
)

const (
	// maxRegistrationBufSize defines the maximum receivers that can be buffered in the registration / unregistration channel
	// before new calls to `mux.Register()` and `mux.Unregister()` will block.
	maxRegistrationBufSize = 256
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin:     func(r *http.Request) bool { return true },
}

// A Mux enables multiplexing subscription messages to multiple Streams.
// Streams will only receive a Message if their configured ID matches the incoming metadata of a Message.
// Additionally, new messages may be published using the Mux.
type Mux struct {
	pub        *pubsub.Topic
	sub        *pubsub.Subscription
	register   chan *Stream
	unregister chan *Stream
	streams    map[*Stream]bool
}

// NewMux initializes and returns a new Mux with the provided pubsub info.
func NewMux(pub *pubsub.Topic, sub *pubsub.Subscription) *Mux {
	return &Mux{
		pub:        pub,
		sub:        sub,
		register:   make(chan *Stream, maxRegistrationBufSize),
		unregister: make(chan *Stream, maxRegistrationBufSize),
		streams:    make(map[*Stream]bool),
	}
}

// send a new message to the configured publish topic.
// The provided message MUST include an id metadata.
func (mux *Mux) send(ctx context.Context, m *pubsub.Message) error {
	if _, ok := m.Metadata[metadataID]; !ok {
		return fmt.Errorf("must set 'id' metadata before publishing")
	}
	return mux.pub.Send(ctx, m)
}

// Register a new stream with the Mux, which will receive broadcast messages from a pubsub subscription
// if the Message metadata ID matches the stream ID.
func (mux *Mux) Register(s *Stream) {
	mux.register <- s
}

// registerStreams inserts all registered streams into the streams map.
func (mux *Mux) registerStreams() {
	for {
		select {
		case r := <-mux.register:
			mux.streams[r] = true
		default:
			return
		}
	}
}

// Unregister a stream when it should no longer receive Messages from the Mux.
// Typically this is done via defer after registering a Stream.
// Unregistering a stream that is not registered will still close the stream channel.
func (mux *Mux) Unregister(s *Stream) {
	mux.unregister <- s
}

// unregisterStreams deletes all unregistered streams from the streams map.
func (mux *Mux) unregisterStreams() {
	for {
		select {
		case s := <-mux.unregister:

			delete(mux.streams, s)
			s.Close()
		default:
			return
		}
	}
}

// Start the mux, returning an error if polling ever fails.
func (mux *Mux) Start(ctx context.Context) error {
	for {
		// Manage Streams
		mux.registerStreams()
		mux.unregisterStreams()

		// Poll for new messages
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			mux.poll(ctx)
		}
	}
}

// poll for a new message, broadcasting to all registered streams.
// poll will also register & unregister streams after a new message is received.
func (mux *Mux) poll(ctx context.Context) error {
	// Block waiting for message
	msg, err := mux.sub.Receive(ctx)
	if err != nil {
		return fmt.Errorf("failed to poll for new message: %w", err)
	}

	// Always acknowledge the message
	defer msg.Ack()

	// Manage Streams
	mux.registerStreams()
	mux.unregisterStreams()

	// Broadcast Message
	for s := range mux.streams {
		if msg.Metadata["id"] == s.id {
			s.processOneMessage(msg)
		}
	}

	return nil
}
