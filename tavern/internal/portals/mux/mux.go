package mux

import (
	"context"
	"fmt"
	"sync"
	"time"

	gcppubsub "cloud.google.com/go/pubsub/v2"
	"cloud.google.com/go/pubsub/v2/apiv1/pubsubpb"
	"github.com/google/uuid"
	"github.com/prometheus/client_golang/prometheus"
	"google.golang.org/protobuf/types/known/durationpb"
	"gocloud.dev/pubsub"
	_ "gocloud.dev/pubsub/gcppubsub"
	_ "gocloud.dev/pubsub/mempubsub"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
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
	active      map[string]*pubsub.Subscription
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
	topics map[string]*pubsub.Topic
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
	gcpClient *gcppubsub.Client
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
		serverID: uuid.New().String(),
		subs: &SubscriberRegistry{
			subs:       make(map[string][]chan *portalpb.Mote),
			bufferSize: 1024, // Default
		},
		subMgr: &SubscriptionManager{
			active:      make(map[string]*pubsub.Subscription),
			refs:        make(map[string]int),
			cancelFuncs: make(map[string]context.CancelFunc),
		},
		history: &HistoryManager{
			buffers: make(map[string]*HistoryBuffer),
			size:    1024, // Default
		},
		topics: &TopicManager{
			topics: make(map[string]*pubsub.Topic),
		},
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
	m.topics.RLock()
	t, ok := m.topics.topics[topicID]
	m.topics.RUnlock()
	if ok {
		return t, nil
	}

	m.topics.Lock()
	defer m.topics.Unlock()
	// Double check
	if t, ok := m.topics.topics[topicID]; ok {
		return t, nil
	}

	t, err := pubsub.OpenTopic(ctx, m.TopicURL(topicID))
	if err != nil {
		return nil, err
	}
	m.topics.topics[topicID] = t
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

	fullTopicName := fmt.Sprintf("projects/%s/topics/%s", m.projectID, topicID)
	_, err := m.gcpClient.TopicAdminClient.GetTopic(ctx, &pubsubpb.GetTopicRequest{Topic: fullTopicName})
	if err == nil {
		return nil
	}
	if status.Code(err) != codes.NotFound {
		return fmt.Errorf("failed to check topic existence: %w", err)
	}

	_, err = m.gcpClient.TopicAdminClient.CreateTopic(ctx, &pubsubpb.Topic{Name: fullTopicName})
	if err != nil {
		// Check for AlreadyExists error to handle race conditions
		if status.Code(err) == codes.AlreadyExists {
			return nil
		}
		return fmt.Errorf("failed to create topic: %w", err)
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

	fullSubName := fmt.Sprintf("projects/%s/subscriptions/%s", m.projectID, subID)
	_, err := m.gcpClient.SubscriptionAdminClient.GetSubscription(ctx, &pubsubpb.GetSubscriptionRequest{Subscription: fullSubName})
	if err == nil {
		return nil
	}
	if status.Code(err) != codes.NotFound {
		return fmt.Errorf("failed to check subscription existence: %w", err)
	}

	fullTopicName := fmt.Sprintf("projects/%s/topics/%s", m.projectID, topicID)
	_, err = m.gcpClient.SubscriptionAdminClient.CreateSubscription(ctx, &pubsubpb.Subscription{
		Name:  fullSubName,
		Topic: fullTopicName,
		ExpirationPolicy: &pubsubpb.ExpirationPolicy{
			Ttl: durationpb.New(24 * time.Hour),
		},
	})
	if err != nil {
		// Check for AlreadyExists error to handle race conditions
		if status.Code(err) == codes.AlreadyExists {
			return nil
		}
		return fmt.Errorf("failed to create subscription: %w", err)
	}
	return nil
}
