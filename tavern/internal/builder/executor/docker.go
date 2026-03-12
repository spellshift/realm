package executor

import (
	"archive/tar"
	"bufio"
	"bytes"
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

	"realm.pub/tavern/internal/builder/builderpb"
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
		tomeScript := ConstructTomeScript(tome)

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
		mainScript += `cp -a /build_tomes install_scripts` + "\n"
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

	// 4. Create container and copy files into it.
	// We use CopyToContainer instead of bind mounts so that this works in
	// Docker-in-Docker environments (e.g. devcontainers) where host paths
	// visible to the Docker daemon differ from the paths inside the outer container.
	entrypointCmd := `set -e; for f in /build_scripts/*.sh; do if [ -f "$f" ]; then sh "$f"; fi; done`

	resp, err := d.client.ContainerCreate(ctx,
		&container.Config{
			Image:      spec.BuildImage,
			Entrypoint: []string{"/bin/sh", "-c", entrypointCmd},
			Env:        append(spec.Env, fmt.Sprintf("ARTIFACT_PATH=%s", spec.ArtifactPath)),
		},
		nil, // host config
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

	// Copy tomes and scripts into the container as tar archives.
	if err := copyDirToContainer(ctx, d.client, containerID, tomesDir, "/build_tomes"); err != nil {
		return nil, fmt.Errorf("failed to copy tomes into container: %w", err)
	}
	if err := copyDirToContainer(ctx, d.client, containerID, scriptsDir, "/build_scripts"); err != nil {
		return nil, fmt.Errorf("failed to copy scripts into container: %w", err)
	}

	if err := d.client.ContainerStart(ctx, containerID, container.StartOptions{}); err != nil {
		return nil, fmt.Errorf("failed to start container: %w", err)
	}

	slog.InfoContext(ctx, "container started", "container_id", containerID, "task_id", spec.TaskID)
	slog.DebugContext(ctx, "container config",
		"task_id", spec.TaskID,
		"entrypoint", entrypointCmd,
		"env", spec.Env,
		"artifact_path", spec.ArtifactPath,
	)

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
	slog.DebugContext(ctx, "waiting for container to exit", "container_id", containerID, "task_id", spec.TaskID)
	statusCh, errCh := d.client.ContainerWait(ctx, containerID, container.WaitConditionNotRunning)
	var exitCode int64
	select {
	case err := <-errCh:
		if err != nil {
			return nil, fmt.Errorf("error waiting for container: %w", err)
		}
	case result := <-statusCh:
		exitCode = result.StatusCode
		if result.Error != nil {
			slog.WarnContext(ctx, "container wait returned error message",
				"task_id", spec.TaskID, "error_message", result.Error.Message)
		}
	case <-ctx.Done():
		return nil, ctx.Err()
	}

	slog.InfoContext(ctx, "container exited", "container_id", containerID, "task_id", spec.TaskID, "exit_code", exitCode)

	// Inspect the container to get its final state for debugging.
	inspectResp, inspectErr := d.client.ContainerInspect(ctx, containerID)
	if inspectErr != nil {
		slog.WarnContext(ctx, "failed to inspect container", "task_id", spec.TaskID, "error", inspectErr)
	} else {
		slog.DebugContext(ctx, "container inspect",
			"task_id", spec.TaskID,
			"state", inspectResp.State.Status,
			"exit_code", inspectResp.State.ExitCode,
			"error", inspectResp.State.Error,
			"started_at", inspectResp.State.StartedAt,
			"finished_at", inspectResp.State.FinishedAt,
		)
	}

	// Extract artifact from the stopped container (before deferred removal).
	buildResult := BuildResult{ExitCode: exitCode}

	if exitCode != ExpectedExitCode {
		return &buildResult, fmt.Errorf("container exited with status %d", exitCode)
	}

	if spec.ArtifactPath == "" {
		slog.DebugContext(ctx, "no artifact path specified, skipping extraction", "task_id", spec.TaskID)
		return &buildResult, nil
	}

	slog.DebugContext(ctx, "attempting artifact extraction",
		"task_id", spec.TaskID, "container_id", containerID, "artifact_path", spec.ArtifactPath)
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

// ConstructTomeScript generates the full eldritch script for a tome, prepending
// the input_params dictionary populated with the tome's parameters.
func ConstructTomeScript(tome *builderpb.BuildTaskTome) string {
	var tomeScript string
	tomeScript += "input_params = {\n"
	for k, v := range tome.Params {
		tomeScript += fmt.Sprintf("  %q: %q,\n", k, v)
	}
	tomeScript += "}\n"
	tomeScript += tome.Script
	return tomeScript
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

// copyDirToContainer creates a tar archive from a local directory and copies
// it into a container at the specified path. This works in Docker-in-Docker
// environments where bind mounts would reference host paths instead of the
// outer container's filesystem.
func copyDirToContainer(ctx context.Context, cli client.APIClient, containerID, srcDir, dstPath string) error {
	var buf bytes.Buffer
	tw := tar.NewWriter(&buf)

	// Use the destination directory name as the tar prefix so we can copy
	// to "/" and have Docker create the directory automatically.
	prefix := filepath.Base(dstPath)

	err := filepath.Walk(srcDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		rel, err := filepath.Rel(srcDir, path)
		if err != nil {
			return err
		}

		header, err := tar.FileInfoHeader(info, "")
		if err != nil {
			return err
		}
		header.Name = filepath.Join(prefix, rel)

		if err := tw.WriteHeader(header); err != nil {
			return err
		}

		if info.IsDir() {
			return nil
		}

		f, err := os.Open(path)
		if err != nil {
			return err
		}
		defer f.Close()
		_, err = io.Copy(tw, f)
		return err
	})
	if err != nil {
		tw.Close()
		return fmt.Errorf("building tar archive from %s: %w", srcDir, err)
	}
	if err := tw.Close(); err != nil {
		return fmt.Errorf("closing tar writer: %w", err)
	}

	return cli.CopyToContainer(ctx, containerID, "/", &buf, container.CopyToContainerOptions{})
}
