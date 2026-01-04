package events

import (
	"context"
	"sync"

	"realm.pub/tavern/internal/ent"
)

// HostCreatedBroadcaster manages subscriptions for host created events
type HostCreatedBroadcaster struct {
	mu          sync.RWMutex
	subscribers map[chan *ent.Host]struct{}
}

// NewHostCreatedBroadcaster creates a new broadcaster for host created events
func NewHostCreatedBroadcaster() *HostCreatedBroadcaster {
	return &HostCreatedBroadcaster{
		subscribers: make(map[chan *ent.Host]struct{}),
	}
}

// Subscribe returns a channel that will receive host created events
func (b *HostCreatedBroadcaster) Subscribe(ctx context.Context) <-chan *ent.Host {
	ch := make(chan *ent.Host, 10) // buffered to avoid blocking

	b.mu.Lock()
	b.subscribers[ch] = struct{}{}
	b.mu.Unlock()

	// Remove subscriber when context is cancelled
	go func() {
		<-ctx.Done()
		b.mu.Lock()
		delete(b.subscribers, ch)
		b.mu.Unlock()
		close(ch)
	}()

	return ch
}

// Broadcast sends a host created event to all subscribers
func (b *HostCreatedBroadcaster) Broadcast(host *ent.Host) {
	b.mu.RLock()
	defer b.mu.RUnlock()

	for ch := range b.subscribers {
		// Non-blocking send
		select {
		case ch <- host:
		default:
			// Skip if channel is full
		}
	}
}
