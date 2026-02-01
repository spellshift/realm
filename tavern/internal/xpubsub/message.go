package xpubsub

// Message represents a Pub/Sub message.
type Message struct {
	Body       []byte
	Metadata   map[string]string
	LoggableID string
	Ack        func()
	Nack       func()
}
