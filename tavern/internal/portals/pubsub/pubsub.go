package pubsub

import (
	"context"

	"realm.pub/tavern/portals/portalpb"
)

// Publisher defines the interface for publishing motes.
type Publisher interface {
	Publish(ctx context.Context, mote *portalpb.Mote) error
}

// Subscriber defines the interface for receiving motes.
type Subscriber interface {
	Receive(ctx context.Context, f func(context.Context, *portalpb.Mote)) error
}

// A Driver provides an implementation for sending and receiving motes.
type Driver interface {
	EnsurePublisher(ctx context.Context, topic string) (Publisher, error)
	EnsureSubscriber(ctx context.Context, topic, subscription string) (Subscriber, error)
}

type Option func(*Client)

type Client struct {
	Driver
}

func NewClient(options ...Option) *Client {
	c := &Client{
		Driver: &memDriver{},
	}
	for _, opt := range options {
		opt(c)
	}
	return c
}
