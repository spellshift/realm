package builder

import (
	"context"
	"crypto/ed25519"
	"crypto/x509"
	"encoding/pem"
	"fmt"
	"log/slog"
	"sync"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"

	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/builder/executor"
)

const (
	// taskPollInterval is how often the builder polls for new build tasks.
	taskPollInterval = 5 * time.Second

	// maxConcurrentBuilds is the maximum number of builds that can run simultaneously.
	maxConcurrentBuilds = 4
)

// builderCredentials implements grpc.PerRPCCredentials for mTLS authentication.
type builderCredentials struct {
	certDER []byte
	privKey ed25519.PrivateKey
}

// GetRequestMetadata generates fresh authentication metadata for each RPC call.
// It signs the current timestamp with the builder's private key to prove possession.
// Binary metadata (keys ending in "-bin") is automatically base64 encoded by gRPC.
func (c *builderCredentials) GetRequestMetadata(ctx context.Context, uri ...string) (map[string]string, error) {
	timestamp := time.Now().UTC().Format(time.RFC3339Nano)

	sig := ed25519.Sign(c.privKey, []byte(timestamp))

	return map[string]string{
		mdKeyBuilderCert:      string(c.certDER),
		mdKeyBuilderSignature: string(sig),
		mdKeyBuilderTimestamp: timestamp,
	}, nil
}

// RequireTransportSecurity returns false since Tavern uses h2c (HTTP/2 cleartext)
// with TLS terminated at the reverse proxy level.
func (c *builderCredentials) RequireTransportSecurity() bool {
	return false
}

// NewCredentialsFromConfig creates gRPC per-RPC credentials from a builder config.
func NewCredentialsFromConfig(cfg *Config) (credentials.PerRPCCredentials, error) {
	return parseMTLSCredentials(cfg.MTLS)
}

// parseMTLSCredentials loads the certificate and private key from the config's
// PEM bundle string.
func parseMTLSCredentials(mtlsPEM string) (*builderCredentials, error) {
	pemBundle := []byte(mtlsPEM)

	var certDER []byte
	var privKey ed25519.PrivateKey

	for {
		block, rest := pem.Decode(pemBundle)
		if block == nil {
			break
		}
		switch block.Type {
		case "CERTIFICATE":
			certDER = block.Bytes
		case "PRIVATE KEY":
			key, err := x509.ParsePKCS8PrivateKey(block.Bytes)
			if err != nil {
				return nil, fmt.Errorf("failed to parse private key: %w", err)
			}
			edKey, ok := key.(ed25519.PrivateKey)
			if !ok {
				return nil, fmt.Errorf("private key is not ED25519")
			}
			privKey = edKey
		}
		pemBundle = rest
	}

	if certDER == nil {
		return nil, fmt.Errorf("no certificate found in mTLS bundle")
	}
	if privKey == nil {
		return nil, fmt.Errorf("no private key found in mTLS bundle")
	}

	return &builderCredentials{
		certDER: certDER,
		privKey: privKey,
	}, nil
}

// Run starts the builder process using the provided configuration.
// It connects to the configured upstream server with mTLS credentials,
// then enters a polling loop to claim and execute build tasks using the
// provided executor.
func Run(ctx context.Context, cfg *Config, exec executor.Executor) error {
	slog.InfoContext(ctx, "builder started",
		"id", cfg.ID,
		"supported_targets", cfg.SupportedTargets,
		"upstream", cfg.Upstream,
	)

	creds, err := parseMTLSCredentials(cfg.MTLS)
	if err != nil {
		return fmt.Errorf("failed to parse mTLS credentials: %w", err)
	}

	conn, err := grpc.NewClient(cfg.Upstream,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithPerRPCCredentials(creds),
	)
	if err != nil {
		return fmt.Errorf("failed to connect to upstream %q: %w", cfg.Upstream, err)
	}
	defer conn.Close()

	client := builderpb.NewBuilderClient(conn)

	// Semaphore limits concurrent builds.
	sem := make(chan struct{}, maxConcurrentBuilds)
	var wg sync.WaitGroup

	// Check for tasks immediately, then poll on interval
	if err := claimAndExecuteTasks(ctx, client, exec, sem, &wg); err != nil {
		slog.ErrorContext(ctx, "error processing build tasks", "error", err)
	}

	ticker := time.NewTicker(taskPollInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			wg.Wait()
			return ctx.Err()
		case <-ticker.C:
			if err := claimAndExecuteTasks(ctx, client, exec, sem, &wg); err != nil {
				slog.ErrorContext(ctx, "error processing build tasks", "error", err)
			}
		}
	}
}

