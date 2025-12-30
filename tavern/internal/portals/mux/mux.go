package mux

import (
	"context"
	"fmt"
	"sync"
	"time"

	gcppubsub "cloud.google.com/go/pubsub"
	"github.com/google/uuid"
	"github.com/prometheus/client_golang/prometheus"
	"gocloud.dev/pubsub"
	_ "gocloud.dev/pubsub/gcppubsub"
	_ "gocloud.dev/pubsub/mempubsub"
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
)

func init() {
	prometheus.MustRegister(msgsPublished)
	prometheus.MustRegister(msgsReceived)
}

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

// Mux is the central router between the global PubSub mesh and local gRPC streams.
type Mux struct {
	serverID    string
	useInMem    bool
	gcpClient   *gcppubsub.Client
	projectID   string
	historySize int

	subscribers sync.RWMutex
	subs        map[string][]chan *portalpb.Mote

	activeSubs sync.RWMutex
	active     map[string]*pubsub.Subscription
	subRefs    map[string]int

	history sync.RWMutex
	hist    map[string]*HistoryBuffer

	// Cache for open topics
	topicsMu sync.RWMutex
	topics   map[string]*pubsub.Topic
}

// Option defines a functional option for Mux configuration.
type Option func(*Mux)

// WithHistorySize sets the size of the history buffer for each topic.
func WithHistorySize(size int) Option {
	return func(m *Mux) {
		m.historySize = size
	}
}

// WithInMemoryDriver configures the Mux to use the in-memory driver.
func WithInMemoryDriver() Option {
	return func(m *Mux) {
		m.useInMem = true
		m.gcpClient = nil
		m.projectID = ""
	}
}

// WithGCPDriver configures the Mux to use the GCP PubSub driver.
func WithGCPDriver(projectID string, client *gcppubsub.Client) Option {
	return func(m *Mux) {
		m.useInMem = false
		m.projectID = projectID
		m.gcpClient = client
	}
}

// New creates a new Mux with the given options.
func New(opts ...Option) *Mux {
	m := &Mux{
		serverID:    uuid.New().String(),
		subs:        make(map[string][]chan *portalpb.Mote),
		active:      make(map[string]*pubsub.Subscription),
		subRefs:     make(map[string]int),
		hist:        make(map[string]*HistoryBuffer),
		topics:      make(map[string]*pubsub.Topic),
		historySize: 100, // Default
	}

	for _, opt := range opts {
		opt(m)
	}

	return m
}

// Naming & URL Construction Helpers

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

// TopicURL returns the URL for the topic based on the configured driver.
func (m *Mux) TopicURL(name string) string {
	if m.useInMem {
		return "mem://" + name
	}
	return fmt.Sprintf("gcppubsub://projects/%s/topics/%s", m.projectID, name)
}

// SubURL returns the URL for the subscription based on the configured driver.
func (m *Mux) SubURL(topicID, subID string) string {
	if m.useInMem {
		return "mem://" + topicID
	}
	return fmt.Sprintf("gcppubsub://projects/%s/subscriptions/%s", m.projectID, subID)
}

// getTopic returns a cached topic handle or opens a new one.
func (m *Mux) getTopic(ctx context.Context, topicID string) (*pubsub.Topic, error) {
	m.topicsMu.RLock()
	t, ok := m.topics[topicID]
	m.topicsMu.RUnlock()
	if ok {
		return t, nil
	}

	m.topicsMu.Lock()
	defer m.topicsMu.Unlock()
	// Double check
	if t, ok := m.topics[topicID]; ok {
		return t, nil
	}

	t, err := pubsub.OpenTopic(ctx, m.TopicURL(topicID))
	if err != nil {
		return nil, err
	}
	m.topics[topicID] = t
	return t, nil
}

// ensureTopic ensures that the topic exists.
func (m *Mux) ensureTopic(ctx context.Context, topicID string) error {
	if m.useInMem {
		// Open and cache it
		_, err := m.getTopic(ctx, topicID)
		return err
	}
	if m.gcpClient == nil {
		return fmt.Errorf("gcp client is nil")
	}

	topic := m.gcpClient.Topic(topicID)
	exists, err := topic.Exists(ctx)
	if err != nil {
		return fmt.Errorf("failed to check topic existence: %w", err)
	}
	if !exists {
		_, err = m.gcpClient.CreateTopic(ctx, topicID)
		if err != nil {
			return fmt.Errorf("failed to create topic: %w", err)
		}
	}
	return nil
}

// ensureSub ensures that the subscription exists.
func (m *Mux) ensureSub(ctx context.Context, topicID, subID string) error {
	if m.useInMem {
		// Ensure topic exists first
		return m.ensureTopic(ctx, topicID)
	}

	if m.gcpClient == nil {
		return fmt.Errorf("gcp client is nil")
	}

	sub := m.gcpClient.Subscription(subID)
	exists, err := sub.Exists(ctx)
	if err != nil {
		return fmt.Errorf("failed to check subscription existence: %w", err)
	}

	if !exists {
		topic := m.gcpClient.Topic(topicID)
		cfg := gcppubsub.SubscriptionConfig{
			Topic:            topic,
			ExpirationPolicy: 24 * time.Hour,
		}
		_, err = m.gcpClient.CreateSubscription(ctx, subID, cfg)
		if err != nil {
			return fmt.Errorf("failed to create subscription: %w", err)
		}
	}
	return nil
}
