package stream_test

import (
	"bytes"
	"context"
	"encoding/json"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gocloud.dev/pubsub"
	_ "gocloud.dev/pubsub/mempubsub"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/logging"
)

type syncLogBuffer struct {
	mu  sync.Mutex
	buf bytes.Buffer
}

func (b *syncLogBuffer) Write(p []byte) (int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.Write(p)
}

func (b *syncLogBuffer) String() string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.String()
}

func (b *syncLogBuffer) Reset() {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.buf.Reset()
}

func TestPreventPubSubColdStarts_ValidInterval(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	topicName := newTopicName("coldstart-valid")
	// Create a mock topic and subscription.
	topic, err := pubsub.OpenTopic(ctx, topicName)
	require.NoError(t, err)
	t.Cleanup(func() { _ = topic.Shutdown(context.Background()) })
	sub, err := pubsub.OpenSubscription(ctx, topicName)
	require.NoError(t, err)
	t.Cleanup(func() { _ = sub.Shutdown(context.Background()) })

	done := make(chan struct{})
	go func() {
		defer close(done)
		stream.PreventPubSubColdStarts(ctx, 50*time.Millisecond, topicName, topicName)
	}()
	t.Cleanup(func() {
		cancel()
		<-done
	})

	// Expect to receive a message
	msg, err := sub.Receive(ctx)
	assert.NoError(t, err)
	assert.NotNil(t, msg)
	if msg != nil {
		assert.Equal(t, "noop", msg.Metadata["id"])
		msg.Ack()
	}
}

func TestPreventPubSubColdStarts_ZeroInterval(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 1*time.Second)
	defer cancel()

	topicName := newTopicName("coldstart-zero")
	topic, err := pubsub.OpenTopic(ctx, topicName)
	require.NoError(t, err)
	t.Cleanup(func() { _ = topic.Shutdown(context.Background()) })
	sub, err := pubsub.OpenSubscription(ctx, topicName)
	require.NoError(t, err)
	t.Cleanup(func() { _ = sub.Shutdown(context.Background()) })

	done := make(chan struct{})
	go func() {
		defer close(done)
		stream.PreventPubSubColdStarts(ctx, 0, topicName, topicName)
	}()
	t.Cleanup(func() {
		cancel()
		<-done
	})

	// Expect to not receive a message and for the context to timeout
	_, err = sub.Receive(ctx)
	assert.Error(t, err)
	assert.Equal(t, context.DeadlineExceeded, err)
}

func TestPreventPubSubColdStarts_SubMillisecondInterval(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	topicName := newTopicName("coldstart-sub")
	topic, err := pubsub.OpenTopic(ctx, topicName)
	require.NoError(t, err)
	t.Cleanup(func() { _ = topic.Shutdown(context.Background()) })
	sub, err := pubsub.OpenSubscription(ctx, topicName)
	require.NoError(t, err)
	t.Cleanup(func() { _ = sub.Shutdown(context.Background()) })

	done := make(chan struct{})
	go func() {
		defer close(done)
		stream.PreventPubSubColdStarts(ctx, 1*time.Microsecond, topicName, topicName)
	}()
	t.Cleanup(func() {
		cancel()
		<-done
	})

	// Expect to receive a message
	msg, err := sub.Receive(ctx)
	assert.NoError(t, err)
	assert.NotNil(t, msg)
	if msg != nil {
		assert.Equal(t, "noop", msg.Metadata["id"])
		msg.Ack()
	}
}

func TestPreventPubSubColdStarts_StructuredErrorLogging(t *testing.T) {
	logBuf := &syncLogBuffer{}
	require.NoError(t, logging.Configure(logging.Options{
		Spec:   "shells.pubsub=debug",
		Format: logging.FormatJSON,
		Output: logBuf,
	}))
	t.Cleanup(func() {
		_ = logging.Configure(logging.Options{Spec: "dev"})
	})

	ctx := context.Background()

	// Case 1: Invalid output topic URL logs structured "topic" (output topic) and "error" without !BADKEY or %q/%v
	stream.PreventPubSubColdStarts(ctx, 10*time.Millisecond, "invalidscheme://bad-output-topic", newTopicName("coldstart-valid-in"))
	rawOut := strings.TrimSpace(logBuf.String())
	require.NotEmpty(t, rawOut)
	assert.NotContains(t, rawOut, "!BADKEY")
	assert.NotContains(t, rawOut, "%q")
	assert.NotContains(t, rawOut, "%v")

	var outRec map[string]any
	require.NoError(t, json.Unmarshal([]byte(strings.Split(rawOut, "\n")[0]), &outRec))
	assert.Equal(t, "shells.pubsub", outRec["subsystem"])
	assert.Equal(t, "invalidscheme://bad-output-topic", outRec["topic"])
	assert.NotEmpty(t, outRec["error"])

	// Case 2: Valid output topic + invalid input topic logs topicShellInput (NOT topicShellOutput!)
	validOutName := newTopicName("coldstart-valid-out")
	validOutTopic, err := pubsub.OpenTopic(ctx, validOutName)
	require.NoError(t, err)
	t.Cleanup(func() { _ = validOutTopic.Shutdown(context.Background()) })

	logBuf.Reset()
	stream.PreventPubSubColdStarts(ctx, 10*time.Millisecond, validOutName, "invalidscheme://bad-input-topic")
	rawIn := strings.TrimSpace(logBuf.String())
	require.NotEmpty(t, rawIn)
	assert.NotContains(t, rawIn, "!BADKEY")
	assert.NotContains(t, rawIn, "%q")
	assert.NotContains(t, rawIn, "%v")

	var inRec map[string]any
	require.NoError(t, json.Unmarshal([]byte(strings.Split(rawIn, "\n")[0]), &inRec))
	assert.Equal(t, "shells.pubsub", inRec["subsystem"])
	assert.Equal(t, "invalidscheme://bad-input-topic", inRec["topic"], "input topic failure must log topicShellInput, not topicShellOutput")
	assert.NotEmpty(t, inRec["error"])
}
