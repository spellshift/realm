package stream

import (
	"context"
	"log/slog"
	"time"

	"gocloud.dev/pubsub"
	_ "gocloud.dev/pubsub/gcppubsub"
)

// PreventPubSubColdStarts by publishing noop messages on an interval.
// This reduces cold-start latency for GCP PubSub which can improve shell user experience.
// In other environments, this functionality may not be necessary.
func PreventPubSubColdStarts(ctx context.Context, interval time.Duration, topicShellOutput string, topicShellInput string) {
	if interval == 0 {
		slog.WarnContext(ctx, "gcppubsub cold-start polling disabled due to 0ms interval, this may impact shell latency")
		return
	}
	if interval < 1*time.Millisecond {
		slog.WarnContext(ctx, "gcppubsub cold-start polling interval less than minimum, setting to 1 millisecond")
		interval = 1 * time.Millisecond
	}

	pubOutput, err := pubsub.OpenTopic(ctx, topicShellOutput)
	if err != nil {
		slog.ErrorContext(ctx, "warmup failed to connect to pubsub output topic, cold-start latency may impact user experience (%q): %v", topicShellOutput, err)
		return
	}
	pubInput, err := pubsub.OpenTopic(ctx, topicShellInput)
	if err != nil {
		slog.ErrorContext(ctx, "warmup failed to connect to pubsub input topic, cold-start latency may impact user experience (%q): %v", topicShellOutput, err)
		return
	}

	ticker := time.NewTicker(interval)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if err := pubOutput.Send(ctx, &pubsub.Message{
				Metadata: map[string]string{
					"id": "noop",
				},
				Body: []byte{0},
			}); err != nil {
				slog.ErrorContext(ctx, "warmup failed to publish shell output keep-alive no op. GCP coldstart latency may be encountered.")
			}
			if err := pubInput.Send(ctx, &pubsub.Message{
				Metadata: map[string]string{
					"id": "noop",
				},
				Body: []byte{0},
			}); err != nil {
				slog.ErrorContext(ctx, "warmup failed to publish shell input keep-alive no op. GCP coldstart latency may be encountered.")
			}
		}
	}
}
