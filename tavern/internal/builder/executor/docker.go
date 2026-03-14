package executor

import (
	"archive/tar"
	"bufio"
	"bytes"
	"compress/gzip"
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

	// Prepare the local tmp dir with /scripts and /tomes.
	tmpDir, err := prepareMountDir(spec)
	if err != nil {
		if tmpDir != "" {
			os.RemoveAll(tmpDir)
		}
		return nil, fmt.Errorf("failed to prepare mount dir: %w", err)
	}
	defer os.RemoveAll(tmpDir)

	slog.InfoContext(ctx, "creating container", "image", spec.BuildImage, "task_id", spec.TaskID)

	// The container entrypoint runs all scripts in /mnt/scripts in order.
	entrypoint := strings.Join([]string{
		"set -e",
		"for s in $(ls /mnt/scripts/*.sh 2>/dev/null | sort); do echo \"==> Running $s\"; sh \"$s\"; done",
	}, " && ")

	resp, err := d.client.ContainerCreate(ctx,
		&container.Config{
			Image:      spec.BuildImage,
			Entrypoint: []string{"/bin/sh", "-c", entrypoint},
			Env:        spec.Env,
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

	// Copy the prepared tmp dir contents into /mnt inside the container.
	if err := d.copyDirToContainer(ctx, containerID, tmpDir, "/mnt"); err != nil {
		return nil, fmt.Errorf("failed to copy build dir to container: %w", err)
	}

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

// copyDirToContainer creates a tar archive from a local directory and copies
// it into the container at the specified path. The directory contents are placed
// directly under destPath (i.e. the top-level dir itself is not nested).
func (d *DockerExecutor) copyDirToContainer(ctx context.Context, containerID, localDir, destPath string) error {
	var buf bytes.Buffer
	tw := tar.NewWriter(&buf)

	baseDir := filepath.Clean(localDir)
	err := filepath.Walk(baseDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		relPath, err := filepath.Rel(baseDir, path)
		if err != nil {
			return err
		}
		if relPath == "." {
			return nil
		}

		header, err := tar.FileInfoHeader(info, "")
		if err != nil {
			return err
		}
		header.Name = relPath

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
		return fmt.Errorf("walking local dir %q: %w", localDir, err)
	}

	if err := tw.Close(); err != nil {
		return fmt.Errorf("closing tar writer: %w", err)
	}

	return d.client.CopyToContainer(ctx, containerID, destPath, &buf, container.CopyToContainerOptions{})
}

// prepareMountDir creates a temporary directory with /scripts and /tomes
// subdirectories populated from the BuildSpec. It writes the pre-build,
// build, and post-build scripts to numbered files under /scripts so they
// execute in order. Returns the tmp dir path (caller must clean up).
func prepareMountDir(spec BuildSpec) (string, error) {
	tmpDir, err := os.MkdirTemp("", "realm-build-*")
	if err != nil {
		return "", fmt.Errorf("creating temp dir: %w", err)
	}

	scriptsDir := filepath.Join(tmpDir, "scripts")
	if err := os.MkdirAll(scriptsDir, 0o755); err != nil {
		return tmpDir, fmt.Errorf("creating scripts dir: %w", err)
	}

	tomesDir := filepath.Join(tmpDir, "tomes")
	if err := os.MkdirAll(tomesDir, 0o755); err != nil {
		return tmpDir, fmt.Errorf("creating tomes dir: %w", err)
	}

	// Write setup script.
	if spec.SetupScript != "" {
		if err := os.WriteFile(filepath.Join(scriptsDir, "0_setup.sh"), []byte(spec.SetupScript), 0o755); err != nil {
			return tmpDir, fmt.Errorf("writing setup script: %w", err)
		}
	}
 
	// Write pre-build script.
	if spec.PreBuildScript != "" {
		if err := os.WriteFile(filepath.Join(scriptsDir, "1_pre_build.sh"), []byte(spec.PreBuildScript), 0o755); err != nil {
			return tmpDir, fmt.Errorf("writing pre-build script: %w", err)
		}
	}

	// Write build script.
	if spec.BuildScript != "" {
		if err := os.WriteFile(filepath.Join(scriptsDir, "4_build.sh"), []byte(spec.BuildScript), 0o755); err != nil {
			return tmpDir, fmt.Errorf("writing build script: %w", err)
		}
	}

	// Write post-build script.
	if spec.PostBuildScript != "" {
		if err := os.WriteFile(filepath.Join(scriptsDir, "9_post_build.sh"), []byte(spec.PostBuildScript), 0o755); err != nil {
			return tmpDir, fmt.Errorf("writing post-build script: %w", err)
		}
	}

	// Copy tomes from source directory if provided.
	if spec.TomesDir != "" {
		err := filepath.Walk(spec.TomesDir, func(path string, info os.FileInfo, err error) error {
			if err != nil {
				return err
			}
			relPath, err := filepath.Rel(spec.TomesDir, path)
			if err != nil {
				return err
			}
			destPath := filepath.Join(tomesDir, relPath)
			if info.IsDir() {
				return os.MkdirAll(destPath, info.Mode())
			}
			data, err := os.ReadFile(path)
			if err != nil {
				return err
			}
			return os.WriteFile(destPath, data, info.Mode())
		})
		if err != nil {
			return tmpDir, fmt.Errorf("copying tomes dir: %w", err)
		}
	}

	// Extract downloaded tome tar.gz archives into per-tome subdirectories.
	for _, t := range spec.Tomes {
		tomeDir := filepath.Join(tomesDir, fmt.Sprintf("%d", t.ID))
		if err := os.MkdirAll(tomeDir, 0o755); err != nil {
			return tmpDir, fmt.Errorf("creating tome dir %d: %w", t.ID, err)
		}

		if err := extractTomeArchive(t.Contents, tomeDir); err != nil {
			return tmpDir, fmt.Errorf("extracting tome %d: %w", t.ID, err)
		}

		// Write params as a JSON file if present.
		if t.Params != "" {
			if err := os.WriteFile(filepath.Join(tomeDir, "params.json"), []byte(t.Params), 0o644); err != nil {
				return tmpDir, fmt.Errorf("writing params for tome %d: %w", t.ID, err)
			}
		}
	}

	return tmpDir, nil
}

// extractTomeArchive decompresses a tar.gz archive and extracts all regular
// files into destDir, preserving their path names and creating subdirectories
// as needed.
func extractTomeArchive(data []byte, destDir string) error {
	gr, err := gzip.NewReader(bytes.NewReader(data))
	if err != nil {
		return fmt.Errorf("opening gzip reader: %w", err)
	}
	defer gr.Close()

	tr := tar.NewReader(gr)
	for {
		hdr, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return fmt.Errorf("reading tar entry: %w", err)
		}
		if hdr.Typeflag != tar.TypeReg {
			continue
		}

		destPath := filepath.Join(destDir, hdr.Name)

		// Create parent directories for nested asset paths.
		if dir := filepath.Dir(destPath); dir != destDir {
			if err := os.MkdirAll(dir, 0o755); err != nil {
				return fmt.Errorf("creating dir for %s: %w", hdr.Name, err)
			}
		}

		content, err := io.ReadAll(tr)
		if err != nil {
			return fmt.Errorf("reading %s: %w", hdr.Name, err)
		}
		if err := os.WriteFile(destPath, content, 0o644); err != nil {
			return fmt.Errorf("writing %s: %w", hdr.Name, err)
		}
	}

	return nil
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
