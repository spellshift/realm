package stream

import (
	"sync"
)

// CircularBuffer is a fixed-size byte buffer that overwrites old data when full.
// It is safe for concurrent use.
type CircularBuffer struct {
	mu     sync.Mutex
	data   []byte
	size   int
	start  int
	length int
}

// NewCircularBuffer creates a new circular buffer with the given size.
func NewCircularBuffer(size int) *CircularBuffer {
	return &CircularBuffer{
		data:   make([]byte, size),
		size:   size,
		start:  0,
		length: 0,
	}
}

// Write appends data to the buffer.
func (cb *CircularBuffer) Write(p []byte) {
	cb.mu.Lock()
	defer cb.mu.Unlock()

	n := len(p)
	if n == 0 {
		return
	}

	// If the data being written is larger than the buffer size,
	// we only care about the last `size` bytes.
	if n >= cb.size {
		copy(cb.data, p[n-cb.size:])
		cb.start = 0
		cb.length = cb.size
		return
	}

	// We are writing n bytes.
	// We write starting at (start + length) % size.
	writeStart := (cb.start + cb.length) % cb.size

	// Check if the write wraps around the end of the buffer
	if writeStart+n <= cb.size {
		// Contiguous write
		copy(cb.data[writeStart:], p)
	} else {
		// Wrapped write
		chunk1 := cb.size - writeStart
		copy(cb.data[writeStart:], p[:chunk1])
		copy(cb.data[0:], p[chunk1:])
	}

	// Update length and start
	if cb.length+n <= cb.size {
		cb.length += n
	} else {
		// Buffer overflowed
		overflow := (cb.length + n) - cb.size
		cb.start = (cb.start + overflow) % cb.size
		cb.length = cb.size
	}
}

// Bytes returns the current content of the buffer.
func (cb *CircularBuffer) Bytes() []byte {
	cb.mu.Lock()
	defer cb.mu.Unlock()

	out := make([]byte, cb.length)
	if cb.length == 0 {
		return out
	}

	// If the data is contiguous
	if cb.start+cb.length <= cb.size {
		copy(out, cb.data[cb.start:cb.start+cb.length])
	} else {
		// Data wraps around
		chunk1 := cb.size - cb.start
		copy(out, cb.data[cb.start:])
		copy(out[chunk1:], cb.data[:cb.length-chunk1])
	}
	return out
}
