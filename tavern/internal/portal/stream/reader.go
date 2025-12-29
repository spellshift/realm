package stream

import (
	"errors"
	"time"

	"realm.pub/tavern/internal/portal/portalpb"
)

var (
	ErrBufferLimitExceeded = errors.New("buffer limit exceeded")
	ErrStaleTimeout        = errors.New("stale timeout")
)

type Receiver func() (*portalpb.Payload, error)

type OrderedReader struct {
	receiver Receiver

	nextSeqID uint64
	buffer    map[uint64]*portalpb.Payload

	maxBuffer int
	timeout   time.Duration

	firstBufferedTime time.Time
}

// NewOrderedReader creates a new OrderedReader.
// maxBuffer is the maximum number of out-of-order messages to buffer.
// timeout is the maximum duration to wait for a missing message after receiving out-of-order messages.
func NewOrderedReader(receiver Receiver, maxBuffer int, timeout time.Duration) *OrderedReader {
	return &OrderedReader{
		receiver:  receiver,
		buffer:    make(map[uint64]*portalpb.Payload),
		maxBuffer: maxBuffer,
		timeout:   timeout,
	}
}

func (r *OrderedReader) Read() (*portalpb.Payload, error) {
	// First, check if we have the next message in the buffer
	if msg, ok := r.buffer[r.nextSeqID]; ok {
		delete(r.buffer, r.nextSeqID)
		r.nextSeqID++
		if len(r.buffer) > 0 {
			r.firstBufferedTime = time.Now()
		} else {
			r.resetStaleState()
		}
		return msg, nil
	}

	for {
		// Check for stale timeout if we are buffering
		if len(r.buffer) > 0 {
			if time.Since(r.firstBufferedTime) > r.timeout {
				return nil, ErrStaleTimeout
			}
		}

		// Read from the receiver
		msg, err := r.receiver()
		if err != nil {
			return nil, err
		}

		if msg.SeqId == r.nextSeqID {
			r.nextSeqID++
			if len(r.buffer) > 0 {
				r.firstBufferedTime = time.Now()
			} else {
				r.resetStaleState()
			}
			return msg, nil
		}

		if msg.SeqId < r.nextSeqID {
			// Ignore duplicate or old messages
			continue
		}

		// Message is from the future (out of order)
		if len(r.buffer) == 0 {
			r.firstBufferedTime = time.Now()
		}

		// Prevent unbounded growth if duplicates arrive or if we just get a flood of future packets
		if _, exists := r.buffer[msg.SeqId]; !exists {
			r.buffer[msg.SeqId] = msg
			if len(r.buffer) > r.maxBuffer {
				return nil, ErrBufferLimitExceeded
			}
		}
	}
}

func (r *OrderedReader) resetStaleState() {
	r.firstBufferedTime = time.Time{}
}
