package stream

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"io"
	"log"
	"strconv"
	"sync"
	"sync/atomic"

	"gocloud.dev/pubsub"
)

// TODO: Docs
const (
	metadataOrderKey   = "order-key"
	metadataOrderIndex = "order-index"

	// MetadataStreamClose can be added by producers when publishing messages to a topic, indicating no more messages will be sent to the stream.
	// This should never be sent if there is more than one producer for the stream (e.g. only send from gRPC, not from websockets).
	MetadataStreamClose = "stream-close"

	// maxStreamMsgBufSize defines the maximum number of messages that can be buffered for a stream before causing the Mux to block.
	maxStreamMsgBufSize = 1024
)

// A Stream is registered with a Mux to receive filtered messages from a pubsub subscription.
type Stream struct {
	mu sync.Mutex

	id          string
	orderKey    string
	orderIndex  *atomic.Uint64
	recv        chan *pubsub.Message
	recvOrdered chan *pubsub.Message
	buffers     map[string]*sessionBuffer
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
	msg.Metadata["id"] = s.id
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
	return nil
}

// processOneMessage is called by a Mux to receive a new pubsub message designated for this stream.
// It may be unordered.
func (s *Stream) processOneMessage(msg *pubsub.Message) {
	s.mu.Lock()
	defer s.mu.Unlock()

	// Get Buffer
	key := parseOrderKey(msg)
	buf, ok := s.buffers[key]
	if !ok || buf == nil {
		buf = &sessionBuffer{
			data: make(map[uint64]*pubsub.Message, maxStreamMsgBufSize),
		}
		s.buffers[key] = buf
	}

	// Write Message (or buffer it)
	buf.writeMessage(msg, s.recvOrdered)

	// Flush possible messages from buffer
	buf.flushBuffer(s.recvOrdered)
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
		panic(fmt.Errorf("failed to generate random token: %w", err))
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

func (buf *sessionBuffer) writeMessage(msg *pubsub.Message, dst chan<- *pubsub.Message) {
	index, ok := parseOrderIndex(msg)
	if !ok {
		// If no valid order key was provided, error but send the message
		// log.Printf("[ERROR] received message without order key")
		dst <- msg
	} else if index == buf.nextToSend || buf.nextToSend == 0 {
		// If we receive the message we're looking for or if this is the first
		// message we've received, send it and update nextToSend
		dst <- msg
		buf.nextToSend = index + 1
	} else if index < buf.nextToSend {
		// We prefer to drop messages instead of sending them out of order
		// If we receive a message after a subsequent one has been sent, drop it
		log.Printf("[ERROR] dropping message, subsequent message has already been sent")
	} else {
		// If we receive the message out of order, buffer it
		buf.data[index] = msg
	}
}

func (buf *sessionBuffer) flushBuffer(dst chan<- *pubsub.Message) {
	for {
		msg, ok := buf.data[buf.nextToSend]
		if !ok || msg == nil {
			// If our buffer has grown too large, skip waiting for messages
			if len(buf.data) > 10 { // TODO: Magic Number
				buf.nextToSend += 1
				continue
			}

			// Otherwise, continue waiting for the next message in the order
			break
		}
		dst <- msg
		delete(buf.data, buf.nextToSend)
		buf.nextToSend += 1
	}
}
