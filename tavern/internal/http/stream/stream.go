package stream

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"io"
	"log/slog"
	"strconv"
	"sync"
	"sync/atomic"

	"gocloud.dev/pubsub"
)

// metadataID defines the ID of the entity for the stream, enabling multiple ents to be multiplexed over a topic.
// metadataOrderKey defines the key for specifying a unique stream ID, which all published messages should be ordered (per order key).
// metadataOrderIndex defines the index of a published message within the context of an order key.
const (
	metadataID         = "id"
	metadataOrderKey   = "order-key"
	metadataOrderIndex = "order-index"

	// MetadataStreamClose can be added by producers when publishing messages to a topic, indicating no more messages will be sent to the stream.
	// This should never be sent if there is more than one producer for the stream (e.g. only send from gRPC, not from websockets).
	MetadataStreamClose = "stream-close"

	// MetadataMsgKind defines the kind of message (data, ping)
	MetadataMsgKind = "msg_kind"

	// maxStreamMsgBufSize defines the maximum number of messages that can be buffered for a stream before causing the Mux to block.
	maxStreamMsgBufSize = 1024

	// maxStreamOrderBuf defines how many messages to wait before dropping an out-of-order message and move on.
	maxStreamOrderBuf = 10
)

// A Stream is registered with a Mux to receive filtered messages from a pubsub subscription.
type Stream struct {
	mu sync.Mutex

	id          string // ID of the underlying ent (e.g. Shell)
	orderKey    string // Unique ID for the session (e.g. Websocket)
	orderIndex  *atomic.Uint64
	recv        chan *pubsub.Message
	recvOrdered chan *pubsub.Message
	buffers     map[string]*sessionBuffer // Receive messages, bucket by sender (order-key), and order messages from same sender
}

// New initializes a new stream that will only receive messages with the provided ID.
// It must be registered on a Mux to begin receiving messages.
// This method panics if it fails to generate a random string for the order-key.
func New(id string) *Stream {
	return &Stream{
		id:          id,
		orderKey:    newOrderKey(),
		orderIndex:  &atomic.Uint64{},
		recv:        make(chan *pubsub.Message, maxStreamMsgBufSize),
		recvOrdered: make(chan *pubsub.Message, maxStreamMsgBufSize),
		buffers:     make(map[string]*sessionBuffer, maxStreamMsgBufSize),
	}
}

// SendMessage
func (s *Stream) SendMessage(ctx context.Context, msg *pubsub.Message, mux *Mux) error {
	// Add Metadata
	if msg.Metadata == nil {
		msg.Metadata = make(map[string]string, 3)
	}
	msg.Metadata[metadataID] = s.id
	msg.Metadata[metadataOrderKey] = s.orderKey
	msg.Metadata[metadataOrderIndex] = fmt.Sprintf("%d", s.orderIndex.Load())

	// Send Message to Mux
	err := mux.send(ctx, msg)
	if err != nil {
		return err
	}

	// Track Index for Stream
	s.orderIndex.Add(1)
	return nil
}

// Messages returns a channel for receiving new pubsub messages.
func (s *Stream) Messages() <-chan *pubsub.Message {
	return s.recvOrdered
}

// Close the stream, preventing it from receiving any new messages.
// The Mux a stream is registered with will call Close() when it is unregistered.
func (s *Stream) Close() error {
	s.mu.Lock()
	defer s.mu.Unlock()

	close(s.recv)
	close(s.recvOrdered)
	return nil
}

// processOneMessage is called by a Mux to receive a new pubsub message designated for this stream.
// It may be unordered.
func (s *Stream) processOneMessage(ctx context.Context, msg *pubsub.Message) {
	s.mu.Lock()
	defer s.mu.Unlock()

	// Get Buffer for the given order-key
	// We order all messages within the same order key, or create a new one
	// if we have not seen this order key yet.
	key := parseOrderKey(msg)
	buf, ok := s.buffers[key]
	if !ok || buf == nil {
		buf = &sessionBuffer{
			data: make(map[uint64]*pubsub.Message, maxStreamMsgBufSize),
		}
		s.buffers[key] = buf
	}

	// Write Message (or buffer it)
	emit := func(m *pubsub.Message) {
		s.recvOrdered <- m
	}
	buf.writeMessage(ctx, msg, emit)
}

func parseOrderKey(msg *pubsub.Message) string {
	key, ok := msg.Metadata[metadataOrderKey]
	if !ok {
		return ""
	}
	return key
}

func parseOrderIndex(msg *pubsub.Message) (uint64, bool) {
	indexStr, ok := msg.Metadata[metadataOrderIndex]
	if !ok {
		return 0, false
	}
	index, err := strconv.ParseUint(indexStr, 10, 64)
	if err != nil {
		return 0, false
	}
	return index, true
}

// newOrderKey returns a new base64 encoded random string, intended for use as a unique order key.
// If it fails to generate a key, it panics.
func newOrderKey() string {
	buf := make([]byte, 16)
	_, err := io.ReadFull(rand.Reader, buf)
	if err != nil {
		slog.Error("failed to generate order key, defaulting to empty string!", "error", err)
		return ""
	}
	return base64.StdEncoding.EncodeToString(buf)
}

// A sessionBuffer enables ordering of messages for a given session of the stream.
// For example, if multiple websockets send to the same pubsub topic, we need to ensure
// that their messages can be received in order. To accomplish this, each websocket (a "session")
// is assigned a unique UUID, which is associated with all messages it publishes (as the order-key).
// Then, when receiving messages, streams will order all messages received from the same order-key.
// In this example, websockets could race to add data, but all the data they add will be in order
// with respect to the websocket.
type sessionBuffer struct {
	nextToSend uint64
	data       map[uint64]*pubsub.Message
}

func (buf *sessionBuffer) writeMessage(ctx context.Context, msg *pubsub.Message, emit func(*pubsub.Message)) {
	index, ok := parseOrderIndex(msg)
	if !ok {
		slog.DebugContext(ctx, "sessionBuffer received no order index, will write immediately and not buffer")
		emit(msg)
		return
	}

	if index < buf.nextToSend {
		slog.ErrorContext(ctx, "dropping message because subsequent message has already been sent",
			"msg_log_id", msg.LoggableID,
			"next_to_send", buf.nextToSend,
			"order_index", index,
		)
		return
	}

	buf.data[index] = msg
	buf.flushBuffer(ctx, emit)
}

func (buf *sessionBuffer) flushBuffer(ctx context.Context, emit func(*pubsub.Message)) {
	for {
		msg, ok := buf.data[buf.nextToSend]
		if !ok {
			if len(buf.data) > maxStreamOrderBuf {
				// To prevent getting stuck, find the lowest index in the buffer and jump to it.
				lowestIndex := uint64(0)
				for k := range buf.data {
					if lowestIndex == 0 || k < lowestIndex {
						lowestIndex = k
					}
				}
				slog.ErrorContext(ctx, "sessionBuffer overflow, skipping message to catch up",
					"skipped_message_index", buf.nextToSend,
					"new_index", lowestIndex,
					"buffered_msgs_count", len(buf.data),
				)
				buf.nextToSend = lowestIndex
				continue
			}
			break
		}
		emit(msg)
		delete(buf.data, buf.nextToSend)
		buf.nextToSend++
	}
}
