package stream

import (
	"context"
	"fmt"
	"net/http"

	"github.com/gorilla/websocket"
	"gocloud.dev/pubsub"
	"golang.org/x/exp/slog"
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

// A MuxOption is used to provide further configuration to the Mux.
type MuxOption func(*Mux)

// NewMux initializes and returns a new Mux with the provided pubsub info.
func NewMux(pub *pubsub.Topic, sub *pubsub.Subscription, options ...MuxOption) *Mux {
	mux := &Mux{
		pub:        pub,
		sub:        sub,
		register:   make(chan *Stream, maxRegistrationBufSize),
		unregister: make(chan *Stream, maxRegistrationBufSize),
		streams:    make(map[*Stream]bool),
	}
	for _, opt := range options {
		opt(mux)
	}
	return mux
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
func (mux *Mux) registerStreams(ctx context.Context) {
	for {
		select {
		case s := <-mux.register:
			slog.DebugContext(ctx, "mux registering new stream", "stream_id", s.id)
			mux.streams[s] = true
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
func (mux *Mux) unregisterStreams(ctx context.Context) {
	for {
		select {
		case s := <-mux.unregister:
			slog.DebugContext(ctx, "mux unregistering stream", "stream_id", s.id)
			delete(mux.streams, s)
			s.Close()
		default:
			return
		}
	}
}

// Start the mux, returning an error if polling ever fails.
func (mux *Mux) Start(ctx context.Context) error {
	slog.DebugContext(ctx, "mux starting to manage streams and polling")
	for {
		// Manage Streams
		mux.registerStreams(ctx)
		mux.unregisterStreams(ctx)

		// Poll for new messages
		select {
		case <-ctx.Done():
			slog.DebugContext(ctx, "mux context finished, exiting")
			return ctx.Err()
		default:
			slog.DebugContext(ctx, "mux polling for message")
			if err := mux.poll(ctx); err != nil {
				slog.ErrorContext(ctx, "mux failed to poll subscription", "error", err)
			}
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
	mux.registerStreams(ctx)
	mux.unregisterStreams(ctx)

	// Get Message Metadata
	msgID, ok := msg.Metadata["id"]
	if !ok {
		slog.DebugContext(ctx, "mux received message without 'id' for stream, ignoring")
		return nil
	}
	msgOrderKey, ok := msg.Metadata[metadataOrderKey]
	if !ok {
		slog.DebugContext(ctx, "mux received message without metadataOrderKey")
	}
	msgOrderIndex, ok := msg.Metadata[metadataOrderIndex]
	if !ok {
		slog.DebugContext(ctx, "mux received message without msgOrderIndex")
	}

	// Broadcast Message
	slog.DebugContext(ctx, "mux broadcasting received message",
		"msg_id", msgID,
		"msg_order_key", msgOrderKey,
		"msg_order_index", msgOrderIndex,
		"stream_count", len(mux.streams),
	)
	for s := range mux.streams {
		if s == nil {
			slog.ErrorContext(ctx, "mux found nil stream in map while broadcasting message, skipping stream", "msg_id", msgID)
			continue
		}

		if s.id == msgID {
			slog.DebugContext(ctx, "mux sending message to stream",
				"msg_id", msgID,
				"stream_id", s.id,
				"stream_order_key", s.orderKey,
				"stream_index", s.orderIndex.Load(),
			)
			s.processOneMessage(ctx, msg)
		}
	}

	return nil
}
