package executor

import (
	"archive/tar"
	"bufio"
	"context"
	"fmt"
	"io"
	"log/slog"
	"os"
	"path/filepath"
	"strings"

	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/api/types/image"
	"github.com/docker/docker/client"
	"github.com/docker/docker/pkg/stdcopy"
)

// DockerExecutor runs build tasks inside Docker containers.
// It pulls the specified image, creates a container using the build script
// as the entrypoint, and streams stdout/stderr over the provided channels.
type DockerExecutor struct {
	client client.APIClient
}

// NewDockerExecutor creates a DockerExecutor using the provided Docker API client.
func NewDockerExecutor(cli client.APIClient) *DockerExecutor {
	return &DockerExecutor{client: cli}
}

// NewDockerExecutorFromEnv creates a DockerExecutor using the default
// Docker client configuration from environment variables.
func NewDockerExecutorFromEnv(ctx context.Context) (*DockerExecutor, error) {
	cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())
	if err != nil {
		return nil, fmt.Errorf("failed to create docker client: %w", err)
	}
	return &DockerExecutor{client: cli}, nil
}

// Build pulls the build image, starts a container with the build script as
// the shell entrypoint, and streams output/error lines over the channels.
// After the container exits successfully, if spec.ArtifactPath is set, the
// artifact is copied from the stopped container before removal.
func (d *DockerExecutor) Build(ctx context.Context, spec BuildSpec, outputCh chan<- string, errorCh chan<- string) (*BuildResult, error) {
	defer close(outputCh)
	defer close(errorCh)

	slog.InfoContext(ctx, "pulling docker image", "image", spec.BuildImage, "task_id", spec.TaskID)

	pullReader, err := d.client.ImagePull(ctx, spec.BuildImage, image.PullOptions{})
	if err != nil {
		return nil, fmt.Errorf("failed to pull image %q: %w", spec.BuildImage, err)
	}
	// Drain and close the pull output to ensure the image is fully downloaded.
	if _, err := io.Copy(io.Discard, pullReader); err != nil {
		pullReader.Close()
		return nil, fmt.Errorf("error reading image pull output: %w", err)
	}
	pullReader.Close()

	slog.InfoContext(ctx, "creating container", "image", spec.BuildImage, "task_id", spec.TaskID)

	// 1. Prepare local volumes for tomes and scripts.
	baseDir := fmt.Sprintf("/tmp/builder-%d", spec.TaskID)
	tomesDir := filepath.Join(baseDir, "tomes")
	scriptsDir := filepath.Join(baseDir, "scripts")

	// Ensure cleanup after container exits.
	defer func() {
		if err := os.RemoveAll(baseDir); err != nil {
			slog.Warn("failed to cleanup local builder directory", "dir", baseDir, "error", err)
		}
	}()

	if err := os.MkdirAll(tomesDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create tomes directory: %w", err)
	}
	if err := os.MkdirAll(scriptsDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create scripts directory: %w", err)
	}

	// 2. Write tomes to local volume.
	for _, tome := range spec.Tomes {
		tomePath := filepath.Join(tomesDir, tome.Name)
		if err := os.MkdirAll(tomePath, 0755); err != nil {
			return nil, fmt.Errorf("failed to create tome directory %s: %w", tomePath, err)
		}

		// Construct and write main.eldritch.
		var tomeScript string
		tomeScript += "input_params = {\n"
		for k, v := range tome.Params {
			tomeScript += fmt.Sprintf("  %q: %q,\n", k, v)
		}
		tomeScript += "}\n"
		tomeScript += tome.Script

		if err := os.WriteFile(filepath.Join(tomePath, "main.eldritch"), []byte(tomeScript), 0644); err != nil {
			return nil, fmt.Errorf("failed to write tome %s script: %w", tome.Name, err)
		}

		// Write assets.
		for _, asset := range tome.Assets {
			assetPath := filepath.Join(tomePath, asset.Name)
			if err := os.MkdirAll(filepath.Dir(assetPath), 0755); err != nil {
				return nil, fmt.Errorf("failed to create asset parent directory %s: %w", filepath.Dir(assetPath), err)
			}
			if err := os.WriteFile(assetPath, asset.Content, 0644); err != nil {
				return nil, fmt.Errorf("failed to write asset %s: %w", assetPath, err)
			}
		}
	}

	// 3. Write pre, main, and post build scripts.
	if spec.PreBuildScript != "" {
		if err := os.WriteFile(filepath.Join(scriptsDir, "0_pre_build.sh"), []byte(spec.PreBuildScript), 0755); err != nil {
			return nil, fmt.Errorf("failed to write pre-build script: %w", err)
		}
	}

	// Adjust the BuildScript to use the mounted tomes.
	// We know BuildScript looks like: `cd /home/vscode && git clone ... && cd realm/implants/imix && cargo build ...`
	// We want to delete `install_scripts` and symlink/copy the tomes directory BEFORE the cargo build.
	// A simple string replace ensures it runs right before cargo.
	parts := strings.Split(spec.BuildScript, " && cargo ")
	var mainScript string
	if len(parts) == 2 {
		setupCmd := parts[0]
		buildCmd := "cargo " + parts[1]

		mainScript += "set -e\n"
		mainScript += setupCmd + "\n"
		mainScript += `rm -rf install_scripts` + "\n"
		mainScript += `mkdir -p install_scripts` + "\n"
		mainScript += `cp -r /build_tomes/. install_scripts/ || true` + "\n"
		mainScript += buildCmd + "\n"
	} else {
		mainScript = spec.BuildScript
	}

	if err := os.WriteFile(filepath.Join(scriptsDir, "1_build.sh"), []byte(mainScript), 0755); err != nil {
		return nil, fmt.Errorf("failed to write main build script: %w", err)
	}

	if spec.PostBuildScript != "" {
		if err := os.WriteFile(filepath.Join(scriptsDir, "9_post_build.sh"), []byte(spec.PostBuildScript), 0755); err != nil {
			return nil, fmt.Errorf("failed to write post-build script: %w", err)
		}
	}

	// 4. Create container with mounted volumes and custom entrypoint.
	// We use `sh -c` to execute all scripts in sequence.
	entrypointCmd := `set -e; for f in /build_scripts/*.sh; do if [ -f "$f" ]; then sh "$f" || exit 1; fi; done`

	hostConfig := &container.HostConfig{
		Binds: []string{
			fmt.Sprintf("%s:/build_tomes:ro", tomesDir),
			fmt.Sprintf("%s:/build_scripts:ro", scriptsDir),
		},
	}

	resp, err := d.client.ContainerCreate(ctx,
		&container.Config{
			Image:      spec.BuildImage,
			Entrypoint: []string{"/bin/sh", "-c", entrypointCmd},
			Env:        spec.Env,
		},
		hostConfig, // host config
		nil, // networking config
		nil, // platform
		"",  // container name (auto-generated)
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create container: %w", err)
	}
	containerID := resp.ID

	// Ensure the container is removed when we're done.
	defer func() {
		removeErr := d.client.ContainerRemove(context.Background(), containerID, container.RemoveOptions{Force: true})
		if removeErr != nil {
			slog.Warn("failed to remove container", "container_id", containerID, "error", removeErr)
		}
	}()

	if err := d.client.ContainerStart(ctx, containerID, container.StartOptions{}); err != nil {
		return nil, fmt.Errorf("failed to start container: %w", err)
	}

	slog.InfoContext(ctx, "container started", "container_id", containerID, "task_id", spec.TaskID)

	// Attach to container logs to stream stdout and stderr.
	logReader, err := d.client.ContainerLogs(ctx, containerID, container.LogsOptions{
		ShowStdout: true,
		ShowStderr: true,
		Follow:     true,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to attach to container logs: %w", err)
	}
	defer logReader.Close()

	// Docker multiplexes stdout/stderr into a single stream with headers.
	// stdcopy.StdCopy demultiplexes them.
	stdoutPR, stdoutPW := io.Pipe()
	stderrPR, stderrPW := io.Pipe()

	go func() {
		_, err := stdcopy.StdCopy(stdoutPW, stderrPW, logReader)
		stdoutPW.CloseWithError(err)
		stderrPW.CloseWithError(err)
	}()

	// Stream stderr lines over errorCh in a background goroutine.
	done := make(chan struct{})
	go func() {
		defer close(done)
		scanner := bufio.NewScanner(stderrPR)
		for scanner.Scan() {
			errorCh <- scanner.Text()
		}
	}()

	scanner := bufio.NewScanner(stdoutPR)
	for scanner.Scan() {
		outputCh <- scanner.Text()
	}

	// Wait for stderr goroutine to finish.
	<-done

	// Wait for the container to exit and check its status.
	statusCh, errCh := d.client.ContainerWait(ctx, containerID, container.WaitConditionNotRunning)
	var exitCode int64
	select {
	case err := <-errCh:
		if err != nil {
			return nil, fmt.Errorf("error waiting for container: %w", err)
		}
	case result := <-statusCh:
		exitCode = result.StatusCode
	case <-ctx.Done():
		return nil, ctx.Err()
	}

	// Extract artifact from the stopped container (before deferred removal).
	buildResult := BuildResult{ExitCode: exitCode}

	if exitCode != ExpectedExitCode {
		return &buildResult, fmt.Errorf("container exited with status %d", exitCode)
	}

	if spec.ArtifactPath == "" {
		return &buildResult, nil
	}

	data, name, extractErr := d.extractArtifact(ctx, containerID, spec.ArtifactPath)
	if extractErr != nil {
		slog.WarnContext(ctx, "artifact extraction failed",
			"task_id", spec.TaskID, "path", spec.ArtifactPath, "error", extractErr)
		return &buildResult, nil
	}

	buildResult.Artifact = data
	buildResult.ArtifactName = name
	slog.InfoContext(ctx, "artifact extracted",
		"task_id", spec.TaskID, "name", name, "size", len(data))

	return &buildResult, nil
}

// extractArtifact copies a file from a stopped container using the Docker API.
// CopyFromContainer returns a tar archive; this method extracts the first
// regular file from that archive and returns its contents and basename.
func (d *DockerExecutor) extractArtifact(ctx context.Context, containerID, path string) ([]byte, string, error) {
	tarReader, _, err := d.client.CopyFromContainer(ctx, containerID, path)
	if err != nil {
		return nil, "", fmt.Errorf("CopyFromContainer %q: %w", path, err)
	}
	defer tarReader.Close()

	tr := tar.NewReader(tarReader)
	for {
		hdr, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return nil, "", fmt.Errorf("reading tar entry: %w", err)
		}
		if hdr.Typeflag != tar.TypeReg {
			continue
		}
		data, err := io.ReadAll(tr)
		if err != nil {
			return nil, "", fmt.Errorf("reading artifact data: %w", err)
		}
		return data, filepath.Base(hdr.Name), nil
	}

	return nil, "", fmt.Errorf("no regular file found at %q", path)
}
