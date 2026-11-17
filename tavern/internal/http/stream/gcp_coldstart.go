package stream

import (
	"context"
	"time"

	"gocloud.dev/pubsub"
	_ "gocloud.dev/pubsub/gcppubsub"
	"realm.pub/tavern/internal/logging"
)

var pubsubLog = logging.For("shells.pubsub")

// PreventPubSubColdStarts by publishing noop messages on an interval.
// This reduces cold-start latency for GCP PubSub which can improve shell user experience.
// In other environments, this functionality may not be necessary.
func PreventPubSubColdStarts(ctx context.Context, interval time.Duration, topicShellOutput string, topicShellInput string) {
	if interval == 0 {
		pubsubLog.WarnContext(ctx, "gcppubsub cold-start polling disabled due to 0ms interval, this may impact shell latency")
		return
	}
	if interval < 1*time.Millisecond {
		pubsubLog.WarnContext(ctx, "gcppubsub cold-start polling interval less than minimum, setting to 1 millisecond")
		interval = 1 * time.Millisecond
	}

	pubOutput, err := pubsub.OpenTopic(ctx, topicShellOutput)
	if err != nil {
		pubsubLog.ErrorContext(ctx, "warmup failed to connect to pubsub output topic, cold-start latency may impact user experience",
			"topic", topicShellOutput,
			"error", err,
		)
		return
	}
	defer func() { _ = pubOutput.Shutdown(context.Background()) }()

	pubInput, err := pubsub.OpenTopic(ctx, topicShellInput)
	if err != nil {
		pubsubLog.ErrorContext(ctx, "warmup failed to connect to pubsub input topic, cold-start latency may impact user experience",
			"topic", topicShellInput,
			"error", err,
		)
		return
	}
	defer func() { _ = pubInput.Shutdown(context.Background()) }()

	ticker := time.NewTicker(interval)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			pubsubLog.TraceContext(ctx, "publishing cold-start keep-alive noop", "topic", topicShellOutput)
			if err := pubOutput.Send(ctx, &pubsub.Message{
				Metadata: map[string]string{
					"id": "noop",
				},
				Body: []byte{0},
			}); err != nil {
				if ctx.Err() != nil {
					return
				}
				pubsubLog.ErrorContext(ctx, "warmup failed to publish shell output keep-alive no op. GCP coldstart latency may be encountered.",
					"topic", topicShellOutput,
					"error", err,
				)
			}
			pubsubLog.TraceContext(ctx, "publishing cold-start keep-alive noop", "topic", topicShellInput)
			if err := pubInput.Send(ctx, &pubsub.Message{
				Metadata: map[string]string{
					"id": "noop",
				},
				Body: []byte{0},
			}); err != nil {
				if ctx.Err() != nil {
					return
				}
				pubsubLog.ErrorContext(ctx, "warmup failed to publish shell input keep-alive no op. GCP coldstart latency may be encountered.",
					"topic", topicShellInput,
					"error", err,
				)
			}
		}
	}
}
