package stream

import (
	"context"
	"fmt"
	"net/http"

	"github.com/gorilla/websocket"
	"golang.org/x/exp/slog"
	"realm.pub/tavern/internal/xpubsub"
)

const (
	// maxRegistrationBufSize defines the maximum receivers that can be buffered in the registration / unregistration channel
	// before new calls to `mux.Register()` and `mux.Unregister()` will block.
	maxRegistrationBufSize = 256
	// defaultHistorySize is the default size of the circular buffer for stream history.
	defaultHistorySize = 1024
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin:     func(r *http.Request) bool { return true },
}

type historyState struct {
	buffer   *CircularBuffer
	sessions map[string]*sessionBuffer
}

// A Mux enables multiplexing subscription messages to multiple Streams.
// Streams will only receive a Message if their configured ID matches the incoming metadata of a Message.
// Additionally, new messages may be published using the Mux.
type Mux struct {
	pub         xpubsub.Publisher
	sub         xpubsub.Subscriber
	register    chan *Stream
	unregister  chan *Stream
	streams     map[*Stream]bool
	history     map[string]*historyState
	historySize int
}

// A MuxOption is used to provide further configuration to the Mux.
type MuxOption func(*Mux)

// WithHistorySize sets the size of the circular buffer for stream history.
func WithHistorySize(size int) MuxOption {
	return func(m *Mux) {
		m.historySize = size
	}
}

// NewMux initializes and returns a new Mux with the provided pubsub info.
func NewMux(pub xpubsub.Publisher, sub xpubsub.Subscriber, options ...MuxOption) *Mux {
	mux := &Mux{
		pub:         pub,
		sub:         sub,
		register:    make(chan *Stream, maxRegistrationBufSize),
		unregister:  make(chan *Stream, maxRegistrationBufSize),
		streams:     make(map[*Stream]bool),
		history:     make(map[string]*historyState),
		historySize: defaultHistorySize,
	}
	for _, opt := range options {
		opt(mux)
	}
	return mux
}

// send a new message to the configured publish topic.
// The provided message MUST include an id metadata.
func (mux *Mux) send(ctx context.Context, m *xpubsub.Message) error {
	if _, ok := m.Metadata[metadataID]; !ok {
		return fmt.Errorf("must set 'id' metadata before publishing")
	}
	return mux.pub.Publish(ctx, m.Body, m.Metadata)
}

// Register a new stream with the Mux, which will receive broadcast messages from a pubsub subscription
// if the Message metadata ID matches the stream ID.
func (mux *Mux) Register(s *Stream) {
	mux.register <- s
}

// Unregister a stream when it should no longer receive Messages from the Mux.
// Typically this is done via defer after registering a Stream.
// Unregistering a stream that is not registered will still close the stream channel.
func (mux *Mux) Unregister(s *Stream) {
	mux.unregister <- s
}

// Start the mux, returning an error if polling ever fails.
func (mux *Mux) Start(ctx context.Context) error {
	slog.DebugContext(ctx, "mux starting to manage streams and polling")

	// Message channel to receive messages from the poller
	type pollResult struct {
		msg *xpubsub.Message
		err error
	}
	msgChan := make(chan pollResult)

	// Start poller goroutine
	go func() {
		defer close(msgChan)
		for {
			msg, err := mux.sub.Receive(ctx)
			select {
			case <-ctx.Done():
				return
			case msgChan <- pollResult{msg: msg, err: err}:
				// If context is done, stop.
				if ctx.Err() != nil {
					return
				}
				// Otherwise, loop again (retry on error).
			}
		}
	}()

	for {
		select {
		case <-ctx.Done():
			slog.DebugContext(ctx, "mux context finished, exiting")
			return ctx.Err()

		case s := <-mux.register:
			// Handle Registration
			slog.DebugContext(ctx, "mux registering new stream", "stream_id", s.id)
			mux.streams[s] = true

			// Send history to the new stream
			if state, ok := mux.history[s.id]; ok && state.buffer != nil {
				data := state.buffer.Bytes()
				if len(data) > 0 {
					slog.DebugContext(ctx, "mux sending history to new stream", "stream_id", s.id, "bytes", len(data))
					msg := &xpubsub.Message{
						Body: data,
						Metadata: map[string]string{
							metadataID:      s.id,
							MetadataMsgKind: "data",
							// No order key needed for history injection
						},
					}
					s.processOneMessage(ctx, msg)
				}
			}

		case s := <-mux.unregister:
			// Handle Unregistration
			slog.DebugContext(ctx, "mux unregistering stream", "stream_id", s.id)
			delete(mux.streams, s)
			s.Close()

		case res, ok := <-msgChan:
			if !ok {
				// Poller exited. If due to context cancel, return that error.
				if ctx.Err() != nil {
					return ctx.Err()
				}
				return fmt.Errorf("poller exited unexpectedly")
			}
			if res.err != nil {
				// Log error and continue, matching original behavior (retry loop).
				// Unless context is done.
				if ctx.Err() != nil {
					return ctx.Err()
				}
				slog.ErrorContext(ctx, "mux failed to poll subscription", "error", res.err)
				continue
			}

			// Handle Message
			mux.handleMessage(ctx, res.msg)
		}
	}
}

// handleMessage processes a new message, updating history and broadcasting to streams.
func (mux *Mux) handleMessage(ctx context.Context, msg *xpubsub.Message) {
	// Always acknowledge the message
	defer msg.Ack()

	// Get Message Metadata
	msgID, ok := msg.Metadata["id"]
	if !ok {
		slog.DebugContext(ctx, "mux received message without 'id' for stream, ignoring")
		return
	}
	msgOrderKey, ok := msg.Metadata[metadataOrderKey]
	if !ok {
		slog.DebugContext(ctx, "mux received message without metadataOrderKey")
	}
	msgOrderIndex, ok := msg.Metadata[metadataOrderIndex]
	if !ok {
		slog.DebugContext(ctx, "mux received message without msgOrderIndex")
	}

	// Update History
	// Only buffer "data" messages (or messages with no kind specified, which default to data)
	kind, hasKind := msg.Metadata[MetadataMsgKind]
	if !hasKind || kind == "data" {
		state, ok := mux.history[msgID]
		if !ok {
			state = &historyState{
				buffer:   NewCircularBuffer(mux.historySize),
				sessions: make(map[string]*sessionBuffer),
			}
			mux.history[msgID] = state
		}

		// Use sessionBuffer to reorder messages before writing to circular buffer
		key := parseOrderKey(msg)
		sessBuf, ok := state.sessions[key]
		if !ok {
			sessBuf = &sessionBuffer{
				data: make(map[uint64]*xpubsub.Message, maxStreamOrderBuf),
			}
			state.sessions[key] = sessBuf
		}

		sessBuf.writeMessage(ctx, msg, func(m *xpubsub.Message) {
			state.buffer.Write(m.Body)
		})
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
}
