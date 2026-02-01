package stream

import (
	"context"
	"log/slog"
	"time"

	"realm.pub/tavern/internal/xpubsub"
)

// PreventPubSubColdStarts by publishing noop messages on an interval.
// This reduces cold-start latency for GCP PubSub which can improve shell user experience.
// In other environments, this functionality may not be necessary.
func PreventPubSubColdStarts(ctx context.Context, client *xpubsub.Client, interval time.Duration, topicShellOutput string, topicShellInput string) {
	if interval == 0 {
		slog.WarnContext(ctx, "gcppubsub cold-start polling disabled due to 0ms interval, this may impact shell latency")
		return
	}
	if interval < 1*time.Millisecond {
		slog.WarnContext(ctx, "gcppubsub cold-start polling interval less than minimum, setting to 1 millisecond")
		interval = 1 * time.Millisecond
	}

	pubOutput := client.NewPublisher(topicShellOutput)
	pubInput := client.NewPublisher(topicShellInput)

	// Since NewPublisher doesn't return error (it just creates the wrapper), we assume it succeeds.
	// But actually we might want to ensure topics exist?
	// The original code used pubsub.OpenTopic which checks URL but doesn't necessarily connect/check existence until Send.
	// We'll proceed.

	ticker := time.NewTicker(interval)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if err := pubOutput.Publish(ctx, []byte{0}, map[string]string{"id": "noop"}); err != nil {
				slog.ErrorContext(ctx, "warmup failed to publish shell output keep-alive no op. GCP coldstart latency may be encountered.", "error", err)
			}
			if err := pubInput.Publish(ctx, []byte{0}, map[string]string{"id": "noop"}); err != nil {
				slog.ErrorContext(ctx, "warmup failed to publish shell input keep-alive no op. GCP coldstart latency may be encountered.", "error", err)
			}
		}
	}
}
