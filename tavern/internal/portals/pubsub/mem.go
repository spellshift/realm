package pubsub

import (
	"context"
	"fmt"
)

type InMemOption func(*memDriver)

func WithInMemoryDriver(options ...InMemOption) Option {
	return func(c *Client) {
		drv := &memDriver{}
		for _, opt := range options {
			opt(drv)
		}
		c.Driver = drv
	}
}

type memDriver struct{}

// EnsurePublisher creates and returns an in-memory Publisher for the specified topic.
func (drv *memDriver) EnsurePublisher(ctx context.Context, topic string) (Publisher, error) {
	return nil, fmt.Errorf("TODO")
}

// EnsureSubscriber creates and returns an in-memory Subscriber for the specified topic and subscription.
func (drv *memDriver) EnsureSubscriber(ctx context.Context, topic, subscription string) (Subscriber, error) {
	return nil, fmt.Errorf("TODO")
}