func claimAndExecuteTasks(ctx context.Context, client builderpb.BuilderClient, exec executor.Executor, sem chan struct{}, wg *sync.WaitGroup) error {
	resp, err := client.ClaimBuildTasks(ctx, &builderpb.ClaimBuildTasksRequest{})
	if err != nil {
		return fmt.Errorf("failed to claim build tasks: %w", err)
	}

	for _, task := range resp.Tasks {
		slog.InfoContext(ctx, "claimed build task",
			"task_id", task.Id,
			"target_os", task.TargetOs,
			"build_image", task.BuildImage,
		)

		// Acquire semaphore slot (blocks if at max concurrency).
		sem <- struct{}{}
		wg.Add(1)
		go func(t *builderpb.BuildTaskSpec) {
			defer wg.Done()
			defer func() { <-sem }()
			executeTask(ctx, client, exec, t)
		}(task)
	}

	return nil
}

// executeTask runs a single build task through the executor, streaming output
// and errors back to the server via the StreamBuildTaskOutput RPC.
func executeTask(ctx context.Context, client builderpb.BuilderClient, exec executor.Executor, task *builderpb.BuildTaskSpec) {
	stream, err := client.StreamBuildTaskOutput(ctx)
	if err != nil {
		slog.ErrorContext(ctx, "failed to open build output stream",
			"task_id", task.Id, "error", err)
		return
	}

	outputCh := make(chan string, 64)
	errorCh := make(chan string, 64)

	// Stream output and errors to the server in a background goroutine.
	var sendErr error
	collectorDone := make(chan struct{})
	go func() {
		defer close(collectorDone)
		outCh, errCh := outputCh, errorCh
		for outCh != nil || errCh != nil {
			select {
			case line, ok := <-outCh:
				if !ok {
					outCh = nil
					continue
				}
				if err := stream.Send(&builderpb.StreamBuildTaskOutputRequest{
					TaskId: task.Id,
					Output: line,
				}); err != nil {
					sendErr = err
					return
				}
				slog.InfoContext(ctx, "build output",
					"task_id", task.Id, "line", line)
			case line, ok := <-errCh:
				if !ok {
					errCh = nil
					continue
				}
				if err := stream.Send(&builderpb.StreamBuildTaskOutputRequest{
					TaskId: task.Id,
					Error:  line,
				}); err != nil {
					sendErr = err
					return
				}
				slog.WarnContext(ctx, "build error output",
					"task_id", task.Id, "line", line)
			}
		}
	}()

	// Run the build through the executor.
	// The executor closes both channels when done.
	result, buildErr := exec.Build(ctx, executor.BuildSpec{
		TaskID:       task.Id,
		TargetOS:     task.TargetOs,
		BuildImage:   task.BuildImage,
		BuildScript:  task.BuildScript,
		ArtifactPath: task.ArtifactPath,
		Env:          task.Env,
	}, outputCh, errorCh)

	// Wait for the collector goroutine to drain remaining channel data.
	<-collectorDone

	if sendErr != nil {
		slog.ErrorContext(ctx, "stream send failed during build",
			"task_id", task.Id, "error", sendErr)
		return
	}

	// Send the final message signaling completion.
	finalMsg := &builderpb.StreamBuildTaskOutputRequest{
		TaskId:   task.Id,
		Finished: true,
	}
	if buildErr != nil {
		finalMsg.Error = buildErr.Error()
		slog.ErrorContext(ctx, "build task failed",
			"task_id", task.Id, "error", buildErr)
	}

	if err := stream.Send(finalMsg); err != nil {
		slog.ErrorContext(ctx, "failed to send final stream message",
			"task_id", task.Id, "error", err)
		return
	}

	if _, err := stream.CloseAndRecv(); err != nil {
		slog.ErrorContext(ctx, "failed to close build output stream",
			"task_id", task.Id, "error", err)
		return
	}

	// Upload artifact if the build succeeded and produced one.
	if buildErr == nil && result != nil && result.Artifact != nil {
		if err := uploadArtifact(ctx, client, task.Id, result); err != nil {
			slog.ErrorContext(ctx, "failed to upload build artifact",
				"task_id", task.Id, "error", err)
		}
	}
}

// uploadArtifact streams the build artifact to the server in chunks via the
// UploadBuildArtifact RPC.
func uploadArtifact(ctx context.Context, client builderpb.BuilderClient, taskID int64, result *executor.BuildResult) error {
	stream, err := client.UploadBuildArtifact(ctx)
	if err != nil {
		return fmt.Errorf("failed to open artifact upload stream: %w", err)
	}

	const chunkSize = 1 * 1024 * 1024 // 1MB chunks
	data := result.Artifact

	for i := 0; i < len(data); i += chunkSize {
		end := i + chunkSize
		if end > len(data) {
			end = len(data)
		}

		msg := &builderpb.UploadBuildArtifactRequest{
			Chunk: data[i:end],
		}
		// Set metadata only on the first message.
		if i == 0 {
			msg.TaskId = taskID
			msg.ArtifactName = result.ArtifactName
		}

		if err := stream.Send(msg); err != nil {
			return fmt.Errorf("failed to send artifact chunk: %w", err)
		}
	}

	resp, err := stream.CloseAndRecv()
	if err != nil {
		return fmt.Errorf("failed to close artifact stream: %w", err)
	}

	slog.InfoContext(ctx, "artifact uploaded",
		"task_id", taskID,
		"asset_id", resp.AssetId,
		"size", len(data),
	)

	return nil
}
