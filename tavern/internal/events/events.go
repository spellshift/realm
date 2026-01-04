package events

import (
	"context"
	"sync"

	"realm.pub/tavern/internal/ent"
)

// EventBusContextKey is the context key for the event bus
type contextKey string

const EventBusContextKey contextKey = "eventBus"

// EventType represents the type of entity lifecycle event
type EventType string

const (
	HostCreatedEvent EventType = "host.created"
	HostUpdatedEvent EventType = "host.updated"
)

// Event represents an entity lifecycle event
type Event struct {
	Type EventType
	Host *ent.Host
}

// Subscriber is a function that handles events
type Subscriber func(ctx context.Context, event Event)

// Bus manages event publishing and subscription
type Bus struct {
	mu          sync.RWMutex
	subscribers map[EventType][]Subscriber
	ctx         context.Context
	cancel      context.CancelFunc
}

// NewBus creates a new event bus
func NewBus(ctx context.Context) *Bus {
	ctx, cancel := context.WithCancel(ctx)
	return &Bus{
		subscribers: make(map[EventType][]Subscriber),
		ctx:         ctx,
		cancel:      cancel,
	}
}

// Subscribe registers a subscriber for a specific event type
func (b *Bus) Subscribe(eventType EventType, subscriber Subscriber) {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.subscribers[eventType] = append(b.subscribers[eventType], subscriber)
}

// Publish publishes an event to all subscribers
func (b *Bus) Publish(event Event) {
	b.mu.RLock()
	subscribers := b.subscribers[event.Type]
	b.mu.RUnlock()

	// Execute subscribers in separate goroutines to avoid blocking
	for _, subscriber := range subscribers {
		sub := subscriber // capture for goroutine
		go func() {
			defer func() {
				if r := recover(); r != nil {
					// Log panic but don't crash
				}
			}()
			sub(b.ctx, event)
		}()
	}
}

// Close shuts down the event bus
func (b *Bus) Close() error {
	b.cancel()
	return nil
}

// WithEventBus adds the event bus to the context
func WithEventBus(ctx context.Context, bus *Bus) context.Context {
	return context.WithValue(ctx, EventBusContextKey, bus)
}
