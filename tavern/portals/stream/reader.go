package stream

import (
	"errors"
	"fmt"
	"time"

	"realm.pub/tavern/portals/portalpb"
)

// ReceiverFunc is a callback that reads a Mote from a source.
// This allows OrderedReader to wrap any gRPC stream method.
type ReceiverFunc func() (*portalpb.Mote, error)

// OrderedReader receives Motes and ensures they are read in order.
type OrderedReader struct {
	nextSeqID       uint64
	buffer          map[uint64]*portalpb.Mote
	maxBuffer       int
	staleTimeout    time.Duration
	firstBufferedAt time.Time
}

// WithMaxBufferedMessages sets the maximum number of out-of-order messages to buffer.
func WithMaxBufferedMessages(max int) func(*OrderedReader) {
	return func(r *OrderedReader) {
		r.maxBuffer = max
	}
}

// WithStaleBufferTimeout sets the duration to wait for the next expected sequence ID before erroring if other messages are arriving.
func WithStaleBufferTimeout(d time.Duration) func(*OrderedReader) {
	return func(r *OrderedReader) {
		r.staleTimeout = d
	}
}

// NewOrderedReader creates a new OrderedReader.
// maxBuffer limits the number of out-of-order messages to buffer.
// staleTimeout is the duration to wait for the next expected sequence ID before erroring if other messages are arriving.
func NewOrderedReader(options ...func(*OrderedReader)) *OrderedReader {
	reader := &OrderedReader{
		nextSeqID:    0,
		buffer:       make(map[uint64]*portalpb.Mote),
		maxBuffer:    1024,
		staleTimeout: 5 * time.Second,
	}
	for _, opt := range options {
		opt(reader)
	}
	return reader
}

// Process handles a new incoming mote.
// If the mote is the expected next one, it returns it along with any subsequent buffered motes that are now in order.
// If the mote is out of order (future), it buffers it and returns nil.
// If the mote is a duplicate (past), it returns nil.
// Returns an error if buffer limits or timeouts are exceeded.
func (r *OrderedReader) Process(mote *portalpb.Mote) ([]*portalpb.Mote, error) {
	if mote.SeqId == r.nextSeqID {
		// Expected packet
		result := []*portalpb.Mote{mote}
		r.nextSeqID++

		// Check for buffered subsequent packets
		for {
			if bufferedMote, ok := r.buffer[r.nextSeqID]; ok {
				result = append(result, bufferedMote)
				delete(r.buffer, r.nextSeqID)
				r.nextSeqID++
			} else {
				break
			}
		}

		// Reset buffer time if buffer becomes empty
		if len(r.buffer) == 0 {
			r.firstBufferedAt = time.Time{}
		} else {
			// Reset timer to now as we made progress
			r.firstBufferedAt = time.Now()
		}

		return result, nil
	} else if mote.SeqId > r.nextSeqID {
		// Future packet (gap)
		if len(r.buffer) == 0 {
			r.firstBufferedAt = time.Now()
		}

		if _, exists := r.buffer[mote.SeqId]; !exists {
			r.buffer[mote.SeqId] = mote
		}

		if len(r.buffer) > r.maxBuffer {
			return nil, errors.New("stale stream: buffer limit exceeded")
		}

		// Check timeout
		if time.Since(r.firstBufferedAt) > r.staleTimeout {
			return nil, fmt.Errorf("stale stream: timeout waiting for seqID %d", r.nextSeqID)
		}

		return nil, nil
	} else {
		// Duplicate or old packet. Ignore.
		return nil, nil
	}
}
