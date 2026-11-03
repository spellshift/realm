package executor

import (
	"archive/tar"
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"log/slog"
	"os"
	"path"
	"path/filepath"
	"slices"
	"strings"
	"sync"

	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/api/types/image"
	"github.com/docker/docker/api/types/mount"
	"github.com/docker/docker/api/types/volume"
	"github.com/docker/docker/client"
	"github.com/docker/docker/pkg/stdcopy"
)

// DockerExecutor runs build tasks inside Docker containers.
// It pulls the specified image, creates a container using the build script
// as the entrypoint, and streams stdout/stderr over the provided channels.
type DockerExecutor struct {
	client client.APIClient

	// cacheMu serializes builds that share the same cache volume + key so that
	// concurrent cargo/go builds don't race inside a shared target directory
	// (cargo does not support concurrent builds into the same target dir).
	// The lock is per-builder-process; see lockFor.
	cacheMu sync.Map // map[string]*sync.Mutex
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

	// Configure build caching. When spec.CacheVolume is set, ensure the
	// Docker volume exists and mount it into the container, pointing the
	// cargo/go cache directories at it. This persists downloaded dependencies
	// and intermediate build artifacts across builds so repeated builds don't
	// rebuild the entire dependency tree from scratch.
	hostConfig := (*container.HostConfig)(nil)
	cacheEnv := []string{}
	unlock := func() {}
	if spec.CacheVolume != "" {
		cachePath := spec.CachePath
		if cachePath == "" {
			cachePath = DefaultCachePath
		}
		if _, err := d.client.VolumeCreate(ctx, volume.CreateOptions{
			Name:   spec.CacheVolume,
			Driver: "local",
			Labels: map[string]string{"realm.buildcache": "true"},
		}); err != nil {
			// Volume creation can fail (e.g. storage driver limits, no
			// permission). Degrade gracefully: build without caching rather
			// than failing the whole build.
			slog.WarnContext(ctx, "failed to create build cache volume, building without cache",
				"task_id", spec.TaskID, "volume", spec.CacheVolume, "error", err)
		} else {
			hostConfig = &container.HostConfig{
				Mounts: []mount.Mount{
					{
						Type:   mount.TypeVolume,
						Source: spec.CacheVolume,
						Target: cachePath,
					},
				},
			}
			key := cacheKeyFor(spec)
			cacheEnv = []string{
				"REALM_CACHE_DIR=" + cachePath,
				"CARGO_HOME=" + path.Join(cachePath, "cargo"),
				"CARGO_TARGET_DIR=" + path.Join(cachePath, "target", key),
				"GOPATH=" + path.Join(cachePath, "gopath"),
				"GOMODCACHE=" + path.Join(cachePath, "gopath", "pkg", "mod"),
				"GOCACHE=" + path.Join(cachePath, "go-cache"),
			}
			// Serialize builds sharing a cache key so concurrent cargo
			// invocations don't race inside the shared target directory.
			unlock = d.lockFor(spec.CacheVolume + "/" + key)
		}
	}
	defer unlock()

	slog.InfoContext(ctx, "creating container", "image", spec.BuildImage, "task_id", spec.TaskID)

	// The container entrypoint runs all scripts in /mnt/scripts in order.
	entrypoint := strings.Join([]string{
		"set -e",
		"for s in $(ls /mnt/scripts/*.sh 2>/dev/null | sort); do echo \"==> Running $s\"; . \"$s\"; done",
	}, " && ")

	resp, err := d.client.ContainerCreate(ctx,
		&container.Config{
			Image:      spec.BuildImage,
			Entrypoint: []string{"/bin/sh", "-c", entrypoint},
			Env: append(append(slices.Clone(spec.Env),
				"REALM_WORKSPACE_DIR=/mnt",
				"REALM_TOMES_DIR=/mnt/tomes",
			), cacheEnv...),
		},
		hostConfig, // host config (may mount the build cache volume)
		nil,        // networking config
		nil,        // platform
		"",         // container name (auto-generated)
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
		streamBuildLines(ctx, stderrPR, errorCh)
	}()

	streamBuildLines(ctx, stdoutPR, outputCh)

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
		// Build already reported ExpectedExitCode, so without a distinct
		// error the task would otherwise show as a successful build with no
		// artifact. Surface it through the existing build-error stream
		// (client.go sets StreamBuildTaskOutputRequest.Error from this
		// return value) so it's indistinguishable from any other failure.
		slog.WarnContext(ctx, "artifact extraction failed",
			"task_id", spec.TaskID, "path", spec.ArtifactPath, "error", extractErr)
		return &buildResult, fmt.Errorf("artifact extraction failed: %w", extractErr)
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
		if len(data) == 0 {
			// Asset content must be non-empty server-side, and the upload
			// RPC can't even open its stream for a zero-byte payload (it has
			// no chunk to carry the initial task/name metadata). Fail here
			// with a clear reason instead of letting the upload fail later
			// with a confusing "no messages received" error.
			return nil, "", fmt.Errorf("artifact file %q is empty", path)
		}
		return data, filepath.Base(hdr.Name), nil
	}

	return nil, "", fmt.Errorf("no regular file found at %q", path)
}

// lockFor returns an unlock function that serializes builds sharing the same
// cache key. This prevents concurrent cargo/go builds from racing inside a
// shared target directory (cargo does not support concurrent builds into the
// same target dir). The returned function is intended to be deferred so the
// lock is held for the duration of a Build call. The lock is per-builder-
// process; separate builder processes sharing the same volume coordinate via
// cargo's own file locking, which is safe for read-mostly dependency caches.
func (d *DockerExecutor) lockFor(key string) func() {
	mu, _ := d.cacheMu.LoadOrStore(key, &sync.Mutex{})
	m := mu.(*sync.Mutex)
	m.Lock()
	return m.Unlock
}

// cacheKeyFor derives a stable identifier for a build's cache layout from the
// inputs that determine which dependencies and target artifacts are produced.
// Builds that share a key can safely reuse the same cache volume contents
// (e.g. the same CARGO_TARGET_DIR), while different keys get isolated target
// directories so concurrent builds of distinct profiles don't collide.
func cacheKeyFor(spec BuildSpec) string {
	h := sha256.New()
	for _, s := range []string{
		spec.BuildImage,
		spec.TargetOS,
		spec.SetupScript,
		spec.BuildScript,
		spec.PreBuildScript,
		spec.PostBuildScript,
	} {
		_, _ = h.Write([]byte(s))
		h.Write([]byte{0})
	}
	return hex.EncodeToString(h.Sum(nil))[:16]
}
