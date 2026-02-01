package mux

import (
	"context"
	"fmt"
	"sync"

	"github.com/google/uuid"
	"github.com/prometheus/client_golang/prometheus"
	"realm.pub/tavern/internal/portals/pubsub"
	"realm.pub/tavern/portals/portalpb"
)

var (
	msgsPublished = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "mux_messages_published_total",
			Help: "Total number of messages published via Mux",
		},
		[]string{"topic", "status"},
	)
	msgsReceived = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "mux_messages_received_total",
			Help: "Total number of messages received via Mux",
		},
		[]string{"topic"},
	)
	msgsDropped = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "mux_messages_dropped_total",
			Help: "Total number of messages dropped due to full buffer",
		},
		[]string{"topic"},
	)
)

func init() {
	prometheus.MustRegister(msgsPublished)
	prometheus.MustRegister(msgsReceived)
	prometheus.MustRegister(msgsDropped)
}

// SubscriptionManager manages active PubSub subscriptions.
type SubscriptionManager struct {
	sync.RWMutex
	active      map[string]pubsub.Subscriber
	refs        map[string]int
	cancelFuncs map[string]context.CancelFunc
}

// SubscriberRegistry manages local channel subscribers.
type SubscriberRegistry struct {
	sync.RWMutex
	subs       map[string][]chan *portalpb.Mote
	bufferSize int
}

// TopicManager manages cached PubSub topics.
type TopicManager struct {
	sync.RWMutex
	topics map[string]pubsub.Publisher
}

// HistoryManager manages message history buffers.
type HistoryManager struct {
	sync.RWMutex
	buffers map[string]*HistoryBuffer
	size    int
}

// Mux is the central router between the global PubSub mesh and local gRPC streams.
type Mux struct {
	serverID string
	pubsub   *pubsub.Client

	subs    *SubscriberRegistry
	subMgr  *SubscriptionManager
	history *HistoryManager
	topics  *TopicManager
}

// Option defines a functional option for Mux configuration.
type Option func(*Mux)

// WithHistorySize sets the size of the history buffer for each topic.
func WithHistorySize(size int) Option {
	return func(m *Mux) {
		m.history.size = size
	}
}

// WithSubscriberBufferSize sets the buffer size for subscriber channels.
func WithSubscriberBufferSize(size int) Option {
	return func(m *Mux) {
		m.subs.bufferSize = size
	}
}

// WithPubSubClient configures the Mux to use the provided PubSub client.
func WithPubSubClient(client *pubsub.Client) Option {
	return func(m *Mux) {
		m.pubsub = client
	}
}

// New creates a new Mux with the given options.
func New(opts ...Option) *Mux {
	m := &Mux{
		serverID: uuid.New().String(),
		subs: &SubscriberRegistry{
			subs:       make(map[string][]chan *portalpb.Mote),
			bufferSize: 15625, // Default
		},
		subMgr: &SubscriptionManager{
			active:      make(map[string]pubsub.Subscriber),
			refs:        make(map[string]int),
			cancelFuncs: make(map[string]context.CancelFunc),
		},
		history: &HistoryManager{
			buffers: make(map[string]*HistoryBuffer),
			size:    1024, // Default
		},
		topics: &TopicManager{
			topics: make(map[string]pubsub.Publisher),
		},
	}

	for _, opt := range opts {
		opt(m)
	}

	// Default to in-memory if no client provided (useful for tests that don't pass client)
	if m.pubsub == nil {
		m.pubsub = pubsub.NewClient(pubsub.WithInMemoryDriver())
	}

	return m
}

// Naming Helpers

// TopicIn returns the topic name for incoming messages to the portal.
func (m *Mux) TopicIn(portalID int) string {
	return fmt.Sprintf("PORTAL_IN_%d", portalID)
}

// TopicOut returns the topic name for outgoing messages from the portal.
func (m *Mux) TopicOut(portalID int) string {
	return fmt.Sprintf("PORTAL_OUT_%d", portalID)
}

// SubName returns the subscription name for a given topic and this server.
func (m *Mux) SubName(topicID string) string {
	return fmt.Sprintf("%s_SUB_%s", topicID, m.serverID)
}

// ensureTopic ensures that the topic exists.
func (m *Mux) ensureTopic(ctx context.Context, topicID string) error {
	// The new pubsub.EnsurePublisher handles creation/existence checks.
	// We can choose to cache it here or let getTopic handle caching later.
	// Since ensureTopic is often called during setup, we can just ensure it exists.
	_, err := m.pubsub.EnsurePublisher(ctx, topicID)
	return err
}

// ensureSub ensures that the subscription exists.
func (m *Mux) ensureSub(ctx context.Context, topicID, subID string) error {
	// The new pubsub.EnsureSubscriber handles creation/existence checks.
	// Note: We need to pass the topicID to EnsureSubscriber as it might need to create the subscription bound to the topic.
	_, err := m.pubsub.EnsureSubscriber(ctx, topicID, subID)
	return err
}
