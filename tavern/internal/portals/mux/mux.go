package mux

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/google/uuid"
	"github.com/prometheus/client_golang/prometheus"
	"realm.pub/tavern/internal/xpubsub"
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
	active      map[string]xpubsub.Subscriber
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
	topics map[string]xpubsub.Publisher
}

// HistoryManager manages message history buffers.
type HistoryManager struct {
	sync.RWMutex
	buffers map[string]*HistoryBuffer
	size    int
}

// Mux is the central router between the global PubSub mesh and local gRPC streams.
type Mux struct {
	serverID  string
	useInMem  bool
	client    *xpubsub.Client
	projectID string

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

// WithInMemoryDriver configures the Mux to use the in-memory driver.
func WithInMemoryDriver() Option {
	return func(m *Mux) {
		m.useInMem = true
		// client init moved to New() or we lazy init?
		// New() handles it
		m.projectID = "test-project"
	}
}

// WithGCPDriver configures the Mux to use the GCP PubSub driver.
func WithGCPDriver(projectID string) Option {
	return func(m *Mux) {
		m.useInMem = false
		m.projectID = projectID
	}
}

// New creates a new Mux with the given options.
func New(ctx context.Context, opts ...Option) (*Mux, error) {
	m := &Mux{
		serverID: uuid.New().String(),
		subs: &SubscriberRegistry{
			subs:       make(map[string][]chan *portalpb.Mote),
			bufferSize: 1024, // Default
		},
		subMgr: &SubscriptionManager{
			active:      make(map[string]xpubsub.Subscriber),
			refs:        make(map[string]int),
			cancelFuncs: make(map[string]context.CancelFunc),
		},
		history: &HistoryManager{
			buffers: make(map[string]*HistoryBuffer),
			size:    1024, // Default
		},
		topics: &TopicManager{
			topics: make(map[string]xpubsub.Publisher),
		},
	}

	for _, opt := range opts {
		opt(m)
	}

	// Initialize Client
	client, err := xpubsub.NewClient(ctx, m.projectID, m.useInMem)
	if err != nil {
		return nil, fmt.Errorf("failed to create xpubsub client: %w", err)
	}
	m.client = client

	return m, nil
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
	return m.client.EnsureTopic(ctx, topicID)
}

// ensureSub ensures that the subscription exists.
func (m *Mux) ensureSub(ctx context.Context, topicID, subID string) error {
	// 24 hour TTL
	return m.client.EnsureSubscription(ctx, topicID, subID, 24*time.Hour)
}

func (m *Mux) Close() error {
	return m.client.Close()
}
