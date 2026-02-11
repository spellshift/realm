package builder

import (
	"context"
	"crypto/ed25519"
	"crypto/x509"
	"encoding/pem"
	"fmt"
	"log/slog"
	"strings"
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

	// Initial ping to verify connectivity
	_, err = client.Ping(ctx, &builderpb.PingRequest{})
	if err != nil {
		return fmt.Errorf("failed to ping upstream: %w", err)
	}

	slog.InfoContext(ctx, "successfully pinged upstream", "upstream", cfg.Upstream)

	// Check for tasks immediately, then poll on interval
	if err := claimAndExecuteTasks(ctx, client, exec); err != nil {
		slog.ErrorContext(ctx, "error processing build tasks", "error", err)
	}

	ticker := time.NewTicker(taskPollInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-ticker.C:
			if err := claimAndExecuteTasks(ctx, client, exec); err != nil {
				slog.ErrorContext(ctx, "error processing build tasks", "error", err)
			}
		}
	}
}

func claimAndExecuteTasks(ctx context.Context, client builderpb.BuilderClient, exec executor.Executor) error {
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

		executeTask(ctx, client, exec, task)
	}

	return nil
}

// executeTask runs a single build task through the executor, streaming output
// and errors back to the server, then reports completion.
func executeTask(ctx context.Context, client builderpb.BuilderClient, exec executor.Executor, task *builderpb.BuildTaskSpec) {
	outputCh := make(chan string, 64)
	errorCh := make(chan string, 64)

	var outputLines []string
	var errorLines []string

	// Collect output and errors in a background goroutine.
	done := make(chan struct{})
	go func() {
		defer close(done)
		for {
			select {
			case line, ok := <-outputCh:
				if !ok {
					outputCh = nil
					if errorCh == nil {
						return
					}
					continue
				}
				outputLines = append(outputLines, line)
				slog.InfoContext(ctx, "build output",
					"task_id", task.Id,
					"line", line,
				)
			case line, ok := <-errorCh:
				if !ok {
					errorCh = nil
					if outputCh == nil {
						return
					}
					continue
				}
				errorLines = append(errorLines, line)
				slog.WarnContext(ctx, "build error output",
					"task_id", task.Id,
					"line", line,
				)
			}
		}
	}()

	// Run the build through the executor.
	buildErr := exec.Build(ctx, executor.BuildSpec{
		TaskID:      task.Id,
		TargetOS:    task.TargetOs,
		BuildImage:  task.BuildImage,
		BuildScript: task.BuildScript,
	}, outputCh, errorCh)

	// Close channels to signal collector goroutine to finish.
	close(outputCh)
	close(errorCh)
	<-done

	// Build the submission request with collected output.
	submitReq := &builderpb.SubmitBuildTaskOutputRequest{
		TaskId: task.Id,
		Output: strings.Join(outputLines, "\n"),
	}

	if buildErr != nil {
		errMsg := buildErr.Error()
		if len(errorLines) > 0 {
			errMsg = strings.Join(errorLines, "\n") + "\n" + errMsg
		}
		submitReq.Error = errMsg
		slog.ErrorContext(ctx, "build task failed",
			"task_id", task.Id,
			"error", buildErr,
		)
	} else if len(errorLines) > 0 {
		submitReq.Error = strings.Join(errorLines, "\n")
	}

	if _, err := client.SubmitBuildTaskOutput(ctx, submitReq); err != nil {
		slog.ErrorContext(ctx, "failed to submit build task output",
			"task_id", task.Id,
			"error", err,
		)
	}
}
