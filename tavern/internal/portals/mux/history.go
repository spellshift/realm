package mux

import (
	"sync"

	"realm.pub/tavern/portals/portalpb"
)

// HistoryBuffer is a circular buffer for storing recent messages.
type HistoryBuffer struct {
	messages []*portalpb.Mote
	capacity int
	head     int
	mutex    sync.RWMutex
}

// NewHistoryBuffer creates a new history buffer with the given capacity.
func NewHistoryBuffer(capacity int) *HistoryBuffer {
	if capacity <= 0 {
		capacity = 100 // Default
	}
	return &HistoryBuffer{
		messages: make([]*portalpb.Mote, 0, capacity),
		capacity: capacity,
	}
}

// Add adds a message to the buffer.
func (h *HistoryBuffer) Add(msg *portalpb.Mote) {
	h.mutex.Lock()
	defer h.mutex.Unlock()

	if len(h.messages) < h.capacity {
		h.messages = append(h.messages, msg)
	} else {
		h.messages[h.head] = msg
		h.head = (h.head + 1) % h.capacity
	}
}

// Get returns all messages in the buffer in order.
func (h *HistoryBuffer) Get() []*portalpb.Mote {
	h.mutex.RLock()
	defer h.mutex.RUnlock()

	result := make([]*portalpb.Mote, 0, len(h.messages))
	if len(h.messages) < h.capacity {
		result = append(result, h.messages...)
	} else {
		result = append(result, h.messages[h.head:]...)
		result = append(result, h.messages[:h.head]...)
	}
	return result
}
