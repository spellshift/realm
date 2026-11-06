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

	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/api/types/image"
	"github.com/docker/docker/api/types/mount"
	"github.com/docker/docker/client"
	"github.com/docker/docker/pkg/stdcopy"
	"github.com/google/uuid"
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

	// Prepare a host cache directory and bind-mount it into the container.
	// Cargo and Go write their dependency caches and build artifacts there.
	hostConfig := (*container.HostConfig)(nil)
	cacheEnv := []string{}
	uncachedSpec := spec
	if spec.CacheDir != "" {
		cachePath := spec.CachePath
		if cachePath == "" {
			cachePath = DefaultCachePath
		}
		key := cacheKeyFor(spec)
		if err := prepareCacheDir(spec.CacheDir, key); err != nil {
			slog.WarnContext(ctx, "failed to prepare build cache directory, building without cache",
				"task_id", spec.TaskID, "cache_dir", spec.CacheDir, "error", err)
		} else {
			hostConfig = &container.HostConfig{
				Mounts: []mount.Mount{
					{
						Type:   mount.TypeBind,
						Source: spec.CacheDir,
						Target: cachePath,
					},
				},
			}
			cargoTargetDir := path.Join(cachePath, "target", key)
			cacheEnv = []string{
				"REALM_CACHE_DIR=" + cachePath,
				"CARGO_HOME=" + path.Join(cachePath, "cargo"),
				"CARGO_TARGET_DIR=" + cargoTargetDir,
				"GOPATH=" + path.Join(cachePath, "gopath"),
				"GOMODCACHE=" + path.Join(cachePath, "gopath", "pkg", "mod"),
				"GOCACHE=" + path.Join(cachePath, "go-cache"),
			}
			// Cargo places the binary under CARGO_TARGET_DIR. Keep the path
			// seen by post-build scripts in sync with artifact extraction.
			spec = cachedArtifactSpec(spec, cargoTargetDir)
		}
	}

	slog.InfoContext(ctx, "creating container", "image", spec.BuildImage, "task_id", spec.TaskID)

	// The container entrypoint runs all scripts in /mnt/scripts in order.
	entrypoint := strings.Join([]string{
		"set -e",
		"for s in $(ls /mnt/scripts/*.sh 2>/dev/null | sort); do echo \"==> Running $s\"; . \"$s\"; done",
	}, " && ")

	createContainer := func() (container.CreateResponse, error) {
		return d.client.ContainerCreate(ctx,
			&container.Config{
				Image:      spec.BuildImage,
				Entrypoint: []string{"/bin/sh", "-c", entrypoint},
				Env: append(append(slices.Clone(spec.Env),
					"REALM_WORKSPACE_DIR=/mnt",
					"REALM_TOMES_DIR=/mnt/tomes",
				), cacheEnv...),
			},
			hostConfig, // host config (may bind-mount the build cache directory)
			nil,        // networking config
			nil,        // platform
			"",         // container name (auto-generated)
		)
	}
	resp, err := createContainer()
	if err != nil && hostConfig != nil && ctx.Err() == nil {
		slog.WarnContext(ctx, "failed to mount build cache, retrying without cache",
			"task_id", spec.TaskID, "cache_dir", spec.CacheDir, "error", err)
		spec = uncachedSpec
		hostConfig = nil
		cacheEnv = nil
		resp, err = createContainer()
	}
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

// prepareCacheDir creates the paths the build user writes beneath a bind
// mount. Docker sees the host file modes directly, so no setup container is
// needed.
func prepareCacheDir(cacheDir, key string) error {
	if !filepath.IsAbs(cacheDir) {
		return fmt.Errorf("cache directory must be absolute: %q", cacheDir)
	}
	for _, dir := range []string{
		cacheDir,
		filepath.Join(cacheDir, "cargo"),
		filepath.Join(cacheDir, "target"),
		filepath.Join(cacheDir, "target", key),
		filepath.Join(cacheDir, "gopath"),
		filepath.Join(cacheDir, "gopath", "pkg"),
		filepath.Join(cacheDir, "gopath", "pkg", "mod"),
		filepath.Join(cacheDir, "go-cache"),
	} {
		if err := os.MkdirAll(dir, 0o777); err != nil {
			return fmt.Errorf("creating %q: %w", dir, err)
		}
		if err := os.Chmod(dir, 0o777); err != nil {
			return fmt.Errorf("setting permissions on %q: %w", dir, err)
		}
	}
	return nil
}

// remapArtifactPath rewrites an artifact path that was derived for the default
// workspace target directory (<buildDir>/realm/implants/target/<triple>/release/imix)
// to point inside CARGO_TARGET_DIR. Setting CARGO_TARGET_DIR redirects all cargo
// output there, so the final binary no longer lands at the default path. Paths
// that don't look like a default-derived artifact path are returned unchanged
// (e.g. a user-overridden ArtifactPath).
func remapArtifactPath(artifactPath, cargoTargetDir string) string {
	const defaultTarget = "/tmp/realm-build/realm/implants/target/"
	if strings.HasPrefix(artifactPath, defaultTarget) {
		return path.Join(cargoTargetDir, strings.TrimPrefix(artifactPath, defaultTarget))
	}
	// Tasks created before the recipe directory became stable still carry a
	// UUID suffix in their saved artifact path.
	const legacyPrefix = "/tmp/realm-build-"
	if rest, ok := strings.CutPrefix(artifactPath, legacyPrefix); ok {
		id, suffix, found := strings.Cut(rest, "/realm/implants/target/")
		if found {
			if _, err := uuid.Parse(id); err == nil {
				return path.Join(cargoTargetDir, suffix)
			}
		}
	}
	return artifactPath
}

func cachedArtifactSpec(spec BuildSpec, cargoTargetDir string) BuildSpec {
	artifactPath := remapArtifactPath(spec.ArtifactPath, cargoTargetDir)
	if artifactPath == spec.ArtifactPath {
		return spec
	}
	spec.ArtifactPath = artifactPath
	spec.Env = slices.Clone(spec.Env)
	for i, env := range spec.Env {
		if strings.HasPrefix(env, "ARTIFACT_PATH=") {
			spec.Env[i] = "ARTIFACT_PATH=" + artifactPath
			break
		}
	}
	return spec
}

// cacheKeyFor separates incompatible toolchain images and target platforms.
// Cargo's fingerprints and file locks handle recipe differences and concurrent
// builds within each target directory.
func cacheKeyFor(spec BuildSpec) string {
	h := sha256.New()
	for _, s := range []string{spec.BuildImage, spec.TargetOS} {
		_, _ = h.Write([]byte(s))
		h.Write([]byte{0})
	}
	return hex.EncodeToString(h.Sum(nil))[:16]
}
