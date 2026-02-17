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
	receiver        ReceiverFunc
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
func NewOrderedReader(receiver ReceiverFunc, options ...func(*OrderedReader)) *OrderedReader {
	reader := &OrderedReader{
		nextSeqID:    0,
		buffer:       make(map[uint64]*portalpb.Mote),
		maxBuffer:    1024,
		staleTimeout: 5 * time.Second,
		receiver:     receiver,
	}
	for _, opt := range options {
		opt(reader)
	}
	return reader
}

// Read returns the next ordered Mote.
// It will block until the next ordered Mote is available or an error occurs.
func (r *OrderedReader) Read() (*portalpb.Mote, error) {
	// First check if the next message is already in the buffer
	if mote, ok := r.buffer[r.nextSeqID]; ok {
		delete(r.buffer, r.nextSeqID)
		r.nextSeqID++
		// Reset buffer time if buffer becomes empty
		if len(r.buffer) == 0 {
			r.firstBufferedAt = time.Time{}
		}
		return mote, nil
	}

	// Check for stale state before reading
	if len(r.buffer) > 0 {
		if time.Since(r.firstBufferedAt) > r.staleTimeout {
			return nil, fmt.Errorf("stale stream: timeout waiting for seqID %d", r.nextSeqID)
		}
	}

	// Read loop
	for {
		mote, err := r.receiver()
		if err != nil {
			return nil, err
		}

		if mote.SeqId == r.nextSeqID {
			r.nextSeqID++
			// We found the next packet. Check if we have subsequent packets buffered.
			// However, since we return one packet at a time, we just return this one.
			// The next call to Read() will check the buffer.

			// If buffer is empty now (implied, unless we had gaps filled out of order, which Read check handles), reset timer?
			// Actually, if we just returned X, and X+1 is in buffer, next Read gets X+1.
			// If X+1 is NOT in buffer but X+2 is, the timer (firstBufferedAt) should presumably persist?
			// The original logic was: firstBufferedAt is set when the buffer goes from empty to non-empty.
			// Here, if we return X, and buffer has items, we might still be waiting for X+1 (if X was fresh but we have X+2).
			// But wait, if we just received X, and we have X+2, X+3 buffered.
			// We return X.
			// Next Read called. X+1 is missing. Buffer has X+2.
			// We are still in a "gap" state relative to X+1.
			// Does the timer reset?
			// "Timeout for stale detection". Usually means if we are stuck at a gap for too long.
			// If we make progress (received X), we should probably reset or extend the timer?
			// But if we are missing X+1, and we have X+2 since 10 minutes ago...
			// If we just got X, we are "making progress". So arguably the stream is alive.
			// Let's reset the timer if we successfully return a packet, OR if the buffer becomes empty.
			// If we return X, and buffer is not empty, it means we have future packets.
			// Logic: If we successfully processed a packet, we are not stale *yet* regarding flow.
			// However, if X+2 arrived 10m ago, and X just arrived, maybe X+1 is lost?
			// If X just arrived, we are moving.

			// I'll update firstBufferedAt to Now if buffer is still non-empty after finding a match.
			// This effectively gives "staleTimeout" duration to fill the *next* gap after a success.
			if len(r.buffer) > 0 {
				r.firstBufferedAt = time.Now()
			} else {
				r.firstBufferedAt = time.Time{}
			}
			return mote, nil
		} else if mote.SeqId > r.nextSeqID {
			// Gap detected
			if len(r.buffer) == 0 {
				r.firstBufferedAt = time.Now()
			}

			if _, exists := r.buffer[mote.SeqId]; !exists {
				r.buffer[mote.SeqId] = mote
			}

			if len(r.buffer) > r.maxBuffer {
				return nil, errors.New("stale stream: buffer limit exceeded")
			}

			// Check timeout again inside the loop as we might be receiving many out of order packets quickly
			if time.Since(r.firstBufferedAt) > r.staleTimeout {
				return nil, fmt.Errorf("stale stream: timeout waiting for seqID %d", r.nextSeqID)
			}

			// Continue reading to find the expected packet
			continue
		} else {
			// mote.SeqId < r.nextSeqID
			// Duplicate or old packet. Ignore.
			continue
		}
	}
}
