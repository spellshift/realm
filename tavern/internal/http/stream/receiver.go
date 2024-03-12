package stream

import (
	"log"
	"strconv"

	"gocloud.dev/pubsub"
)

// A Receiver is registered with a Mux to receive filtered messages from a pubsub subscription.
// It is NOT safe to use concurrently from multiple goroutines, however the Mux it is registered with
// should be.
type Receiver struct {
	id  string
	in  chan *pubsub.Message
	out chan *pubsub.Message

	nextToSend uint64
	buffer     map[uint64]*pubsub.Message
}

// NewReceiver initializes a new receiver that will only receive messages with the provided ID.
// It must be registered on a Mux to begin receiving messages.
func NewReceiver(id string) *Receiver {
	return &Receiver{
		id:     id,
		in:     make(chan *pubsub.Message, maxRecvMsgBufSize),
		out:    make(chan *pubsub.Message, maxRecvMsgBufSize),
		buffer: make(map[uint64]*pubsub.Message, maxRecvMsgBufSize),
	}
}

func (r *Receiver) recv(msg *pubsub.Message) {
	key, ok := parseOrderKey(msg)
	if !ok {
		// If no valid order key was provided, error but send the message
		// log.Printf("[ERROR] received message without order key")
		r.out <- msg
	} else if key == r.nextToSend || r.nextToSend == 0 {
		// If we receive the message we're looking for or if this is the first
		// message we've received, send it and update nextToSend
		r.out <- msg
		r.nextToSend = key + 1
	} else if key < r.nextToSend {
		// We prefer to drop messages instead of sending them out of order
		// If we receive a message after a subsequent one has been sent, drop it
		log.Printf("[ERROR] received message out of order too late")
	} else {
		// If we receive the message out of order, buffer it
		r.buffer[key] = msg
	}

	// Attempt to flush buffer
	for {
		nextMsg, ok := r.buffer[r.nextToSend]
		if !ok || nextMsg == nil {
			// If our buffer has grown too large, skip waiting for messages
			if len(r.buffer) > 10 { // TODO: Magic Number
				r.nextToSend += 1
				continue
			}

			// Otherwise, continue waiting for the next message in the order
			break
		}
		r.out <- nextMsg
		delete(r.buffer, r.nextToSend)
		r.nextToSend += 1
	}
}

func (r *Receiver) Close() error {
	close(r.in)
	return nil
}

// Messages returns a channel for receiving new pubsub messages.
func (r *Receiver) Messages() <-chan *pubsub.Message {
	return r.out
}

func parseOrderKey(msg *pubsub.Message) (uint64, bool) {
	keyStr, ok := msg.Metadata["order-key"]
	if !ok {
		return 0, false
	}
	key, err := strconv.ParseUint(keyStr, 10, 64)
	if err != nil {
		return 0, false
	}
	return key, true
}
