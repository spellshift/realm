package stream

import (
	"sync"
)

// CircularBuffer is a fixed-size byte buffer that overwrites old data when full.
// It is safe for concurrent use.
type CircularBuffer struct {
	mu   sync.Mutex
	buf  []byte
	size int
}

// NewCircularBuffer creates a new circular buffer with the given size.
func NewCircularBuffer(size int) *CircularBuffer {
	return &CircularBuffer{
		buf:  make([]byte, 0, size),
		size: size,
	}
}

// Write appends data to the buffer.
func (cb *CircularBuffer) Write(p []byte) {
	cb.mu.Lock()
	defer cb.mu.Unlock()

	n := len(p)
	if n > cb.size {
		// If data is larger than buffer, just take the last size bytes
		cb.buf = append([]byte(nil), p[n-cb.size:]...)
		return
	}

	// Simpler approach for "history log":
	// append to slice. if len > size, take suffix.
	cb.buf = append(cb.buf, p...)
	if len(cb.buf) > cb.size {
		cb.buf = cb.buf[len(cb.buf)-cb.size:]
	}
}

// Bytes returns the current content of the buffer.
func (cb *CircularBuffer) Bytes() []byte {
	cb.mu.Lock()
	defer cb.mu.Unlock()
	// Return a copy
	out := make([]byte, len(cb.buf))
	copy(out, cb.buf)
	return out
}
